//! `showcase:demo`: a short film of the steps that change how a film looks rather than where its
//! camera goes, in the meadow by the lake where seed 42 starts a player. Dusk with a warm grade;
//! a walk toward the lake as mist rolls in and the grade turns cool, in step; a flight down to
//! the lake as both lift; and a dive under its water, the screen as a player sees it there. It
//! builds nothing: the walk goes over the land as it is.

use showcase_api::{Ease, Film, Look, Pos, Set, Weather, film, flyby, orbit, screen};
use weft_sdk::{Registry, block_info};

/// Height of a standing player's eyes over the ground, in blocks.
const EYES: f64 = 1.62;
/// Where the walk goes along, from the origin: toward -z, the lake, from `WALK[0]` to `WALK[1]`.
const WALK: [i32; 2] = [30, -18];
/// Blocks between the points of the walk, each at a standing player's eyes over the ground.
const PACE: i32 = 3;
/// Points of the walk before the mist comes: dusk is seen first.
const DUSK_POINTS: usize = 4;
/// Blocks above and below the origin the ground is looked for.
const REACH: i32 = 12;
/// Blocks a second the walk goes at: a player's walking speed.
const WALKING: f64 = 4.3;
/// Blocks past the walk's end where it looks, so that it looks ahead, not at the slopes.
const LOOK_AHEAD: f64 = 40.0;
/// Where the flight from the walk's end goes down the meadow to the lake, from the origin,
/// ending over the water (its top is 33 blocks below the origin).
const FLIGHT: [[f64; 3]; 4] = [
    [2.0, -2.0, -45.0],
    [10.0, -13.0, -75.0],
    [25.0, -27.0, -105.0],
    [35.8, -31.5, -126.2],
];
/// Seconds the flight takes.
const FLIGHT_SECS: f32 = 11.0;
/// Where the camera goes on under the water, on the flight's heading: 3 blocks below the top,
/// over a bottom 6 blocks down.
const DIVE: [[f64; 3]; 2] = [[43.9, -36.0, -142.1], [52.0, -36.5, -158.0]];
/// Seconds the dive takes.
const DIVE_SECS: f32 = 6.0;
/// Blocks a second the flight and the dive meet at.
const DIVING: f32 = 8.0;
/// Seconds each fade between scenes takes.
const FADE: f32 = 1.5;
/// Seconds the mist and the cool grade take to come in, and to go.
const CHANGE: f32 = 4.0;
/// How strongly the dusk is graded warm, 0 to 1.
const WARMTH: f32 = 0.8;
/// How strongly the mist is graded cool, 0 to 1.
const COOLNESS: f32 = 0.7;

pub fn register(r: &mut Registry) {
    film(r, "demo", demo);
}

fn demo(set: &Set) -> Film {
    let world = set.world();
    let ground = |pos: Pos| {
        let state = world.block(set.at(pos));
        block_info(state).is_some_and(|info| info.solid() || info.liquid.is_some())
    };
    film_over(&walk(|z| feet(|y| ground(Pos::new(0, y, z)))))
}

/// The film, its walk through `walk`.
fn film_over(walk: &[[f64; 3]]) -> Film {
    let film = dusk(Film::new());
    let film = mist(film, walk);
    let film = clearing(film, walk.last().copied().unwrap_or_default());
    under_water(film)
}

/// Opens on black onto dusk, graded warm, circling the meadow; then fades to black.
fn dusk(film: Film) -> Film {
    film.screen(screen::BLACK, 1.0, 0.0)
        // At once, behind the black: the film opens on dusk, not on day turning to it.
        .weather(Weather::dusk().sunlight(0.3).fade(0.0))
        .screen(screen::WARM, WARMTH, 0.0)
        .fade_in(FADE)
        .title("showcase.film.dusk", 5.0)
        .shot(
            orbit([0.5, 1.0, 0.5], 16.0)
                .arc(200.0, 290.0)
                .height(7.0, 5.0)
                .secs(12.0),
        )
        .fade_out(FADE)
}

/// Walks toward the lake, still at dusk, as mist rolls in and the grade turns cool together.
fn mist(film: Film, walk: &[[f64; 3]]) -> Film {
    let (at_dusk, in_mist) = walk.split_at(DUSK_POINTS.min(walk.len()));
    let end = walk.last().copied().unwrap_or_default();
    let ahead = [end[0], end[1], end[2] - LOOK_AHEAD];
    // The mist's walk starts where the dusk's ends.
    let in_mist = [&at_dusk[at_dusk.len().saturating_sub(1)..], in_mist].concat();
    let stretch = |points: &[[f64; 3]]| {
        let blocks = (points.len().saturating_sub(1) as i32 * PACE) as f64;
        flyby(points)
            .look(Look::At(ahead))
            .bob()
            .ease(Ease::Linear)
            .secs((blocks / WALKING) as f32)
    };
    film.fade_in(FADE)
        .shot(stretch(at_dusk))
        .weather(Weather::mist().fade(CHANGE))
        .screen(screen::WARM, 0.0, CHANGE)
        .screen(screen::COOL, COOLNESS, CHANGE)
        .title("showcase.film.mist", 5.0)
        .shot(stretch(&in_mist))
}

/// Flies on from the walk's end, `from`, down the meadow to the lake as the mist and the grade
/// lift together.
fn clearing(film: Film, from: [f64; 3]) -> Film {
    let points = [&[from][..], &FLIGHT].concat();
    film.clear_weather()
        .screen(screen::COOL, 0.0, CHANGE)
        .title("showcase.film.clearing", 5.0)
        .shot(
            flyby(&points)
                .blend(1.0)
                .ease(Ease::Speeds {
                    from: WALKING as f32,
                    to: DIVING,
                })
                .secs(FLIGHT_SECS),
        )
}

/// Dives under the lake's water, the screen as a player sees it there, and fades to white.
fn under_water(film: Film) -> Film {
    let points = [&FLIGHT[FLIGHT.len() - 1..], &DIVE].concat();
    film.title("showcase.film.under_water", 5.0)
        .shot(
            flyby(&points)
                .ease(Ease::Speeds {
                    from: DIVING,
                    to: 0.0,
                })
                .secs(DIVE_SECS),
        )
        .screen(screen::WHITE, 1.0, FADE)
        .wait(FADE)
}

/// The walk's points, at a standing player's eyes over the ground at `feet(z)`.
fn walk(feet: impl Fn(i32) -> i32) -> Vec<[f64; 3]> {
    let steps = (WALK[0] - WALK[1]) / PACE;
    (0..=steps)
        .map(|step| WALK[0] - step * PACE)
        .map(|z| [0.5, feet(z) as f64 + EYES, z as f64 + 0.5])
        .collect()
}

/// Where a player standing in a column stands, from the origin: on the highest block for which
/// `ground(y)` holds within `REACH`, or at the origin where none does.
fn feet(ground: impl Fn(i32) -> bool) -> i32 {
    (-REACH..=REACH)
        .rev()
        .find(|&y| ground(y))
        .map_or(0, |y| y + 1)
}

#[cfg(test)]
mod tests {
    use showcase_api::Step;

    use super::*;

    #[test]
    fn every_title_is_in_every_lang_file() {
        let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/lang");
        let files = std::fs::read_dir(folder).expect("showcase has an assets/lang folder");
        let langs: Vec<String> = files
            .map(|file| std::fs::read_to_string(file.expect("a lang file").path()))
            .collect::<Result<_, _>>()
            .expect("lang files read");
        for step in film_over(&walk(|_| 0)).steps {
            let Step::Title { key, .. } = step else {
                continue;
            };
            for lang in &langs {
                let line = format!("{key} = ");
                assert!(lang.lines().any(|l| l.starts_with(&line)), "{key} in each");
            }
        }
    }

    #[test]
    fn a_player_stands_on_the_highest_ground() {
        assert_eq!(
            feet(|y| y <= 3 || y == 7),
            8,
            "on the tree, as a player would be"
        );
        assert_eq!(feet(|y| y <= -2), -1, "down in a hollow");
        assert_eq!(feet(|_| false), 0, "at the origin over nothing");
    }

    #[test]
    fn the_walk_goes_toward_the_lake_at_eye_height_over_the_ground() {
        let points = walk(|z| if z > 0 { 2 } else { 0 });
        assert_eq!(
            points.first(),
            Some(&[0.5, 2.0 + EYES, 30.5]),
            "from its start"
        );
        assert_eq!(points.last(), Some(&[0.5, EYES, -17.5]), "down the slope");
        assert_eq!(points.len(), 17, "a point every 3 blocks");
    }

    #[test]
    fn the_mist_walk_goes_on_from_where_the_dusk_walk_ends() {
        let film = mist(Film::new(), &walk(|_| 0));
        let shots: Vec<_> = (film.steps.iter())
            .filter_map(|step| match step {
                Step::Shot(shot) => Some(shot.secs),
                _ => None,
            })
            .collect();
        let whole = (WALK[0] - WALK[1]) as f64 / WALKING;
        assert!(
            ((shots[0] + shots[1]) as f64 - whole).abs() < 1e-4,
            "{shots:?}"
        );
    }
}
