//! The settlements of base's overworld: hamlets of a few cottages and fields around a well, and
//! villages with an inn, a smithy, a chapel, market stalls, farms and a mill as well.

use weft_sdk::*;

use crate::settlement::{self, Settlement, Want};

static DRAW_WELL: Template = Template::named("draw_well");
static CROFTER_COTTAGE: Template = Template::named("crofter_cottage");
static LONGHOUSE: Template = Template::named("longhouse");
static DORMER_COTTAGE: Template = Template::named("dormer_cottage");
static WHEAT_FIELD: Template = Template::named("wheat_field");
static STADDLE_GRANARY: Template = Template::named("staddle_granary");
static GREAT_BARN: Template = Template::named("great_barn");
static HOF: Template = Template::named("hof");
static STABLE: Template = Template::named("stable");
static WAYSIDE_TAVERN: Template = Template::named("wayside_tavern");
static OPEN_FORGE: Template = Template::named("open_forge");
static SMITHY_HOUSE: Template = Template::named("smithy_house");
static WAYSIDE_CHAPEL: Template = Template::named("wayside_chapel");
static VILLAGE_CHURCH: Template = Template::named("village_church");
static MARKET_STALL: Template = Template::named("market_stall");
static MARKET_STALL_LARGE: Template = Template::named("market_stall_large");
static POST_MILL: Template = Template::named("post_mill");
static TOWER_MILL: Template = Template::named("tower_mill");
static LUMBERYARD: Template = Template::named("lumberyard");

static COTTAGES: &[&Template] = &[&CROFTER_COTTAGE, &LONGHOUSE, &DORMER_COTTAGE];

static HAMLET: Settlement = Settlement {
    region: 384,
    chance: 0.5,
    margin: 64,
    centre: &DRAW_WELL,
    wants: &[
        want(COTTAGES, 4, [8, 32], true),
        want(&[&STADDLE_GRANARY], 1, [10, 36], true),
        want(&[&WHEAT_FIELD], 2, [14, 40], false),
    ],
    roofs: &["spruce_roof", "spruce_roof", "oak_roof"],
    frames: &["", "spruce_frame"],
    infills: &[""],
};

static VILLAGE: Settlement = Settlement {
    region: 512,
    chance: 0.35,
    margin: 96,
    centre: &DRAW_WELL,
    wants: &[
        want(&[&MARKET_STALL, &MARKET_STALL_LARGE], 2, [6, 14], true),
        want(&[&WAYSIDE_TAVERN], 1, [10, 24], true),
        want(&[&OPEN_FORGE, &SMITHY_HOUSE], 1, [10, 28], true),
        want(&[&WAYSIDE_CHAPEL, &VILLAGE_CHURCH], 1, [14, 34], true),
        want(COTTAGES, 7, [12, 44], true),
        want(&[&GREAT_BARN, &HOF, &STABLE], 2, [28, 54], true),
        want(&[&STADDLE_GRANARY, &LUMBERYARD], 1, [24, 50], true),
        want(&[&WHEAT_FIELD], 4, [30, 60], false),
        want(&[&POST_MILL, &TOWER_MILL], 1, [40, 64], true),
    ],
    roofs: &["spruce_roof", "spruce_roof", "oak_roof", "brick_roof"],
    frames: &["", "spruce_frame", "birch_frame"],
    infills: &["", "white_infill"],
};

const fn want(
    templates: &'static [&'static Template],
    count: usize,
    [near, far]: [i32; 2],
    faces_centre: bool,
) -> Want {
    Want {
        templates,
        count,
        near,
        far,
        faces_centre,
    }
}

pub fn register(r: &mut Registry) {
    for (name, settlement) in [("hamlet", &HAMLET), ("village", &VILLAGE)] {
        structgen_api::plan(r, name, move |p| settlement::plan(p, settlement))
            .dimension(&base_api::OVERWORLD)
            .region(settlement.region);
    }
}
