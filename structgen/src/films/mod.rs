//! Structgen's film for the showcase mod, built only with the `showcase` feature:
//! `tools/showcase.sh structgen:showcase`. A journey through seed 42's world as it generates,
//! nothing built or levelled for it: a hamlet, a village, a town and, over the mountains, a last
//! hamlet, flying north from one to the next and into some of their buildings. The places are where seed 42 puts them, written in world coordinates, so
//! the film is for that seed and the settlements as they are now laid out.

mod building;
mod flights;
mod hamlets;
mod town;
mod village;

use showcase_api::{Ease, Film, Look, Path, Pos, Shot, Step, film, flyby, hold};
use weft_sdk::*;

/// Mid-morning: the film plays into the early afternoon. The showcase sets noon when it is
/// asked for, but loading the world along the whole journey takes a minute or two first.
const MORNING: u64 = 4500;

pub fn register(r: &mut Registry) {
    film(r, "showcase", |set| from(set.origin(), journey()))
        .on_cue("morning", |set| set.world().set_day_time(MORNING));
}

fn journey() -> Film {
    let film = hamlets::first(Film::new().cue("morning"));
    let film = flights::to_the_village(film);
    let film = village::village(film);
    let film = flights::to_the_town(film);
    let film = town::town(film);
    let film = flights::over_the_mountains(film);
    hamlets::last(film)
}

/// A flight through the first of each pair, looking at the second there, turning smoothly
/// between them. A shot that starts with the pair the one before ended on goes on from it
/// without a jump.
fn glide(path: &[([f64; 3], [f64; 3])], secs: f32) -> Shot {
    let points: Vec<[f64; 3]> = path.iter().map(|&(eye, _)| eye).collect();
    let angles = path.iter().map(|&(eye, at)| toward(eye, at)).collect();
    flyby(&points).look(Look::Along(angles)).secs(secs)
}

/// From `from` blocks a second to `to`: a shot going on from the one before at the speed that
/// one ended at, so the camera doesn't stop between them.
fn speeds(from: f32, to: f32) -> Ease {
    Ease::Speeds { from, to }
}

/// Turns on the spot at `eye`, from looking at the first of `points` (where the shot before
/// left off) past the others in turn to the last, so it turns the way meant. It turns at an
/// even pace, so it goes on from the glide before and into the one after without a pause.
fn turn(eye: [f64; 3], points: &[[f64; 3]], secs: f32) -> Shot {
    let angles = points.iter().map(|&at| toward(eye, at)).collect();
    hold(eye)
        .look(Look::Along(angles))
        .secs(secs)
        .ease(Ease::Linear)
}

/// Where an orbit round `center` is at `angle` (0 toward +x, 90 toward +z), `radius` out and
/// `height` up: the shots before and after an orbit meet it there.
fn on_orbit(center: [f64; 3], radius: f64, height: f64, angle: f64) -> [f64; 3] {
    let a = angle.to_radians();
    [
        center[0] + a.cos() * radius,
        center[1] + height,
        center[2] + a.sin() * radius,
    ]
}

/// The yaw and pitch that look from `eye` at `at`, as the game counts them: yaw 0 looks north
/// (-z) and 90 west (-x), pitch up is positive.
fn toward(eye: [f64; 3], at: [f64; 3]) -> [f32; 2] {
    let [dx, dy, dz] = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
    let yaw = (-dx).atan2(-dz).to_degrees();
    let pitch = dy.atan2(dx.hypot(dz)).to_degrees();
    [yaw as f32, pitch as f32]
}

/// `film`, written in world coordinates, with every place counted from `origin` instead: the
/// block the film starts at, from which the showcase counts.
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
        step => step,
    });
    Film {
        steps: steps.collect(),
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{dolly, orbit};

    use super::*;

    #[test]
    fn places_are_counted_from_where_the_film_starts() {
        let film = Film::new()
            .shot(dolly([10.0, 70.0, -5.0], [20.0, 70.0, -5.0]).look_at([0.0, 64.0, 0.0]))
            .shot(orbit([1.0, 2.0, 3.0], 5.0));
        let moved = from(Pos::new(10, 64, -10), film);
        let Step::Shot(dolly) = &moved.steps[0] else {
            panic!("a shot")
        };
        assert_eq!(
            dolly.path,
            Path::Line([0.0, 6.0, 5.0], [10.0, 6.0, 5.0]),
            "the path"
        );
        assert_eq!(dolly.look, Look::At([-10.0, 0.0, 10.0]), "where it looks");
        let Step::Shot(orbit) = &moved.steps[1] else {
            panic!("a shot")
        };
        let Path::Orbit { center, radius, .. } = orbit.path else {
            panic!("an orbit")
        };
        assert_eq!(
            (center, radius),
            ([-9.0, -62.0, 13.0], [5.0; 2]),
            "only the center moves"
        );
    }

    #[test]
    fn looks_are_counted_as_the_game_does() {
        let eye = [10.0, 64.0, 10.0];
        assert_eq!(toward(eye, [10.0, 64.0, 0.0]), [0.0, 0.0], "north");
        assert_eq!(toward(eye, [0.0, 64.0, 10.0]), [90.0, 0.0], "west");
        let [yaw, pitch] = toward(eye, [10.0, 74.0, 20.0]);
        assert_eq!((yaw.abs(), pitch), (180.0, 45.0), "south, and up");
    }
}
