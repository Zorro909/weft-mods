//! What a game master builds: the plaza a city is founded with, roads, plots in three sizes and
//! market squares, each a template drawn here in boxes of base's blocks, in the rustic look of
//! cobblestone and stone bricks. Layer 0 is the ground the city is levelled to; a plot's post
//! is set by `place.rs`, beside the street it faces.

use weft_sdk::{Registry, Template, TemplateBuilder};

use super::{CellKind, PlotSize};

pub static PLAZA: Template = Template::named("plaza");
pub static ROAD: Template = Template::named("road");
pub static SMALL_PLOT: Template = Template::named("small_plot");
pub static NORMAL_PLOT: Template = Template::named("normal_plot");
pub static LARGE_PLOT: Template = Template::named("large_plot");
pub static MARKET: Template = Template::named("market");

const PAVING: &str = "base:cobblestone";
const EDGING: &str = "base:stone_bricks";
const RENT_POST: &str = "plotty:rent_post";
const KERB: &str = "base:stone_bricks_slab";
const GRASS: &str = "base:grass";
const POST: &str = "base:oak_log";
const LIGHT: &str = "base:glowstone";
const WATER: &str = "base:water";
pub const CITY_STONE: &str = "plotty:city_stone";

/// Where a market's six places start, from its lowest corner, and their size along x and z.
pub const PLACES: [[i32; 2]; 6] = [[3, 3], [13, 3], [24, 3], [3, 25], [13, 25], [24, 25]];
pub const PLACE_SIZE: [i32; 2] = [5, 4];

/// Something `/city build` puts down.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Plaza,
    Road,
    Small,
    Normal,
    Large,
    Market,
}

/// What one kind is: the word for it in commands, its template, cells along a side, what its
/// cells become, and the plot it makes.
struct Row {
    kind: Kind,
    word: &'static str,
    template: &'static Template,
    side: i32,
    cells: CellKind,
    plot: Option<PlotSize>,
}

const fn row(
    kind: Kind,
    word: &'static str,
    template: &'static Template,
    side: i32,
    cells: CellKind,
    plot: Option<PlotSize>,
) -> Row {
    Row {
        kind,
        word,
        template,
        side,
        cells,
        plot,
    }
}

const KINDS: [Row; 6] = [
    row(Kind::Plaza, "plaza", &PLAZA, 2, CellKind::Plaza, None),
    row(Kind::Road, "road", &ROAD, 1, CellKind::Road, None),
    row(
        Kind::Small,
        "small",
        &SMALL_PLOT,
        1,
        CellKind::Plot,
        Some(PlotSize::Small),
    ),
    row(
        Kind::Normal,
        "normal",
        &NORMAL_PLOT,
        2,
        CellKind::Plot,
        Some(PlotSize::Normal),
    ),
    row(
        Kind::Large,
        "large",
        &LARGE_PLOT,
        4,
        CellKind::Plot,
        Some(PlotSize::Large),
    ),
    row(Kind::Market, "market", &MARKET, 2, CellKind::Market, None),
];

/// The words for the kinds, as a command's usage lists them.
#[cfg(test)]
pub const WORDS: &str = "plaza,road,small,normal,large,market";

impl Kind {
    pub fn named(word: &str) -> Option<Kind> {
        KINDS
            .iter()
            .find(|row| row.word == word)
            .map(|row| row.kind)
    }

    fn row(self) -> &'static Row {
        KINDS
            .iter()
            .find(|row| row.kind == self)
            .expect("every kind has a row")
    }

    pub fn word(self) -> &'static str {
        self.row().word
    }
    pub fn template(self) -> &'static Template {
        self.row().template
    }
    pub fn side(self) -> i32 {
        self.row().side
    }
    pub fn cell_kind(self) -> CellKind {
        self.row().cells
    }
    pub fn plot_size(self) -> Option<PlotSize> {
        self.row().plot
    }
}

pub fn register(r: &mut Registry) {
    plaza(r.template(&PLAZA));
    road(r.template(&ROAD));
    plot(r.template(&SMALL_PLOT), 16);
    plot(r.template(&NORMAL_PLOT), 32);
    plot(r.template(&LARGE_PLOT), 64);
    market(r.template(&MARKET));
}

/// A square of cobblestone edged and crossed in stone bricks, a well in the middle, lamps round
/// it and the city stone beside the well.
fn plaza(t: TemplateBuilder) {
    let t = edged_square(t, 32)
        .fill(EDGING, [15, 0, 0], [16, 0, 31])
        .fill(EDGING, [0, 0, 15], [31, 0, 16]);
    let t = well(t, 14);
    let t = [
        [2, 2],
        [29, 2],
        [2, 29],
        [29, 29],
        [2, 9],
        [2, 22],
        [29, 9],
        [29, 22],
    ]
    .into_iter()
    .fold(t, lamp);
    t.fill(CITY_STONE, [18, 1, 13], [18, 1, 13]);
}

/// A cell of cobblestone street with a lamp at one corner.
fn road(t: TemplateBuilder) {
    let t = t.fill(PAVING, [0, 0, 0], [15, 0, 15]);
    lamp(t, [0, 0]);
}

/// Levelled grass with a low stone kerb round it, `side` blocks across.
fn plot(t: TemplateBuilder, side: i32) {
    let last = side - 1;
    t.fill(GRASS, [0, 0, 0], [last, 0, last])
        .fill(KERB, [0, 1, 0], [last, 1, 0])
        .fill(KERB, [0, 1, last], [last, 1, last])
        .fill(KERB, [0, 1, 1], [0, 1, last - 1])
        .fill(KERB, [last, 1, 1], [last, 1, last - 1]);
}

/// A square with six places outlined for stalls, each with its rent post at the corner toward
/// the middle, a well in the middle and lamps at the corners.
fn market(t: TemplateBuilder) {
    let t = PLACES.into_iter().fold(edged_square(t, 32), |t, [x, z]| {
        let [width, depth] = PLACE_SIZE;
        t.fill(EDGING, [x - 1, 0, z - 1], [x + width, 0, z + depth])
            .fill(PAVING, [x, 0, z], [x + width - 1, 0, z + depth - 1])
    });
    let t = PLACES.into_iter().fold(t, |t, [x, z]| {
        let [_, depth] = PLACE_SIZE;
        let toward_middle = if z < 16 { z + depth } else { z - 1 };
        t.fill(
            RENT_POST,
            [x - 1, 1, toward_middle],
            [x - 1, 1, toward_middle],
        )
    });
    let t = well(t, 14);
    [[1, 1], [30, 1], [1, 30], [30, 30]]
        .into_iter()
        .fold(t, lamp);
}

/// Cobblestone `side` blocks across with a stone brick edge.
fn edged_square(t: TemplateBuilder, side: i32) -> TemplateBuilder {
    let last = side - 1;
    t.fill(PAVING, [0, 0, 0], [last, 0, last])
        .fill(EDGING, [0, 0, 0], [last, 0, 0])
        .fill(EDGING, [0, 0, last], [last, 0, last])
        .fill(EDGING, [0, 0, 0], [0, 0, last])
        .fill(EDGING, [last, 0, 0], [last, 0, last])
}

/// A well four blocks across from `at` along x and z: a stone brick rim round two by two of water.
fn well(t: TemplateBuilder, at: i32) -> TemplateBuilder {
    let far = at + 3;
    t.fill(EDGING, [at, 0, at], [far, 1, far]).fill(
        WATER,
        [at + 1, 0, at + 1],
        [far - 1, 1, far - 1],
    )
}

/// A log post two blocks high with a glowstone lamp on top.
fn lamp(t: TemplateBuilder, [x, z]: [i32; 2]) -> TemplateBuilder {
    t.fill(POST, [x, 1, z], [x, 2, z])
        .fill(LIGHT, [x, 3, z], [x, 3, z])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_a_word_and_back() {
        for Row { kind, word, .. } in KINDS {
            assert_eq!(Kind::named(word), Some(kind), "{word} names its kind");
            assert!(
                WORDS.split(',').any(|listed| listed == word),
                "{word} is in the usage"
            );
        }
    }

    #[test]
    fn plot_kinds_are_16_32_and_64_blocks_across() {
        let blocks = [Kind::Small, Kind::Normal, Kind::Large]
            .map(|kind| kind.side() * super::super::grid::CELL);
        assert_eq!(blocks, [16, 32, 64], "a side each");
    }

    #[test]
    fn market_places_stay_inside_and_apart() {
        let [width, depth] = PLACE_SIZE;
        for (index, [x, z]) in PLACES.into_iter().enumerate() {
            assert!(
                x >= 2 && z >= 2 && x + width <= 30 && z + depth <= 30,
                "place {index} is inside the edge"
            );
            let clear_of_well = x + width < 13 || x > 18 || z + depth < 13 || z > 18;
            assert!(clear_of_well, "place {index} is clear of the well");
        }
    }
}
