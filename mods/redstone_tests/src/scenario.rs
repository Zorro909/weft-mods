//! Scenarios run on the server, one at a time, in the region of the player who asked for them.
//! A scenario is a list of steps: `Do` runs code against the world, `Wait(n)` lets exactly n
//! redstone ticks pass (the runner steps every `GAME_TICKS` game ticks, right after redstone's own
//! tick, so a change made by a `Do` is solved by the tick after it), `Until(f, max)` runs `f` once
//! a tick until it returns true, failing after `max` ticks. Every "tick" here is a redstone tick.

use std::collections::VecDeque;
use std::fmt::Debug;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};

use redstone_core::Pos;
use weft_sdk::*;

pub enum Step {
    Do(fn(&mut Ctx)),
    Wait(u32),
    Until(fn(&mut Ctx) -> bool, u32),
}

/// What a scenario has to do with the world across a restart (`scripts/test.sh` runs the pair in
/// two launches on one world). Neither runs with `all`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Restart {
    No,
    /// Leaves its build in place and writes its origin for the `After` one.
    Before,
    /// Runs at the origin the `Before` one wrote.
    After,
}

pub struct Scenario {
    pub name: &'static str,
    pub steps: Vec<Step>,
    /// Run somewhere else: the player is moved to this dimension and position, kept hovering
    /// there, and the origin is that position (+4, 0, +4). Afterwards they go back to the
    /// overworld where they were.
    pub place: Option<(&'static Dimension, [f64; 3])>,
    pub restart: Restart,
    /// A benchmark: runs only when named, never in `all`.
    pub bench: bool,
}

impl Scenario {
    pub fn new(name: &'static str, steps: Vec<Step>) -> Scenario {
        Scenario { name, steps, place: None, restart: Restart::No, bench: false }
    }

    pub fn restart(mut self, r: Restart) -> Scenario {
        self.restart = r;
        self
    }

    pub fn bench(mut self) -> Scenario {
        self.bench = true;
        self
    }

    pub fn in_dimension(mut self, dim: &'static Dimension, pos: [f64; 3]) -> Scenario {
        self.place = Some((dim, pos));
        self
    }
}

static OVERWORLD: Dimension = Dimension::named("base:overworld");

/// Ticks to let a teleport happen and the area around it load.
const SETTLE: u32 = 20;

/// The failures of the whole run once it ended, -1 before.
pub static DONE: AtomicI32 = AtomicI32::new(-1);

/// What a step gets: the world and the scenario's origin, every position relative to it.
pub struct Ctx<'a> {
    pub w: &'a World,
    pub origin: Pos,
    pub player: u64,
    player_pos: [f64; 3],
    failures: &'a mut Vec<String>,
    touched: &'a mut Vec<Pos>,
    hover: &'a mut Option<[f64; 3]>,
    start: &'a mut Option<[f64; 3]>,
}

impl Ctx<'_> {
    pub fn at(&self, rel: Pos) -> Pos {
        [self.origin[0] + rel[0], self.origin[1] + rel[1], self.origin[2] + rel[2]]
    }

    /// Sets a block by its full name, properties included (`base:stone`,
    /// `redstone:lever[facing=up,powered=true]`), and tells redstone, like any mod changing
    /// blocks by code should.
    pub fn set(&mut self, rel: Pos, state: &str) {
        let p = self.at(rel);
        match parse_state(state) {
            Some(s) => {
                if !redstone_api::set_and_announce(self.w, p, s) {
                    self.fail(format!("cannot set {state} at {rel:?} (unloaded?)"));
                }
                self.touched.push(p);
            }
            None => self.fail(format!("unknown state {state}")),
        }
    }

    /// Full state name with properties, as `/tk get` shows it.
    pub fn get(&self, rel: Pos) -> String {
        block_name(self.w.get(self.at(rel))).to_string()
    }

    /// The strength of the wire at `rel`, as redstone solves it.
    pub fn strength(&self, rel: Pos) -> u8 {
        redstone_api::strength(self.w, self.at(rel))
    }

    /// One property of the block at `rel`, empty if it has none.
    pub fn prop(&self, rel: Pos, name: &str) -> String {
        self.w.prop(self.at(rel), name).unwrap_or("").to_string()
    }

    /// Whether the block at `rel` is loaded (writing what is there changes nothing).
    /// The world's time in redstone ticks.
    pub fn ticks(&self) -> u64 {
        self.w.time() / redstone_core::GAME_TICKS as u64
    }

    pub fn loaded(&self, rel: Pos) -> bool {
        let p = self.at(rel);
        self.w.set(p, self.w.get(p))
    }

    /// Moves the player by `by` and keeps them hovering there; they go back where they were
    /// before the first move when the scenario ends. Not for scenarios with a `place`.
    pub fn go(&mut self, by: [f64; 3]) {
        let from = self.hover.unwrap_or(self.player_pos);
        self.start.get_or_insert(self.player_pos);
        let to = [from[0] + by[0], from[1] + by[1], from[2] + by[2]];
        *self.hover = Some(to);
        self.w.teleport(self.player, None, to);
    }

    /// Moves the player to `at` in `dim` and keeps them hovering there; only for scenarios with a
    /// `place`, which end in the overworld where they started.
    pub fn visit(&mut self, dim: &Dimension, at: [f64; 3]) {
        *self.hover = Some(at);
        self.w.teleport(self.player, Some(dim), at);
    }

    /// Puts the player's feet into the cell at `rel` and lets them be there (no hovering), until
    /// they go back where they were when the scenario ends.
    pub fn stand(&mut self, rel: Pos) {
        self.start.get_or_insert(self.player_pos);
        *self.hover = None;
        let p = self.at(rel);
        self.w.teleport(self.player, None, [p[0] as f64 + 0.5, p[1] as f64, p[2] as f64 + 0.5]);
    }

    /// Where the player is, relative to the origin.
    pub fn player_at(&self) -> [f64; 3] {
        let pos = self.w.entity(self.player).map_or(self.player_pos, |e| e.pos);
        std::array::from_fn(|i| pos[i] - self.origin[i] as f64)
    }

    /// Keeps the player hovering where a scenario at this origin starts them (`tick`), for a
    /// scenario continuing one from before a restart.
    pub fn come_back(&mut self) {
        let o = self.origin;
        let at = [o[0] as f64 - 3.5, o[1] as f64 - 24.0, o[2] as f64 - 3.5];
        *self.hover = Some(at);
        self.w.teleport(self.player, None, at);
    }

    /// Back where the player was before the first `go`.
    pub fn go_back(&mut self) {
        if let Some(s) = *self.start {
            *self.hover = Some(s);
            self.w.teleport(self.player, None, s);
        }
    }

    pub fn fail(&mut self, msg: impl Into<String>) {
        self.failures.push(msg.into());
    }

    pub fn check(&mut self, ok: bool, msg: &str) {
        if !ok {
            self.fail(msg.to_string());
        }
    }

    pub fn check_eq<T: PartialEq + Debug>(&mut self, got: T, want: T, what: &str) {
        if got != want {
            self.fail(format!("{what}: got {got:?}, want {want:?}"));
        }
    }
}

/// `ns:block[k=v,...]` to a state id.
pub fn parse_state(text: &str) -> Option<u32> {
    let (name, props) = match text.split_once('[') {
        Some((n, rest)) => (n, rest.strip_suffix(']')?),
        None => (text, ""),
    };
    let mut s = Block::lookup(name)?;
    for kv in props.split(',').filter(|kv| !kv.is_empty()) {
        let (k, v) = kv.split_once('=')?;
        s = state_with(s, k.trim(), v.trim())?;
    }
    Some(s)
}

struct Current {
    idx: usize,
    step: usize,
    wait: u32,
    /// Ticks to do nothing, after a teleport.
    settle: u32,
    /// Where the player was before `Scenario::place` moved them.
    home: Option<[f64; 3]>,
    /// Where `Ctx::go` keeps the player, and where they were before it.
    hover: Option<[f64; 3]>,
    start: Option<[f64; 3]>,
    /// Ticks the current `Until` has waited.
    until: u32,
    /// Going back to the overworld after a scenario somewhere else.
    leaving: bool,
    origin: Option<Pos>,
    failures: Vec<String>,
    touched: Vec<Pos>,
    /// Positions still to clear after the scenario, a slice per tick.
    clearing: Option<Vec<Pos>>,
}

struct Runner {
    all: Vec<Scenario>,
    queue: VecDeque<usize>,
    cur: Option<Current>,
    player: u64,
    running: bool,
    failed: i32,
    lines: Vec<String>,
}

static RUNNER: Mutex<Option<Runner>> = Mutex::new(None);

/// Blocks cleared per tick after a scenario: each change costs up to 7 neighbor updates and the
/// engine drops everything past 65,536 in one flush.
const CLEAR_PER_TICK: usize = 4000;

pub fn register(r: &mut Registry, all: Vec<Scenario>) {
    *RUNNER.lock().unwrap() = Some(Runner { all, queue: VecDeque::new(), cur: None, player: 0, running: false, failed: 0, lines: Vec::new() });
    r.command("rstest", "[names:text]", |cmd| {
        let names = cmd.text("names").unwrap_or("all").to_string();
        let mut guard = RUNNER.lock().unwrap();
        let Some(run) = guard.as_mut() else { return };
        let picked: Vec<usize> = if names == "all" {
            (0..run.all.len()).filter(|&i| run.all[i].restart == Restart::No && !run.all[i].bench).collect()
        } else {
            let mut v = Vec::new();
            for n in names.split(',').map(str::trim) {
                match run.all.iter().position(|s| s.name == n) {
                    Some(i) => v.push(i),
                    None => {
                        cmd.reply("command.redstone_tests.unknown", &[Arg::Text(n)]);
                        return;
                    }
                }
            }
            v
        };
        run.player = cmd.player();
        run.running = true;
        run.queue.extend(picked.iter().copied());
        cmd.reply("command.redstone_tests.queued", &[Arg::Int(picked.len() as i64)]);
    });
    r.world_tick(|w| {
        let mut guard = RUNNER.lock().unwrap();
        if let Some(run) = guard.as_mut() {
            tick(run, w);
        }
    });
}

fn tick(run: &mut Runner, w: &World) {
    if !run.running || !w.players().contains(&run.player) || w.time() % redstone_core::GAME_TICKS as u64 != 0 {
        return;
    }
    if run.cur.is_none() {
        match run.queue.pop_front() {
            Some(idx) => {
                let origin = if run.all[idx].restart == Restart::After { saved_origin() } else { None };
                run.cur = Some(Current { idx, step: 0, wait: 0, settle: 0, home: None, hover: None, start: None, until: 0, leaving: false, origin, failures: Vec::new(), touched: Vec::new(), clearing: None });
            }
            None => return finish(run),
        }
    }
    let player = run.player;
    let cur = run.cur.as_mut().unwrap();
    let place = run.all[cur.idx].place;
    if cur.settle > 0 {
        cur.settle -= 1;
        if cur.settle == 0 && cur.leaving {
            run.cur = None;
        }
        return;
    }
    if let Some(left) = cur.clearing.as_mut() {
        let n = left.len().min(CLEAR_PER_TICK);
        for p in left.drain(..n) {
            w.set(p, 0);
        }
        if left.is_empty() {
            match cur.home.take() {
                Some(home) => {
                    w.teleport(player, Some(&OVERWORLD), home);
                    cur.settle = SETTLE;
                    cur.leaving = true;
                }
                None => run.cur = None,
            }
        }
        return;
    }
    let Some(st) = w.entity(player) else { return };
    if let Some((dim, at)) = place {
        if cur.home.is_none() {
            cur.home = Some(st.pos);
            w.teleport(player, Some(dim), at);
            cur.settle = SETTLE;
            return;
        }
    }
    // Keep them hovering there, so the area stays loaded around the scenario.
    if let Some(at) = cur.hover.or(place.map(|(_, at)| at)) {
        if (0..3).any(|i| (st.pos[i] - at[i]).abs() > 0.5) {
            w.teleport(player, None, at);
        }
    }
    let origin = match cur.origin {
        Some(o) => o,
        None => {
            // Above the player (or at the scenario's place), in loaded air.
            let (base, up) = match place {
                Some((_, at)) => (at, 0),
                None => (st.pos, 24),
            };
            let o = [base[0].floor() as i32 + 4, base[1].floor() as i32 + up, base[2].floor() as i32 + 4];
            if !w.set(o, w.get(o)) {
                return;
            }
            cur.origin = Some(o);
            o
        }
    };
    if cur.wait > 0 {
        cur.wait -= 1;
        if cur.wait > 0 {
            return;
        }
    }
    let scenario = &run.all[cur.idx];
    while cur.step < scenario.steps.len() {
        let step = &scenario.steps[cur.step];
        cur.step += 1;
        match step {
            Step::Wait(n) => {
                cur.wait = *n;
                if *n > 0 {
                    return;
                }
            }
            Step::Do(f) => {
                let mut ctx = Ctx { w, origin, player, player_pos: st.pos, failures: &mut cur.failures, touched: &mut cur.touched, hover: &mut cur.hover, start: &mut cur.start };
                run_step(&mut ctx, |c| f(c));
            }
            Step::Until(f, max) => {
                let mut ctx = Ctx { w, origin, player, player_pos: st.pos, failures: &mut cur.failures, touched: &mut cur.touched, hover: &mut cur.hover, start: &mut cur.start };
                if !run_step(&mut ctx, |c| f(c)).unwrap_or(true) {
                    if cur.until < *max {
                        cur.until += 1;
                        cur.step -= 1;
                        return;
                    }
                    // Go on: the steps after it still clean up (settings, the player's place).
                    cur.failures.push(format!("step {} still false after {max} ticks", cur.step));
                }
                cur.until = 0;
            }
        }
    }
    let line = if cur.failures.is_empty() { format!("PASS {}", scenario.name) } else { format!("FAIL {}: {}", scenario.name, cur.failures.join("; ")) };
    info!("{line}");
    if !cur.failures.is_empty() {
        run.failed += 1;
    }
    run.lines.push(line);
    if scenario.restart == Restart::Before {
        save_origin(origin);
        cur.clearing = Some(Vec::new());
    } else {
        cur.clearing = Some(clear_area(&cur.touched));
    }
    // Back where `Ctx::go` took them from first, so the area loads again before it is cleared;
    // before a restart they stay away, so the game quits with the area unloaded.
    cur.hover = None;
    if let Some(s) = cur.start.take()
        && scenario.restart != Restart::Before
    {
        w.teleport(player, None, s);
        cur.settle = SETTLE;
    }
}

/// Runs a step, a panic counting as a failure (and `None`).
fn run_step<T>(ctx: &mut Ctx, f: impl FnOnce(&mut Ctx) -> T) -> Option<T> {
    match catch_unwind(AssertUnwindSafe(|| f(ctx))) {
        Ok(v) => Some(v),
        Err(e) => {
            let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
            ctx.fail(format!("panicked: {msg}"));
            None
        }
    }
}

/// Where a `Restart::Before` scenario leaves its origin: next to the report.
fn origin_file() -> std::path::PathBuf {
    let report = launch_option("redstone-report").unwrap_or_else(|| "redstone.txt".to_string());
    std::path::Path::new(&report).with_file_name("restart-origin.txt")
}

fn save_origin(o: Pos) {
    let path = origin_file();
    if let Err(e) = std::fs::write(&path, format!("{} {} {}\n", o[0], o[1], o[2])) {
        error!("cannot write {}: {e}", path.display());
    }
}

fn saved_origin() -> Option<Pos> {
    let text = std::fs::read_to_string(origin_file()).ok()?;
    let v: Vec<i32> = text.split_whitespace().filter_map(|t| t.parse().ok()).collect();
    (v.len() == 3).then(|| [v[0], v[1], v[2]])
}

/// The box around everything a scenario set, one block wider (pistons and wire reach out),
/// only what isn't air yet.
fn clear_area(touched: &[Pos]) -> Vec<Pos> {
    if touched.is_empty() {
        return Vec::new();
    }
    let mut lo = touched[0];
    let mut hi = touched[0];
    for p in touched {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    let mut v = Vec::new();
    for x in lo[0] - 1..=hi[0] + 1 {
        for y in lo[1] - 1..=hi[1] + 1 {
            for z in lo[2] - 1..=hi[2] + 1 {
                v.push([x, y, z]);
            }
        }
    }
    v
}

fn finish(run: &mut Runner) {
    run.running = false;
    run.lines.push(format!("DONE {}", run.failed));
    if let Some(path) = launch_option("redstone-report") {
        if let Some(dir) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&path, run.lines.join("\n") + "\n") {
            error!("cannot write {path}: {e}");
        }
    }
    info!("redstone tests done, {} failed", run.failed);
    DONE.store(run.failed, Ordering::Relaxed);
    run.failed = 0;
    run.lines.clear();
}
