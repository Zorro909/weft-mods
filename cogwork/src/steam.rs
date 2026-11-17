//! Steam: what a boiler makes of water over a firebox's fire, and what drives the steam engine
//! and the trip hammer. It is a Cogwork liquid that rises: in liquid tubes and tanks it spreads to
//! where they are emptier, up as well as down (`liquid::flow::pressure`), and poured into the
//! world it flows up. It comes in no bucket. Steam machines let out a puff of it as they work.

use std::sync::LazyLock;

use weft_sdk::{Block, Output, Particle, Registry, Vec3, World};

use crate::liquid::{LiquidKey, key_named};

pub static STEAM: Block = Block::named("steam");
/// A white puff that rises and fades: out of a working steam engine and a trip hammer.
static PUFF: Particle = Particle::named("steam_puff");

pub fn register(r: &mut Registry) {
    // Thin and quick, and see-through, as steam is.
    r.liquid(&STEAM)
        .block(|block| block.translucent().light_opacity(0))
        .flows_up()
        .flow_delay(5)
        .drop_off(2)
        .fog(0xc8ccd0, 0.0, 3.0);
    r.particle(&PUFF)
        .textures(&["core:generic_3", "core:generic_4", "core:generic_5"])
        .life(0.8, 1.4)
        .size(0.2, 0.35)
        .gravity(-0.12)
        .drag(0.92)
        .color(0xF0F4F8D0, 0xE0E4E800)
        .translucent();
}

/// The key of steam, as holders keep liquids.
pub fn key() -> LiquidKey {
    static KEY: LazyLock<LiquidKey> = LazyLock::new(|| key_named("cogwork:steam"));
    *KEY
}

/// Lets `count` puffs of steam out at `at`, rising.
pub fn puff(world: &World, at: Vec3, count: u32) {
    let burst = world.spawn_particles(&PUFF, at).count(count);
    drop(
        burst
            .spread(0.12)
            .velocity(Vec3::new(0.0, 0.25, 0.0))
            .speed(0.06),
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn steam_has_its_picture() {
        assert!(crate::has_asset("textures/block/steam_still.png"), "steam");
    }
}
