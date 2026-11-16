//! Boxes Plotty outlines for one player: a build's preview, the plots around them. The server
//! sends each box to the player, and their client draws it every frame until its time is up.
//! The engine (v26.11.5) draws a mod's server-side gizmos only while the player shows some
//! gizmo group with `/gizmo`, so `World::draw_box` alone would show nothing.

use std::sync::{Mutex, MutexGuard, PoisonError};

use weft_sdk::{Channel, Client, Output, Registry, Vec3, World, value};
use weftkit::Area;

/// Seconds in a tick.
const TICK_SECS: f32 = 0.05;

value! {
    /// A box round the blocks from `lo` to `hi`, both included, in color `0xRRGGBBAA`, shown for
    /// `ticks`.
    pub struct Outline {
        pub lo: [i32; 3],
        pub hi: [i32; 3],
        pub color: u32,
        pub ticks: u32,
    }
}

static OUTLINE: Channel<Outline> = Channel::named("outline");

/// On the client: the boxes shown, each with its seconds left.
static SHOWN: Mutex<Vec<(Outline, f32)>> = Mutex::new(Vec::new());

pub fn register(r: &mut Registry) {
    r.client_packet(&OUTLINE, |_, outline| {
        let secs = outline.ticks as f32 * TICK_SECS;
        lock().push((outline, secs));
    });
    r.client_frame(frame);
}

/// Outlines the blocks of `area` for `player` in `color` (`0xRRGGBBAA`) for `ticks`.
pub fn show(world: &World, player: u64, area: Area, color: u32, ticks: u32) {
    let outline = Outline {
        lo: [area.lo.x, area.lo.y, area.lo.z],
        hi: [area.hi.x, area.hi.y, area.hi.z],
        color,
        ticks,
    };
    world.send(player, &OUTLINE, &outline);
}

/// Draws the boxes still shown, and forgets those whose time is up.
fn frame(client: &Client) {
    let mut shown = lock();
    age(&mut shown, client.state().dt);
    for (outline, _) in shown.iter() {
        let [lo, hi] = [outline.lo, outline.hi.map(|at| at + 1)]
            .map(|[x, y, z]| Vec3::new(f64::from(x), f64::from(y), f64::from(z)));
        client.draw_box(None, lo, hi, outline.color);
    }
}

/// `shown` after `dt` seconds more: each box's time shorter, and those whose time is up gone.
fn age(shown: &mut Vec<(Outline, f32)>, dt: f32) {
    for (_, left) in shown.iter_mut() {
        *left -= dt;
    }
    shown.retain(|(_, left)| *left > 0.0);
}

fn lock() -> MutexGuard<'static, Vec<(Outline, f32)>> {
    SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_outline_stays_its_ticks() {
        let outline = Outline {
            lo: [0; 3],
            hi: [15, 8, 15],
            color: 0xFF8C2AFF,
            ticks: 20,
        };
        let mut shown = vec![(outline, outline.ticks as f32 * TICK_SECS)];
        age(&mut shown, 0.9);
        assert_eq!(shown.len(), 1, "a second's box after 0.9 s");
        age(&mut shown, 0.2);
        assert!(shown.is_empty(), "gone after 1.1 s");
    }
}
