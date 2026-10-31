//! In-game tests for the redstone mod.
//!
//! `/rstest [names]` runs scenarios (`all` or a comma list) in the player's world. Launched with
//! `--redstone-test <names>` the game opens the world given by `--world`, runs them, writes
//! `--redstone-report <file>` and quits with the number of failed scenarios (`scripts/test.sh`).

mod ext;
mod scenario;
mod tests;

use std::sync::atomic::{AtomicU32, Ordering};

use weft_sdk::*;

declare_mod!(deps: ["base", "redstone"], init: init);

fn init(r: &mut Registry) {
    ext::register(r);
    scenario::register(r, tests::all());
    let Some(names) = launch_option("redstone-test") else { return };
    r.client_frame(move |c| {
        static PHASE: AtomicU32 = AtomicU32::new(0);
        match PHASE.load(Ordering::Relaxed) {
            0 if c.state().flags & weft_sdk::client::client_flags::JOINED != 0 => {
                c.run_command("/rule core:mob_spawning 0");
                c.run_command(&format!("/rstest {names}"));
                PHASE.store(1, Ordering::Relaxed);
            }
            1 => {
                let done = scenario::DONE.load(Ordering::Relaxed);
                if done >= 0 {
                    PHASE.store(2, Ordering::Relaxed);
                    c.quit(done);
                }
            }
            _ => {}
        }
    });
}
