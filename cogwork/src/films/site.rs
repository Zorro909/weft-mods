//! Where the film is set: a meadow of seed 42 that slopes down to a lake, some 90 blocks north
//! of where the world puts the player, round a seep of oil with its field underneath. The set
//! is built and filmed in blocks from `SITE`, wherever the film is started from, so it is for
//! that seed and the land as it now generates there.

use showcase_api::{Film, Look, Path, Set, Step};

use crate::lib::Pos;

/// The block everything is counted from, in the world: the first above the hall's floor, on
/// the hall's long axis at its west end. The seep's puddle is the block (-32, 2, -13) from it.
pub const SITE: Pos = Pos::new(16, 76, -76);

/// The middle of `block`, `up` blocks higher: where the camera looks to have it in view.
pub const fn above(block: Pos, up: f64) -> [f64; 3] {
    [
        block.x as f64 + 0.5,
        block.y as f64 + 0.5 + up,
        block.z as f64 + 0.5,
    ]
}

/// `set`, counting from the site instead of from where the film was started.
pub fn on_site(set: &Set) -> Set {
    Set::new(set.world(), set.player(), SITE)
}

/// A cue handler that is handed the set counted from the site.
pub fn at_site(handler: fn(&Set)) -> impl Fn(&Set) + Send + Sync + 'static {
    move |set| handler(&on_site(set))
}

/// `film`, written in blocks from the site, counted from `origin` instead: the block the film
/// was started at, from which the showcase counts.
pub fn from_site(origin: Pos, film: Film) -> Film {
    let by = [SITE.x - origin.x, SITE.y - origin.y, SITE.z - origin.z];
    let point = |place: [f64; 3]| [0, 1, 2].map(|axis| place[axis] + f64::from(by[axis]));
    let steps = film.steps.into_iter().map(|step| match step {
        Step::Shot(mut shot) => {
            shot.path = match shot.path {
                Path::Still(eye) => Path::Still(point(eye)),
                Path::Line(from, to) => Path::Line(point(from), point(to)),
                Path::Spline(points) => Path::Spline(points.into_iter().map(point).collect()),
                Path::Orbit {
                    center,
                    radius,
                    height,
                    angle,
                } => Path::Orbit {
                    center: point(center),
                    radius,
                    height,
                    angle,
                },
                follow => follow,
            };
            shot.look = match shot.look {
                Look::At(at) => Look::At(point(at)),
                Look::Pan(from, to) => Look::Pan(point(from), point(to)),
                look => look,
            };
            Step::Shot(shot)
        }
        Step::Menu { menu, at, secs } => Step::Menu {
            menu,
            at: at + Pos::new(by[0], by[1], by[2]),
            secs,
        },
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
    fn a_block_is_looked_at_in_its_middle() {
        let pump = Pos::new(-32, 3, -13);
        assert_eq!(above(pump, 0.0), [-31.5, 3.5, -12.5], "its middle");
        assert_eq!(above(pump, 1.0), [-31.5, 4.5, -12.5], "a block higher");
    }

    #[test]
    fn places_are_counted_from_where_the_film_starts() {
        let film = Film::new()
            .shot(dolly([1.0, 2.0, 3.0], [4.0, 2.0, 3.0]).look_at([0.0, 0.0, 0.0]))
            .shot(orbit([1.0, 2.0, 3.0], 5.0))
            .menu("cogwork:pump", Pos::new(-32, 3, -13), 4.0);
        let moved = from_site(Pos::new(16, 97, 0), film);
        let Step::Shot(dolly) = &moved.steps[0] else {
            panic!("a shot")
        };
        let line = Path::Line([1.0, -19.0, -73.0], [4.0, -19.0, -73.0]);
        assert_eq!(dolly.path, line, "the path");
        assert_eq!(dolly.look, Look::At([0.0, -21.0, -76.0]), "where it looks");
        let Step::Shot(orbit) = &moved.steps[1] else {
            panic!("a shot")
        };
        let Path::Orbit { center, radius, .. } = orbit.path else {
            panic!("an orbit")
        };
        let moved_center = [1.0, -19.0, -73.0];
        assert_eq!(
            (center, radius),
            (moved_center, [5.0; 2]),
            "only the center"
        );
        let Step::Menu { at, .. } = &moved.steps[2] else {
            panic!("a menu")
        };
        assert_eq!(*at, Pos::new(-32, -18, -89), "the block whose menu opens");
    }
}
