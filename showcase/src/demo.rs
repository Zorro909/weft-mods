//! `showcase:demo`: a short film of the steps that change how a film looks rather than where its
//! camera goes (weather, fades, a grade and a walking camera), in the meadow by the lake where
//! seed 42 starts a player. It builds nothing: the walk goes over the land as it is.

use showcase_api::{Ease, Film, Look, Pos, Set, Weather, film, flyby, orbit, screen};
use weft_sdk::{Registry, block_info};

/// Height of a standing player's eyes over the ground, in blocks.
const EYES: f64 = 1.62;
/// Where the walk goes along, from the origin: toward +z, from `WALK[0]` to `WALK[1]`.
const WALK: [i32; 2] = [-18, 18];
/// Blocks between the points of the walk, each at a standing player's eyes over the ground.
const PACE: usize = 3;
/// Blocks above and below the origin the ground is looked for.
const REACH: i32 = 12;
/// Blocks a second the walk goes at: a player's walking speed.
const WALKING: f64 = 4.3;
/// Blocks past the walk's end where it looks, so that it looks ahead, not at the slopes.
const LOOK_AHEAD: f64 = 40.0;
/// Seconds each fade between scenes takes.
const FADE: f32 = 1.5;
/// How strongly the dusk is graded warm, 0 to 1.
const WARMTH: f32 = 0.8;

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
    let end = walk.last().copied().unwrap_or_default();
    let ahead = [end[0], end[1], end[2] + LOOK_AHEAD];
    let walk_secs = (WALK[1] - WALK[0]) as f64 / WALKING;
    Film::new()
        .screen(screen::BLACK, 1.0, 0.0)
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
        .screen(screen::WARM, 0.0, 0.0)
        .weather(Weather::mist())
        .fade_in(FADE)
        .title("showcase.film.mist", 5.0)
        .shot(
            flyby(walk)
                .look(Look::At(ahead))
                .bob()
                .ease(Ease::Linear)
                .secs(walk_secs as f32),
        )
        .screen(screen::WHITE, 1.0, FADE)
        .wait(FADE)
}

/// The walk's points, at a standing player's eyes over the ground at `feet(z)`.
fn walk(feet: impl Fn(i32) -> i32) -> Vec<[f64; 3]> {
    (WALK[0]..=WALK[1])
        .step_by(PACE)
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
    fn the_walk_goes_at_eye_height_over_the_ground() {
        let points = walk(|z| if z < 0 { 0 } else { 2 });
        assert_eq!(points.first(), Some(&[0.5, EYES, -17.5]), "from its start");
        assert_eq!(
            points.last(),
            Some(&[0.5, 2.0 + EYES, 18.5]),
            "up the slope"
        );
    }
}
