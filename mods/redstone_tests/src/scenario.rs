//! Scenarios run on the server, one at a time, in the region of the player who asked for them.
//! A scenario is a list of steps: `Do` runs code against the world, `Wait(n)` lets exactly n world
//! ticks pass (redstone's own tick runs before ours, so a change made by a `Do` is solved by the
//! tick after it).

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
}

pub struct Scenario {
    pub name: &'static str,
    pub steps: Vec<Step>,
    /// Run somewhere else: the player is moved to this dimension and position, kept hovering
    /// there, and the origin is that position (+4, 0, +4). Afterwards they go back to the
    /// overworld where they were.
    pub place: Option<(&'static Dimension, [f64; 3])>,
}

impl Scenario {
    pub fn new(name: &'static str, steps: Vec<Step>) -> Scenario {
        Scenario { name, steps, place: None }
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
    failures: &'a mut Vec<String>,
    touched: &'a mut Vec<Pos>,
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
            (0..run.all.len()).collect()
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
    if !run.running || !w.players().contains(&run.player) {
        return;
    }
    if run.cur.is_none() {
        match run.queue.pop_front() {
            Some(idx) => run.cur = Some(Current { idx, step: 0, wait: 0, settle: 0, home: None, origin: None, failures: Vec::new(), touched: Vec::new(), clearing: None }),
            None => return finish(run),
        }
    }
    let player = run.player;
    let cur = run.cur.as_mut().unwrap();
    let place = run.all[cur.idx].place;
    if cur.settle > 0 {
        cur.settle -= 1;
        if cur.settle == 0 && cur.clearing.is_some() {
            // Back home after a scenario somewhere else.
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
        // Keep them hovering there, so the area stays loaded around the scenario.
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
                let mut ctx = Ctx { w, origin, player, failures: &mut cur.failures, touched: &mut cur.touched };
                if let Err(e) = catch_unwind(AssertUnwindSafe(|| f(&mut ctx))) {
                    let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    cur.failures.push(format!("panicked: {msg}"));
                }
            }
        }
    }
    let line = if cur.failures.is_empty() { format!("PASS {}", scenario.name) } else { format!("FAIL {}: {}", scenario.name, cur.failures.join("; ")) };
    info!("{line}");
    if !cur.failures.is_empty() {
        run.failed += 1;
    }
    run.lines.push(line);
    cur.clearing = Some(clear_area(&cur.touched));
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
