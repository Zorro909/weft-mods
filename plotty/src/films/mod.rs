//! Plotty's film for the showcase mod, built only with the `showcase` feature:
//! `tools/showcase.sh --with structgen plotty:showcase -- --view 32`. A town founded and grown on
//! camera on a plateau of seed 42 (`town`): its plaza, streets, market square and plots, each
//! outlined first, then houses from structgen and the neighbours' stalls on market places they
//! rent (`stalls`). The camera comes down to Mira's stall and buys bread for Golden Woof, goes on
//! over the plaza while the plots are claimed and outlined, and puts together and sets up a stall
//! on its own plot. Places are in world coordinates, so the film is for that seed and its land as
//! it now generates.

mod stalls;
mod town;

use showcase_api::{Ease, Film, Look, Path, Pos, Shot, Step, dolly, film, flyby, orbit, screen};
use weft_sdk::Registry;

use self::stalls::{MIRAS_REGISTER, YOUR_REGISTER};
use self::town::Wave;
use crate::market::pattern::Part;

/// What the camera looks at: the middle of the town and of its market; the camera's stall and
/// Mira's.
const TOWN: [f64; 3] = [-152.0, 76.0, -40.0];
const MARKET: [f64; 3] = [-176.0, 76.0, -64.0];
const YOUR_STALL: [f64; 3] = above(YOUR_REGISTER);
const MIRAS_STALL: [f64; 3] = above(MIRAS_REGISTER);

pub fn register(r: &mut Registry) {
    film(r, "showcase", |set| from(set.origin(), showcase()))
        .on_cue("outline_plaza", |set| town::outline(set, Wave::Plaza))
        .on_cue("plaza", |set| town::build(set, Wave::Plaza))
        .on_cue("outline_roads", |set| town::outline(set, Wave::Roads))
        .on_cue("roads", |set| town::build(set, Wave::Roads))
        .on_cue("outline_market", |set| town::outline(set, Wave::Market))
        .on_cue("market", |set| town::build(set, Wave::Market))
        .on_cue("outline_plots", |set| town::outline(set, Wave::Plots))
        .on_cue("plots", |set| town::build(set, Wave::Plots))
        .on_cue("houses", |set| {
            town::houses(set);
            stalls::market(set);
        })
        .on_cue("settle", |set| {
            town::claims(set);
            stalls::stock_market(set);
        })
        .on_cue("buy", stalls::buy)
        .on_cue("show_plots", town::show_plots)
        .on_cue("crates", |set| stalls::assemble(set, &[Part::Crate]))
        .on_cue("counters", |set| stalls::assemble(set, &[Part::Counter]))
        .on_cue("posts", |set| stalls::assemble(set, &[Part::Post]))
        .on_cue("awning", |set| stalls::assemble(set, &[Part::Awning]))
        .on_cue("register", |set| stalls::assemble(set, &[Part::Register]))
        .on_cue("stock", stalls::stock_yours);
}

/// The town grows, the camera shops at the market, goes on to the plots, and builds a stall
/// of its own: always on south and east, so it never goes back the way it came.
fn showcase() -> Film {
    let film = Film::new().screen(screen::BLACK, 1.0, 0.0);
    let film = opening(film.fade_in(3.0));
    let film = founding(film);
    let film = to_market(film);
    let film = trading(film);
    let film = plots(film);
    let film = your_stall(film);
    closing(film)
}

/// The town's orbit: how far out, how high, from where to where (east by south, round the
/// south and west, to north by west), and how fast.
const TOWN_OUT: f64 = 58.0;
const TOWN_UP: [f64; 2] = [38.0, 30.0];
const TOWN_ARC: [f64; 2] = [30.0, 265.0];
const TOWN_SPEED: f32 = 9.0;

/// In from the east over the meadows to the empty plateau.
fn opening(film: Film) -> Film {
    let path = [
        [-30.0, 120.0, 40.0],
        [-65.0, 117.0, 15.0],
        on_orbit(TOWN, TOWN_OUT, TOWN_UP[0], TOWN_ARC[0]),
    ];
    film.title("plotty.film.title", 5.0).shot(
        flyby(&path)
            .look(Look::Pan([-110.0, 76.0, -20.0], TOWN))
            .ease(speeds(10.0, TOWN_SPEED))
            .secs(9.0),
    )
}

/// Round the plateau while the town goes up a wave at a time, each outlined first; houses and
/// the neighbours' stalls last.
fn founding(film: Film) -> Film {
    let waves = [
        ("outline_plaza", "plaza", 57.0, 93.0, "plotty.film.found"),
        ("outline_roads", "roads", 111.0, 129.0, "plotty.film.grow"),
        ("outline_market", "market", 143.0, 161.0, ""),
        ("outline_plots", "plots", 175.0, 218.0, "plotty.film.undo"),
    ];
    let mut film = film;
    let mut from = TOWN_ARC[0];
    for (outline, build, built_at, next, title) in waves {
        film = film.cue(outline);
        if !title.is_empty() {
            film = film.title(title, 4.5);
        }
        film = film
            .shot(round_town(from, built_at))
            .cue(build)
            .shot(round_town(built_at, next));
        from = next;
    }
    film.cue("houses")
        .title("plotty.film.houses", 5.0)
        .shot(round_town(from, TOWN_ARC[1]))
        .cue("settle")
}

/// Part of the town's orbit, sinking from its first height to its last along the whole arc.
fn round_town(from: f64, to: f64) -> Shot {
    let up = |angle: f64| {
        let part = (angle - TOWN_ARC[0]) / (TOWN_ARC[1] - TOWN_ARC[0]);
        TOWN_UP[0] + (TOWN_UP[1] - TOWN_UP[0]) * part
    };
    let secs = ((to - from).abs().to_radians() * TOWN_OUT / f64::from(TOWN_SPEED)) as f32;
    orbit(TOWN, TOWN_OUT)
        .arc(from, to)
        .height(up(from), up(to))
        .ease(speeds(TOWN_SPEED, TOWN_SPEED))
        .secs(secs)
}

/// Where the camera stands before Mira's stall, north of it in the market, near enough to
/// trade, and where it has drifted to when it leaves.
const BY_MIRAS_STALL: [f64; 3] = [-186.5, 78.6, -59.0];
const LEAVING_MIRA: [f64; 3] = [-186.3, 78.5, -58.2];

/// South off the orbit and down into the market square, to Mira's stall.
fn to_market(film: Film) -> Film {
    let path = [
        on_orbit(TOWN, TOWN_OUT, TOWN_UP[1], TOWN_ARC[1]),
        [-180.0, 98.0, -84.0],
        [-185.0, 87.0, -70.0],
        BY_MIRAS_STALL,
    ];
    film.title("plotty.film.market", 6.0).shot(
        flyby(&path)
            .look(Look::Pan(MARKET, MIRAS_STALL))
            .ease(speeds(TOWN_SPEED, 0.1))
            .secs(10.0),
    )
}

/// The shop of Mira's stall, a trade, and the camera's Golden Woof beside its inventory.
fn trading(film: Film) -> Film {
    film.title("plotty.film.trade", 6.0)
        .menu("plotty:stall_shop", MIRAS_REGISTER, 8.0)
        .shot(
            dolly(BY_MIRAS_STALL, LEAVING_MIRA)
                .look_at(MIRAS_STALL)
                .ease(speeds(0.1, 0.1))
                .secs(3.5),
        )
        .cue("buy")
        .wait(4.5)
        .title("plotty.film.woof", 5.0)
        .menu("base:inventory", MIRAS_REGISTER, 5.0)
        .wait(5.5)
}

/// What the camera looks at over the plots east of the plaza, between their two rows; where it
/// looks across them from over the plaza; and where it waits beside its own stall, north of it
/// on the plaza's edge.
const PLOTS: [f64; 3] = [-122.0, 76.0, -24.0];
const OVER_PLOTS: [[f64; 3]; 2] = [[-152.0, 99.0, -30.0], [-149.0, 97.0, -29.0]];
const BY_YOUR_STALL: [f64; 3] = [-136.5, 78.6, -16.5];

/// Up out of the market and east over the plaza, looking out over the plots, which the
/// camera's player outlines.
fn plots(film: Film) -> Film {
    let across = [
        LEAVING_MIRA,
        [-185.5, 86.0, -54.0],
        [-168.0, 95.0, -37.0],
        OVER_PLOTS[0],
    ];
    film.title("plotty.film.claim", 6.0)
        .shot(
            flyby(&across)
                .look(Look::Pan(MIRAS_STALL, PLOTS))
                .ease(speeds(0.1, 0.6))
                .secs(10.0),
        )
        .cue("show_plots")
        .title("plotty.film.show", 5.0)
        .shot(
            dolly(OVER_PLOTS[0], OVER_PLOTS[1])
                .look_at(PLOTS)
                .ease(speeds(0.6, 0.6))
                .secs(5.0),
        )
}

/// Down to the camera's own plot, where its stall is put together and set up.
fn your_stall(film: Film) -> Film {
    let down = [OVER_PLOTS[1], [-142.0, 88.0, -23.0], BY_YOUR_STALL];
    let near = [
        BY_YOUR_STALL[0],
        BY_YOUR_STALL[1] - 0.2,
        BY_YOUR_STALL[2] + 0.8,
    ];
    let parts = ["crates", "counters", "posts", "awning", "register"];
    let film = film.shot(
        flyby(&down)
            .look(Look::Pan(PLOTS, YOUR_STALL))
            .ease(speeds(0.6, 0.1))
            .secs(6.0),
    );
    let film = parts
        .into_iter()
        .fold(film.title("plotty.film.stall", 5.0), |film, part| {
            film.cue(part).wait(0.9)
        });
    film.wait(0.5)
        .cue("stock")
        .wait(0.6)
        .title("plotty.film.setup", 6.0)
        .menu("plotty:stall_setup", YOUR_REGISTER, 6.5)
        .shot(
            dolly(BY_YOUR_STALL, near)
                .look_at(YOUR_STALL)
                .ease(speeds(0.1, 0.1))
                .secs(7.0),
        )
}

/// Up and out round the stall, by its east to its south-east, where the town rises behind it.
fn closing(film: Film) -> Film {
    let [dx, dz] = [
        BY_YOUR_STALL[0] + 0.0 - YOUR_STALL[0],
        BY_YOUR_STALL[2] + 0.8 - YOUR_STALL[2],
    ];
    let out = dx.hypot(dz);
    let from = dz.atan2(dx).to_degrees();
    let low = BY_YOUR_STALL[1] - 0.2 - YOUR_STALL[1];
    film.title("plotty.film.end", 6.0)
        .shot(
            orbit(YOUR_STALL, out)
                .arc(from, from + 135.0)
                .radius(out, 42.0)
                .height(low, 30.0)
                .ease(Ease::In)
                .secs(11.0),
        )
        .fade_out(2.0)
}

/// The middle of `block`, half a block higher: where the camera looks to have it in view.
const fn above(block: weftkit::Pos) -> [f64; 3] {
    [
        block.x as f64 + 0.5,
        block.y as f64 + 1.0,
        block.z as f64 + 0.5,
    ]
}

/// From `from` blocks a second to `to`: a shot going on from the one before at the speed that
/// one ended at, so the camera doesn't stop between them.
fn speeds(from: f32, to: f32) -> Ease {
    Ease::Speeds { from, to }
}

/// Where an orbit round `center` is at `angle` (0 toward +x, 90 toward +z), `radius` out and
/// `height` up: the shots before and after an orbit meet it there.
fn on_orbit(center: [f64; 3], radius: f64, height: f64, angle: f64) -> [f64; 3] {
    let (sin, cos) = angle.to_radians().sin_cos();
    [
        center[0] + cos * radius,
        center[1] + height,
        center[2] + sin * radius,
    ]
}

/// `film`, written in world coordinates, with every place (shots and menus) counted from
/// `origin` instead: the block the film starts at, from which the showcase counts. As
/// structgen's film does it.
fn from(origin: Pos, film: Film) -> Film {
    let o = [origin.x, origin.y, origin.z].map(f64::from);
    let at = |p: [f64; 3]| [p[0] - o[0], p[1] - o[1], p[2] - o[2]];
    let steps = film.steps.into_iter().map(|step| match step {
        Step::Shot(mut shot) => {
            shot.path = match shot.path {
                Path::Still(p) => Path::Still(at(p)),
                Path::Line(a, b) => Path::Line(at(a), at(b)),
                Path::Spline(points) => Path::Spline(points.into_iter().map(at).collect()),
                Path::Orbit {
                    center,
                    radius,
                    height,
                    angle,
                } => Path::Orbit {
                    center: at(center),
                    radius,
                    height,
                    angle,
                },
                follow => follow,
            };
            shot.look = match shot.look {
                Look::At(p) => Look::At(at(p)),
                Look::Pan(a, b) => Look::Pan(at(a), at(b)),
                look => look,
            };
            Step::Shot(shot)
        }
        Step::Menu { menu, at, secs } => Step::Menu {
            menu,
            at: Pos::new(at.x - origin.x, at.y - origin.y, at.z - origin.z),
            secs,
        },
        step => step,
    });
    Film {
        steps: steps.collect(),
    }
}
