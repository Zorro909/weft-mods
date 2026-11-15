//! `showcase:demo`: a short film of the steps that change how a film looks rather than where its
//! camera goes, in the meadow by the lake where seed 42 starts a player. It builds nothing; a
//! strip of the meadow is levelled for the walk.

use showcase_api::{Area, Ease, Film, Pos, Set, Weather, dolly, film, orbit};
use weft_sdk::Registry;

/// Height of a standing player's eyes over the ground, in blocks.
const EYES: f64 = 1.62;
/// Where the walk starts and ends: along the levelled strip, at a standing player's eyes.
const WALK: [[f64; 3]; 2] = [[0.5, EYES, -18.0], [0.5, EYES, 18.0]];
/// Blocks a second the walk goes at: a player's walking speed.
const WALKING: f64 = 4.3;

pub fn register(r: &mut Registry) {
    let strip = Area::between(Pos::new(-3, 0, -21), Pos::new(3, 6, 21));
    film(r, "demo", |_: &Set| demo()).level(strip);
}

fn demo() -> Film {
    let walk_secs = (WALK[1][2] - WALK[0][2]) / WALKING;
    Film::new()
        .weather(Weather::dusk())
        .title("showcase.film.dusk", 5.0)
        .shot(
            orbit([0.5, 1.0, 0.5], 16.0)
                .arc(200.0, 290.0)
                .height(7.0, 5.0)
                .secs(12.0),
        )
        .weather(Weather::mist())
        .title("showcase.film.mist", 5.0)
        .shot(
            dolly(WALK[0], WALK[1])
                .ease(Ease::Linear)
                .secs(walk_secs as f32),
        )
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
        for step in demo().steps {
            let Step::Title { key, .. } = step else {
                continue;
            };
            for lang in &langs {
                let line = format!("{key} = ");
                assert!(lang.lines().any(|l| l.starts_with(&line)), "{key} in each");
            }
        }
    }
}
