//! Asphalt: a road block of tar, gravel and sand, crafted with a bucket of tar from a refinery,
//! which gives the bucket back. It is a little less sticky underfoot than other ground, so
//! walking on it is about a fifth faster.

use weft_sdk::{Block, Registry, tool_tier};

pub static ASPHALT: Block = Block::named("asphalt");

/// Its grip underfoot, as the engine's `friction`: other ground has 0.6. On the ground the engine
/// speeds a walker up by 0.216 / friction³ and keeps friction × 0.91 of its speed a tick, so the
/// speed it settles at goes as 1 / (friction³ × (1 − 0.91 friction)): 2.20 at 0.6, 2.60 at 0.55.
const GRIP: f32 = 0.55;

pub fn register(r: &mut Registry) {
    r.block(&ASPHALT)
        .texture("asphalt")
        .hardness(1.5)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .friction(GRIP)
        .tab(&crate::TAB);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The speed a walker settles at on ground of `friction`, as the engine moves it.
    fn settled_speed(friction: f32) -> f32 {
        0.216 / friction.powi(3) / (1.0 - 0.91 * friction)
    }

    #[test]
    fn walking_on_asphalt_is_about_a_fifth_faster() {
        let faster = settled_speed(GRIP) / settled_speed(0.6);
        assert!((1.15..1.25).contains(&faster), "{faster} times as fast");
    }

    #[test]
    fn asphalt_has_its_name_and_picture() {
        assert!(crate::translated("block.cogwork.asphalt"), "its name");
        assert!(
            crate::has_asset("textures/block/asphalt.png"),
            "its picture"
        );
    }
}
