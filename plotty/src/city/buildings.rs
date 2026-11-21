//! structgen's buildings, placed on a free plot by a game master: found by their template name,
//! so Plotty needs no structgen to load, and turned with the copies structgen registers (the
//! engine's own turning skips log axes and ladder sides). A building's front is its north side.

use weft_sdk::{Template, Transform, template_size};

use super::grid::Side;

/// structgen's houses and workshops that stand on a plot, by template name.
pub const BUILDINGS: [&str; 22] = [
    "crofter_cottage",
    "dormer_cottage",
    "corner_house",
    "hall_house",
    "narrow_gable",
    "longhouse",
    "smithy_house",
    "open_forge",
    "storehouse",
    "staddle_granary",
    "stable",
    "great_barn",
    "hof",
    "lumberyard",
    "coaching_inn",
    "wayside_tavern",
    "wayside_chapel",
    "guildhall",
    "town_hall",
    "village_church",
    "market_hall",
    "draw_well",
];

/// Blocks kept free between a building and its plot's edge, on every side.
const MARGIN: i32 = 1;

/// The name of `building`'s copy whose front faces `street`.
pub fn turned(building: &str, street: Side) -> String {
    let suffix = match street {
        Side::North => "",
        Side::East => "/right",
        Side::South => "/back",
        Side::West => "/left",
    };
    format!("structgen:{building}{suffix}")
}

/// A template's size along x, y and z; `None` where no mod registers it.
pub fn size_of(template: &Template) -> Option<[i32; 3]> {
    let size = template_size(template, Transform::NONE);
    (size.x > 0).then_some([size.x, size.y, size.z])
}

/// Whether something `width` by `height` by `depth` fits a plot `blocks` across with its
/// margin, and under the height a build saves for its undo.
pub fn fits([width, height, depth]: [i32; 3], blocks: i32) -> bool {
    width + 2 * MARGIN <= blocks && depth + 2 * MARGIN <= blocks && height <= super::place::ABOVE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_building_turns_to_face_its_street() {
        assert_eq!(
            turned("corner_house", Side::North),
            "structgen:corner_house",
            "as drawn"
        );
        assert_eq!(
            turned("corner_house", Side::South),
            "structgen:corner_house/back",
            "turned round"
        );
    }

    #[test]
    fn cottages_fit_a_small_plot_and_halls_a_normal_one() {
        assert!(fits([9, 9, 7], 16), "a crofter's cottage on 16");
        assert!(!fits([21, 30, 18], 16), "a town hall is too big for 16");
        assert!(fits([21, 30, 18], 32), "but fits 32");
        assert!(!fits([15, 5, 15], 16), "no room for the margin");
        assert!(!fits([9, 40, 7], 32), "taller than undo saves");
    }
}
