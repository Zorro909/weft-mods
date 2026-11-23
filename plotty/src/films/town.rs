//! The town the film founds and grows on camera, on a plateau of seed 42 some 150 blocks west
//! of where the world puts a player: one ground height, 75, from x -192 to -113 and z -80 to
//! -1. Each wave of builds is outlined first, as `/city build` shows it, then built through
//! Plotty's own `place::put`, so what stands is what a game master's commands make.

use showcase_api::Set;
use weft_sdk::{Template, warn};
use weftkit::{PlayerKey, Pos};

use crate::city::build::{self, Plan, Target, What};
use crate::city::grid::{CellXZ, Footprint};
use crate::city::structures::Kind;
use crate::city::{buildings, city_named, place};
use crate::players::PlayerEntry;
use crate::tables::{BALANCES, PLAYERS, PLOTS};
use crate::{dims, players, plot, settings};

/// The block layer the town is levelled to.
pub const GROUND: i32 = 75;
/// The town's name.
const NAME: &str = "Lindenfeld";
/// Ticks an outline stays: until a little after its build stands.
const OUTLINE_TICKS: u32 = 70;

/// Corner cells of what the film builds: the plaza, the roads out of it, the market square, the
/// normal plot west of the plaza and the small plots east of it.
pub const PLAZA: [i32; 2] = [-10, -3];
const ROADS: [[i32; 2]; 7] = [
    [-10, -4],
    [-10, -5],
    [-11, -3],
    [-12, -3],
    [-8, -3],
    [-8, -2],
    [-10, -1],
];
pub const MARKET: [i32; 2] = [-12, -5];
pub const NORMAL: [i32; 2] = [-12, -2];
const SMALL: [[i32; 2]; 5] = [[-9, -5], [-9, -4], [-8, -4], [-9, -1], [-8, -1]];
/// The camera's own small plot, where it builds its stall.
pub const YOURS: [i32; 2] = [-9, -1];

/// The structgen houses game masters put on plots, by corner cell.
const HOUSES: [([i32; 2], &str); 3] = [
    (NORMAL, "hall_house"),
    ([-9, -5], "crofter_cottage"),
    ([-8, -4], "narrow_gable"),
];

/// The other players of the town, by key and name, and the plot each owns.
pub const NEIGHBOURS: [(&str, &str, [i32; 2]); 3] = [
    ("film-mira", "Mira", NORMAL),
    ("film-bo", "Bo", [-9, -5]),
    ("film-ana", "Ana", [-8, -4]),
];

/// A wave of builds the film outlines and then builds.
#[derive(Clone, Copy)]
pub enum Wave {
    Plaza,
    Roads,
    Market,
    Plots,
}

impl Wave {
    /// What it builds, by corner cell.
    fn builds(self) -> Vec<(Kind, [i32; 2])> {
        match self {
            Wave::Plaza => vec![(Kind::Plaza, PLAZA)],
            Wave::Roads => ROADS.map(|corner| (Kind::Road, corner)).to_vec(),
            Wave::Market => vec![(Kind::Market, MARKET)],
            Wave::Plots => std::iter::once((Kind::Normal, NORMAL))
                .chain(SMALL.map(|corner| (Kind::Small, corner)))
                .collect(),
        }
    }

    /// Its plans, in the world of `set`.
    fn plans(self, set: &Set) -> Vec<Plan> {
        let dim = dims::key_of(&set.world());
        let target = match self {
            Wave::Plaza => Target::New(NAME.to_string()),
            _ => match city_named(NAME) {
                Some((city, _)) => Target::Existing(city),
                None => return Vec::new(),
            },
        };
        let plan = |(kind, [x, z]): (Kind, [i32; 2])| Plan {
            what: What::Kind(kind),
            target: target.clone(),
            dim,
            footprint: Footprint {
                corner: CellXZ::new(x, z),
                side: kind.side(),
            },
            ground_y: GROUND,
        };
        self.builds().into_iter().map(plan).collect()
    }
}

/// Shows the outlines of `wave`, as a preview does.
pub fn outline(set: &Set, wave: Wave) {
    for plan in wave.plans(set) {
        build::outline(&set.world(), set.player(), &plan, OUTLINE_TICKS);
    }
}

/// Builds `wave`, as `/city confirm` does.
pub fn build(set: &Set, wave: Wave) {
    for plan in wave.plans(set) {
        put(set, &plan);
    }
}

/// Puts the structgen houses on their plots.
pub fn houses(set: &Set) {
    let dim = dims::key_of(&set.world());
    let Some((city, _)) = city_named(NAME) else {
        return;
    };
    for (corner, name) in HOUSES {
        let Some((id, plot)) = plot_at(dim, corner) else {
            continue;
        };
        let side = plot::post::street_side(dim, plot.footprint);
        let fits = buildings::size_of(Template::generated(buildings::turned(name, side)))
            .is_some_and(|size| buildings::fits(size, plot.footprint.blocks()));
        if !fits {
            warn!("plotty film: {name} doesn't fit its plot");
            continue;
        }
        let plan = Plan {
            what: What::Building {
                template: name.to_string(),
                plot: id,
            },
            target: Target::Existing(city),
            dim,
            footprint: plot.footprint,
            ground_y: GROUND,
        };
        put(set, &plan);
    }
}

/// The neighbours and the camera claim their plots. Everyone has the starting Woof, as at a
/// first join: the camera too, whose balance a film played before in the same game folder
/// changed.
pub fn claims(set: &Set) {
    let world = set.world();
    let dim = dims::key_of(&world);
    for (key, name, corner) in NEIGHBOURS {
        let Some(key) = PlayerKey::new(key) else {
            continue;
        };
        let entry = PlayerEntry {
            name: name.to_string(),
            first_seen: players::now(),
            ..PlayerEntry::default()
        };
        PLAYERS.set(&key, entry);
        BALANCES.set(&key, settings::get().starting_woof);
        claim(dim, corner, key);
    }
    if let Some(you) = PlayerKey::of(&world, set.player()) {
        BALANCES.set(&you, settings::get().starting_woof);
        claim(dim, YOURS, you);
    }
}

/// Outlines the plots around the camera, as `/plot show` does there.
pub fn show_plots(set: &Set) {
    let world = set.world();
    let (Some(you), Some(camera)) = (
        PlayerKey::of(&world, set.player()),
        world.entity(set.player()),
    ) else {
        return;
    };
    let below = Pos::from(camera.block_pos());
    plot::show(
        &world,
        set.player(),
        &you,
        Pos::new(below.x, GROUND + 1, below.z),
    );
}

fn claim(dim: u64, corner: [i32; 2], owner: PlayerKey) {
    if let Some((id, mut plot)) = plot_at(dim, corner) {
        plot.owner = Some(owner);
        PLOTS.set(&id, plot);
    }
}

/// The plot whose corner cell is `corner`.
fn plot_at(dim: u64, [x, z]: [i32; 2]) -> Option<(u32, crate::city::Plot)> {
    plot::at(dim, CellXZ::new(x, z).corner(GROUND))
}

fn put(set: &Set, plan: &Plan) {
    if let Err((key, pos)) = place::put(&set.world(), plan) {
        warn!("plotty film: a build was refused: {key} {pos:?}");
    }
}
