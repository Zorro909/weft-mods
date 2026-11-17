//! The seismic thumper: the core drill's driven counterpart. A player starts a survey on its
//! screen; it takes down the survey markers beside it and surveys their rectangle, or the square
//! around it without them (`survey`). Its weight drops again and again while engine energy goes
//! into the survey; once that is paid for, it counts every ore below its area a little each tick,
//! by kind and band of 16 layers, then listens for oil fields within `oil::search` of it
//! (`work`). Its screen shows what it found until the next survey.
//!
//! It keeps the layer it counts loaded while it counts, and lets go once it is done or broken.
//!
//! Its shape is the Blockbench model `assets/models/block/seismic_thumper.json`, a steel frame
//! over a strike plate with a gearbox of plastic gears; the weight that drops is
//! `assets/models/seismic_thumper_weight.geo.json` (`weight`).

mod oil;
mod screen;
mod survey;
mod weight;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use survey::{BANDS, KINDS};
use weft_sdk::{
    Block, Channel, Component, ENERGY, Facings, Menu, Packet, Registry, Toward, component_flags,
    tool_tier, value,
};

use crate::ghost::ItemKey;
use crate::lib::{Area, Pos};

pub static SEISMIC_THUMPER: Block = Block::named("seismic_thumper");
static THUMPER_MENU: Menu = Menu::named("seismic_thumper");

/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 1_000;
const STORES: i64 = 10_000;
/// Oil fields its screen lists, the nearest first.
pub const FIELDS: usize = 4;

value! {
    /// A thumper's survey: its `weight` (an entity), the energy `spent` on it, its `status`, the
    /// area it surveys (`lo`, `hi`, at the height of the thumper) and the `layers` it counts
    /// below, the blocks `counted` so far and the ticks it `waited` for one to load, and the
    /// squares of the oil grid it `listened` to.
    ///
    /// Of each kind of ore it found, the key of its `ores/<m>` name (`kinds`), the key of the name
    /// of the first block of it found (`shown`) and its `counts` by band, kind after kind. Of each
    /// oil field it found, `fields` of them, where its seep lies from the thumper (`field_at`, x
    /// then z) and its oil's highest and lowest layer (`field_depth`, `i32::MIN` when unknown).
    /// The game time it last `worked_at`, so it works once a tick (`machine::first_call_at`).
    pub struct Survey {
        pub weight: u64,
        pub spent: i64,
        pub status: u32,
        pub layers: u32,
        pub lo: Pos,
        pub hi: Pos,
        pub counted: u32,
        pub waited: u32,
        pub listened: u32,
        pub fields: u32,
        pub kinds: [u64; KINDS],
        pub shown: [ItemKey; KINDS],
        pub counts: [u32; KINDS * BANDS],
        pub field_at: [i32; FIELDS * 2],
        pub field_depth: [i32; FIELDS * 2],
        pub worked_at: u64,
    }
}
static SURVEY: Component<Survey> = Component::named("cogwork:survey");

impl Default for Survey {
    fn default() -> Survey {
        Survey {
            weight: 0,
            spent: 0,
            status: 0,
            layers: 0,
            lo: Pos::default(),
            hi: Pos::default(),
            counted: 0,
            waited: 0,
            listened: 0,
            fields: 0,
            kinds: [0; KINDS],
            shown: [0; KINDS],
            counts: [0; KINDS * BANDS],
            field_at: [0; FIELDS * 2],
            field_depth: [0; FIELDS * 2],
            worked_at: 0,
        }
    }
}

/// What the thumper is doing; its screen says `cogwork.seismic_thumper.<name>`. Kept in
/// `Survey::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// No survey yet.
    #[default]
    Idle,
    Thumping,
    NoPower,
    Counting,
    Listening,
    Done,
}

impl Status {
    /// Whether a survey is under way, which a new one waits for.
    fn surveying(self) -> bool {
        matches!(
            self,
            Status::Thumping | Status::NoPower | Status::Counting | Status::Listening
        )
    }
}

impl Survey {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }

    /// The columns it surveys, at its height.
    pub fn area(&self) -> Area {
        Area::between(self.lo, self.hi)
    }

    /// Starts a new survey of `area`'s columns, `layers` deep, forgetting the last one but its
    /// weight.
    pub fn set_area(&mut self, area: Area, layers: u32) {
        *self = Survey {
            weight: self.weight,
            lo: area.lo,
            hi: area.hi,
            layers,
            ..Survey::default()
        };
    }
}

value! {
    /// The thumper `owner` is to start a survey.
    pub struct Start {
        pub owner: u64,
    }
}
static START: Channel<Start> = Channel::named("thumper_start");

pub fn register(r: &mut Registry) {
    r.entity_component(
        &SURVEY,
        Survey::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&SURVEY, work::tick);
    r.packet(&START, start);
    r.block(&SEISMIC_THUMPER)
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("seismic_thumper")
        .hitbox([0.0; 3], [16.0; 3])
        .hardness(5.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .on_break(crate::loading::let_go)
        .entity(|entity| {
            entity
                .attach(&SURVEY)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
        })
        .opens(&THUMPER_MENU)
        .tab(&crate::TAB);
    r.menu(&THUMPER_MENU).screen(screen::screen);
    weight::register(r);
}

/// Server side: starts a survey on the thumper asked for, when the player is at it and it is not
/// surveying already.
fn start(packet: &Packet, start: Start) {
    let world = packet.world();
    let (Some(block), Some(player)) = (world.entity(start.owner), world.entity(packet.player()))
    else {
        return;
    };
    if !crate::ghost::within_reach(block.pos, player.pos) {
        return;
    }
    let pos = Pos::from(block.block_pos());
    let Some(survey) = world.component_mut(start.owner, &SURVEY) else {
        return;
    };
    if survey.status().surveying() {
        return;
    }
    let marked = crate::markers::take_area(&world, pos);
    let area = survey::columns(marked, pos);
    survey.set_area(area, survey::layers(pos.y));
    survey.set_status(Status::Thumping);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.seismic_thumper.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_thumper_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Survey, weight), 0, "its weight");
        assert_eq!(offset_of!(Survey, spent), 8, "the energy spent");
        assert_eq!(offset_of!(Survey, status), 16, "its status");
        assert_eq!(offset_of!(Survey, layers), 20, "its layers");
        assert_eq!(offset_of!(Survey, lo), 24, "its area");
        assert_eq!(offset_of!(Survey, hi), 36, "to its far corner");
        assert_eq!(offset_of!(Survey, counted), 48, "what it counted");
        assert_eq!(offset_of!(Survey, waited), 52, "how long it waited");
        assert_eq!(offset_of!(Survey, listened), 56, "the oil squares");
        assert_eq!(offset_of!(Survey, fields), 60, "the fields found");
        assert_eq!(offset_of!(Survey, kinds), 64, "the kinds of ore");
        assert_eq!(offset_of!(Survey, shown), 128, "their names");
        assert_eq!(offset_of!(Survey, counts), 192, "their counts");
        assert_eq!(offset_of!(Survey, field_at), 512, "where the fields are");
        assert_eq!(offset_of!(Survey, field_depth), 544, "how deep");
        assert_eq!(offset_of!(Survey, worked_at), 576, "when it last worked");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_idle() {
        let survey = Survey {
            status: 99,
            ..Survey::default()
        };
        assert_eq!(survey.status(), Status::Idle, "from a newer cogwork");
    }

    #[test]
    fn a_new_survey_starts_over_but_keeps_its_weight() {
        let mut survey = Survey {
            weight: 7,
            counted: 900,
            fields: 2,
            ..Survey::default()
        };
        survey.tally(3, 3, 0);
        let area = Area::between(Pos::new(0, 64, 0), Pos::new(9, 64, 9));
        survey.set_area(area, 96);
        assert_eq!((survey.weight, survey.counted, survey.fields), (7, 0, 0));
        assert_eq!(survey.kinds, [0; KINDS], "no ore from the last one");
        assert_eq!((survey.area(), survey.layers), (area, 96), "its new area");
    }

    #[test]
    fn a_survey_under_way_is_not_started_again() {
        assert!(!Status::Idle.surveying() && !Status::Done.surveying());
        assert!(Status::NoPower.surveying() && Status::Listening.surveying());
    }

    #[test]
    fn the_thumper_has_its_models() {
        for file in [
            "models/block/seismic_thumper.json",
            "models/item/seismic_thumper.json",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
