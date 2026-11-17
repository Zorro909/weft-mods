//! Lubricant from a refinery, led into a quarry by liquid tube: while it holds some, each block
//! its drill mines costs a fifth less energy, at a fifth less a tick so it takes as long, uses a
//! millibucket of lubricant, and wears its bit only every other block. A quarry is a holder that
//! takes lubricant alone (`liquid::Kind::Lubed`); a Gantry Excavator, which digs with no bit,
//! saves only energy.

use weft_sdk::World;

use crate::liquid::HELD;
use crate::refinery::products::Product;

/// Millibuckets of lubricant a quarry holds.
pub const CAPACITY: u32 = 1_000;
/// Millibuckets a block mined with it uses.
const A_BLOCK: u32 = 1;

/// Whether the quarry `id` holds lubricant for a block.
pub fn has(world: &World, id: u64) -> bool {
    let held = world.component(id, &HELD).unwrap_or_default();
    held.liquid == Product::Lubricant.key() && held.amount >= A_BLOCK
}

/// Uses the lubricant of a block mined by the quarry `id`.
pub fn use_up(world: &World, id: u64) {
    if let Some(held) = world.component_mut(id, &HELD) {
        held.remove(A_BLOCK);
    }
}

/// `energy` (a block's cost, or a drill's draw) a fifth less, as lubricant makes it.
pub fn eased(energy: i64) -> i64 {
    energy - energy / 5
}

/// Whether a block just mined, `lubed` or not, wears the bit; `skipped`, 1 while the last
/// lubricated block skipped its wear, keeps the count between blocks.
pub fn wears(lubed: bool, skipped: &mut u32) -> bool {
    if !lubed {
        return true;
    }
    *skipped ^= 1;
    *skipped == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lubricant_takes_a_fifth_off() {
        assert_eq!(eased(1_000), 800, "stone");
        assert_eq!(eased(120), 96, "the Gantry's draw");
        assert_eq!(eased(18), 15, "less a fifth, rounded down");
    }

    #[test]
    fn lubricant_halves_the_wear() {
        let mut skipped = 0;
        let worn = (0..100).filter(|_| wears(true, &mut skipped)).count();
        assert_eq!(worn, 50, "every other block");
        let worn = (0..100).filter(|_| wears(false, &mut skipped)).count();
        assert_eq!(worn, 100, "every block without");
    }
}
