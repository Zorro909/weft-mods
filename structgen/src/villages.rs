//! The settlements of base's overworld: hamlets of a few cottages round a well, villages of
//! lanes round a green with a church, a tavern and a smithy, and towns of cobbled streets round
//! a market square.

use weft_sdk::*;

use crate::settlement::{self, Faces, Kind, Settlement};

static DRAW_WELL: Template = Template::named("draw_well");
static CROFTER_COTTAGE: Template = Template::named("crofter_cottage");
static LONGHOUSE: Template = Template::named("longhouse");
static DORMER_COTTAGE: Template = Template::named("dormer_cottage");
static STADDLE_GRANARY: Template = Template::named("staddle_granary");
static GREAT_BARN: Template = Template::named("great_barn");
static HOF: Template = Template::named("hof");
static STABLE: Template = Template::named("stable");
static LUMBERYARD: Template = Template::named("lumberyard");
static WAYSIDE_TAVERN: Template = Template::named("wayside_tavern");
static SMITHY_HOUSE: Template = Template::named("smithy_house");
static OPEN_FORGE: Template = Template::named("open_forge");
static VILLAGE_CHURCH: Template = Template::named("village_church");
static MARKET_FOUNTAIN: Template = Template::named("market_fountain");
static MARKET_STALL: Template = Template::named("market_stall");
static MARKET_STALL_LARGE: Template = Template::named("market_stall_large");
static HALL_HOUSE: Template = Template::named("hall_house");
static CORNER_HOUSE: Template = Template::named("corner_house");
static NARROW_GABLE: Template = Template::named("narrow_gable");
static STOREHOUSE: Template = Template::named("storehouse");
static GUILDHALL: Template = Template::named("guildhall");
static TOWN_HALL: Template = Template::named("town_hall");
static TOWN_CHURCH: Template = Template::named("town_church");
static COACHING_INN: Template = Template::named("coaching_inn");
static POST_MILL: Template = Template::named("post_mill");
static TOWER_MILL: Template = Template::named("tower_mill");
static OAK_TREE: Template = Template::named("oak_tree");
static OAK_TREE_LARGE: Template = Template::named("oak_tree_large");
static BIRCH_TREE: Template = Template::named("birch_tree");
static FRUIT_TREE: Template = Template::named("fruit_tree");
static BUSH: Template = Template::named("bush");

static COTTAGES: &[&Template] = &[&CROFTER_COTTAGE, &LONGHOUSE, &DORMER_COTTAGE];
static FARMS: &[&Template] = &[&HOF, &GREAT_BARN, &STABLE, &STADDLE_GRANARY, &LUMBERYARD];
static TOWN_HOUSES: &[&Template] = &[
    &HALL_HOUSE,
    &CORNER_HOUSE,
    &NARROW_GABLE,
    &NARROW_GABLE,
    &STOREHOUSE,
];
static TREES: &[&Template] = &[
    &OAK_TREE_LARGE,
    &FRUIT_TREE,
    &OAK_TREE,
    &BIRCH_TREE,
    &BUSH,
    &BUSH,
];

static HAMLET: Settlement = Settlement {
    region: 384,
    chance: 0.45,
    priority: 1,
    kind: Kind::Village,
    steps: 3,
    branches: [0, 1],
    heart: [8.0, 6.0],
    setback: [2, 5],
    gap: 3,
    centre: &[&DRAW_WELL],
    landmarks: &[],
    homes: 5,
    houses: COTTAGES,
    farms: &[&HOF, &STADDLE_GRANARY],
    mill: None,
    fields: 1,
    trees: TREES,
    roofs: &["spruce_roof", "spruce_roof", "oak_roof"],
    frames: &["", "spruce_frame"],
    infills: &[""],
};

static VILLAGE: Settlement = Settlement {
    region: 512,
    chance: 0.45,
    priority: 2,
    kind: Kind::Village,
    steps: 5,
    branches: [1, 2],
    heart: [14.0, 10.0],
    setback: [2, 6],
    gap: 3,
    centre: &[&DRAW_WELL, &OAK_TREE],
    landmarks: &[
        (&VILLAGE_CHURCH, Faces::Heart),
        (&WAYSIDE_TAVERN, Faces::Heart),
        (&SMITHY_HOUSE, Faces::Road),
        (&OPEN_FORGE, Faces::Road),
    ],
    homes: 5,
    houses: COTTAGES,
    farms: FARMS,
    mill: Some(&POST_MILL),
    fields: 2,
    trees: TREES,
    roofs: &["spruce_roof", "spruce_roof", "oak_roof", "brick_roof"],
    frames: &["", "spruce_frame", "birch_frame"],
    infills: &["", "white_infill"],
};

static TOWN: Settlement = Settlement {
    region: 640,
    chance: 0.5,
    priority: 3,
    kind: Kind::Town,
    steps: 6,
    branches: [2, 3],
    heart: [15.0, 11.0],
    setback: [0, 1],
    gap: 1,
    centre: &[&MARKET_FOUNTAIN, &MARKET_STALL, &MARKET_STALL_LARGE],
    landmarks: &[
        (&TOWN_HALL, Faces::Heart),
        (&TOWN_CHURCH, Faces::Heart),
        (&GUILDHALL, Faces::Heart),
        (&COACHING_INN, Faces::Road),
        (&SMITHY_HOUSE, Faces::Road),
    ],
    homes: 12,
    houses: TOWN_HOUSES,
    farms: &[],
    mill: Some(&TOWER_MILL),
    fields: 2,
    trees: &[&OAK_TREE_LARGE, &FRUIT_TREE],
    roofs: &["brick_roof", "brick_roof", "spruce_roof"],
    frames: &["", "spruce_frame"],
    infills: &["", "white_infill"],
};

pub fn register(r: &mut Registry) {
    for (name, settlement) in [("hamlet", &HAMLET), ("village", &VILLAGE), ("town", &TOWN)] {
        structgen_api::plan(r, name, move |p| settlement::plan(p, settlement))
            .dimension(&base_api::OVERWORLD)
            .region(settlement.region)
            .priority(settlement.priority);
    }
}
