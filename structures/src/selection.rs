//! The box a player selects with the wand or `/structure pos1` and `pos2`, which `save` writes
//! when it gets no corners. The server keeps each player's box; their client draws it.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use weft_sdk::*;

/// How far away a looked-at block can be taken as a corner.
const REACH: f64 = 64.0;
/// The box's outline, `0xRRGGBBAA`.
const COLOR: u32 = 0xffd2_3cff;

value! {
    /// A selected box, both corners included.
    pub struct Shown { pub from: [i32; 3], pub to: [i32; 3] }
}

/// Server to client: the player's box, once both corners are set.
static SHOWN: Channel<Shown> = Channel::named("selection");

/// By player entity.
static SELECTIONS: LazyLock<Mutex<HashMap<u64, Selection>>> = LazyLock::new(Default::default);

/// On the client: the box this player selected.
static DRAWN: Mutex<Option<Shown>> = Mutex::new(None);

#[derive(Clone, Copy, Default)]
struct Selection {
    dimension: u32,
    corners: [Option<[i32; 3]>; 2],
}

impl Selection {
    /// Lowest corner, then highest.
    fn bounds(&self) -> Option<([i32; 3], [i32; 3])> {
        let [Some(a), Some(b)] = self.corners else {
            return None;
        };
        Some((
            [0, 1, 2].map(|i| a[i].min(b[i])),
            [0, 1, 2].map(|i| a[i].max(b[i])),
        ))
    }

    /// What the player sees: the box, or the one corner set so far.
    fn shown(&self) -> Option<Shown> {
        let [a, b] = self.corners;
        let (from, to) = self.bounds().or_else(|| a.or(b).map(|c| (c, c)))?;
        Some(Shown { from, to })
    }
}

/// The client's half: draws the player's box every frame.
pub fn register(r: &mut Registry) {
    r.client_packet(&SHOWN, |_, shown| *lock(&DRAWN) = Some(shown));
    r.client_frame(|client| {
        if let Some(Shown { from, to }) = *lock(&DRAWN) {
            let (min, max) = (from.map(|v| v as f64), to.map(|v| v as f64 + 1.0));
            client.draw_box(None, min, max, COLOR);
        }
    });
}

/// `/structure pos1 [at]`.
pub fn first(cmd: &mut ServerCommand) {
    set(cmd, 0);
}

/// `/structure pos2 [at]`.
pub fn second(cmd: &mut ServerCommand) {
    set(cmd, 1);
}

/// The box `player` selected in `dimension`: lowest corner, then highest.
pub fn selected(dimension: u32, player: u64) -> Option<([i32; 3], [i32; 3])> {
    let selections = lock(&SELECTIONS);
    let selection = selections.get(&player)?;
    (selection.dimension == dimension).then(|| selection.bounds())?
}

/// Corner `corner` at `at`, else the block the player looks at, else where they stand.
fn set(cmd: &mut ServerCommand, corner: usize) {
    let (world, player) = (cmd.world(), cmd.player());
    let at = cmd
        .pos("at")
        .map(|pos| pos.map(|v| v.floor() as i32))
        .or_else(|| world.look_at(player, REACH, false).map(|look| look.pos))
        .or_else(|| {
            world
                .entity(player)
                .map(|me| me.pos.map(|v| v.floor() as i32))
        });
    let Some(at) = at else {
        return cmd.reply("structures.no_player", &[]);
    };
    set_corner(&world, player, corner, at);
    let [x, y, z] = at.map(|v| Arg::Int(v as i64));
    cmd.reply(
        "structures.corner_set",
        &[Arg::Int(corner as i64 + 1), x, y, z],
    );
}

/// Corner `corner` (0 or 1) of `player`'s selection at `at`, shown to them. Corners chosen in
/// another dimension are forgotten.
pub fn set_corner(world: &World, player: u64, corner: usize, at: [i32; 3]) {
    let selection = {
        let mut selections = lock(&SELECTIONS);
        let selection = selections.entry(player).or_default();
        if selection.dimension != world.dimension() {
            *selection = Selection {
                dimension: world.dimension(),
                ..Selection::default()
            };
        }
        selection.corners[corner] = Some(at);
        *selection
    };
    if let Some(shown) = selection.shown() {
        world.send(player, &SHOWN, &shown);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_needs_both_corners() {
        let mut selection = Selection::default();
        selection.corners[0] = Some([5, 64, -2]);
        assert_eq!(selection.bounds(), None, "one corner");
        selection.corners[1] = Some([1, 70, -9]);
        assert_eq!(
            selection.bounds(),
            Some(([1, 64, -9], [5, 70, -2])),
            "lowest corner first, whichever was set first"
        );
    }

    #[test]
    fn one_corner_is_shown_alone() {
        let mut selection = Selection::default();
        selection.corners[1] = Some([2, 3, 4]);
        let shown = selection.shown().expect("a corner");
        assert_eq!(
            (shown.from, shown.to),
            ([2, 3, 4], [2, 3, 4]),
            "the one block"
        );
    }
}
