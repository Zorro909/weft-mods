//! What quarries bring up and the hand machines work: Rubble, broken stone that falls like gravel
//! and sifts into sand, gravel and now and then something better (`sifting`); and Ore Rubble, an
//! item of one metal (`metal`) at one grade (`grade`), which grinds into that metal's raw ore
//! (`grinding`). Quarries bring up both (`quarry`); every Ore Rubble starts Poor, and machines
//! raise its grade a step at a time (`raise`).

mod grade;
pub mod grinding;
mod metal;
mod quarry;
pub mod raise;
pub mod sifting;

pub use grade::Grade;
pub use metal::{Ore, ore_name};
pub use quarry::quarried;
use weft_sdk::{Block, Component, Item, ItemStack, Registry, StackData, abi, tool_tier};

pub static RUBBLE: Block = Block::named("rubble");
pub static ORE_RUBBLE: Item = Item::named("ore_rubble");
/// The metal of a stack of Ore Rubble.
pub static ORE: Component<Ore> = Component::named("cogwork:ore");
/// The grade of a stack of Ore Rubble, a `Grade` kept as its number.
pub static GRADE: Component<u32> = Component::named("cogwork:grade");

pub fn register(r: &mut Registry) {
    // Both change how a stack looks, and stacks that differ in either don't merge.
    r.stack_component(&ORE, abi::stack_flags::VISUAL);
    r.stack_component(&GRADE, abi::stack_flags::VISUAL);
    r.block(&RUBBLE)
        .sound_group("base:block.gravel")
        .hardness(0.6)
        .mined_by("shovel", tool_tier::HAND)
        .falling()
        .tab(&crate::TAB);
    r.item(&ORE_RUBBLE)
        .condition(|stack, looks| {
            if let Some((ore, grade)) = ore_of(stack) {
                looks.texture(&look(&ore, grade));
            }
        })
        .tab(&crate::TAB);
}

/// `count` Ore Rubble of `ore` at `grade`.
pub fn ore_rubble(ore: Ore, grade: Grade, count: u32) -> ItemStack {
    ORE_RUBBLE
        .stack(count)
        .with(&ORE, ore)
        .with(&GRADE, grade as u32)
}

/// The metal and grade of `stack`, when it is Ore Rubble of a metal.
pub fn ore_of(stack: &ItemStack) -> Option<(Ore, Grade)> {
    if stack.item != ORE_RUBBLE.id() {
        return None;
    }
    let ore = ORE
        .on_stack_own(stack)
        .filter(|ore| !ore.metal().is_empty())?;
    let grade = GRADE.on_stack_own(stack).unwrap_or_default();
    Some((ore, Grade::from_saved(grade)))
}

/// The pictures Ore Rubble of `ore` at `grade` shows: grey stone, and over it flecks in the
/// metal's color, more of them the richer it is.
fn look(ore: &Ore, grade: Grade) -> String {
    let flecks: &str = grade.into();
    format!("ore_rubble ore_rubble_{flecks}#{:06X}", ore.tint())
}

#[cfg(test)]
mod tests {
    use strum::VariantArray;

    use super::*;

    #[test]
    fn ore_rubble_shows_its_metal_and_grade() {
        let iron = Ore::of("iron").expect("iron fits");
        assert_eq!(
            look(&iron, Grade::Rich),
            "ore_rubble ore_rubble_rich#D8A07A",
            "rich flecks in iron's color"
        );
    }

    #[test]
    fn the_pictures_of_rubble_are_there() {
        assert!(crate::has_asset("textures/block/rubble.png"), "rubble");
        assert!(
            crate::has_asset("textures/item/ore_rubble.png"),
            "ore rubble"
        );
        for &grade in Grade::VARIANTS {
            let name: &str = grade.into();
            let file = format!("textures/item/ore_rubble_{name}.png");
            assert!(crate::has_asset(&file), "no {file}");
        }
    }
}
