//! What a film says: its steps and camera shots, as plain data built with the functions here.
//! Positions are in blocks from the set's origin (`Set::origin`), angles in degrees.

/// A film's steps, in the order they play.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Film {
    pub steps: Vec<Step>,
}

/// One thing a film does. Shots and waits take their time; the others happen at once.
#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Shot(Shot),
    /// The camera stays where the last shot left it, for seconds.
    Wait(f32),
    /// The camera stays until the film's mod sends `signal(name)`, at most `max` seconds.
    WaitFor {
        name: String,
        max: f32,
    },
    /// A line of text from the lang files on screen for `secs`, while the film goes on.
    Title {
        key: String,
        secs: f32,
    },
    /// Fires `CUE` on the server, for the film's mod to start something.
    Cue(String),
    /// The menu `menu` of the block `at` open on screen for `secs`, while the film goes on.
    Menu {
        menu: String,
        at: [i32; 3],
        secs: f32,
    },
}

/// One camera move: where the eye goes, where it looks, how long it takes. The functions below
/// make shots of 5 seconds that ease in and out (`Ease::Smooth`), keep the player's field of view
/// and cut from the shot before.
#[derive(Clone, Debug, PartialEq)]
pub struct Shot {
    pub path: Path,
    pub look: Look,
    pub secs: f32,
    pub ease: Ease,
    /// Field of view at the start and end, in degrees; 0 keeps the player's own.
    pub fov: [f32; 2],
    /// Seconds at the start in which the camera comes over from where the last shot ended;
    /// 0 cuts.
    pub blend: f32,
}

/// Where the eye goes during a shot.
#[derive(Clone, Debug, PartialEq)]
pub enum Path {
    Still([f64; 3]),
    /// A straight line at an even speed.
    Line([f64; 3], [f64; 3]),
    /// A smooth curve through every point, at an even speed.
    Spline(Vec<[f64; 3]>),
    /// Around `center` from `angle[0]` to `angle[1]` (0 toward +x, 90 toward +z), while the
    /// distance and the height above the center go from their first value to their second.
    Orbit {
        center: [f64; 3],
        radius: [f64; 2],
        height: [f64; 2],
        angle: [f64; 2],
    },
    /// Behind a subject (`subject`), `offset` from it.
    Follow {
        subject: String,
        offset: [f64; 3],
    },
}

/// Where the eye looks during a shot.
#[derive(Clone, Debug, PartialEq)]
pub enum Look {
    At([f64; 3]),
    /// From the first point to the second over the shot.
    Pan([f64; 3], [f64; 3]),
    Subject(String),
    /// Along the path; a shot that stands still keeps the way the camera looked before.
    Ahead,
    Angles {
        yaw: f32,
        pitch: f32,
    },
    /// Yaw and pitch at each point of a flyby (or spread evenly over any other shot), turning
    /// smoothly from one to the next: a path marked by hand, looking where it looked.
    Along(Vec<[f32; 2]>),
}

/// How a shot speeds up and slows down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ease {
    Linear,
    /// Gently at both ends.
    Smooth,
    /// Gently at the start only.
    In,
    /// Gently at the end only.
    Out,
}

impl Film {
    pub fn new() -> Film {
        Film::default()
    }

    pub fn shot(mut self, shot: Shot) -> Film {
        self.steps.push(Step::Shot(shot));
        self
    }

    pub fn wait(mut self, secs: f32) -> Film {
        self.steps.push(Step::Wait(secs));
        self
    }

    pub fn wait_for(mut self, name: &str, max: f32) -> Film {
        self.steps.push(Step::WaitFor {
            name: name.into(),
            max,
        });
        self
    }

    pub fn title(mut self, key: &str, secs: f32) -> Film {
        self.steps.push(Step::Title {
            key: key.into(),
            secs,
        });
        self
    }

    pub fn cue(mut self, name: &str) -> Film {
        self.steps.push(Step::Cue(name.into()));
        self
    }

    /// Opens the menu `menu` (`mymod:press`) on the block `at` in the set, as if the camera had
    /// used it, and closes it after `secs`.
    pub fn menu(mut self, menu: &str, at: [i32; 3], secs: f32) -> Film {
        self.steps.push(Step::Menu {
            menu: menu.into(),
            at,
            secs,
        });
        self
    }
}

fn shot(path: Path, look: Look) -> Shot {
    Shot {
        path,
        look,
        secs: 5.0,
        ease: Ease::Smooth,
        fov: [0.0; 2],
        blend: 0.0,
    }
}

/// A full turn around `center` at `radius`, a third of that above it, looking at it.
pub fn orbit(center: [f64; 3], radius: f64) -> Shot {
    let path = Path::Orbit {
        center,
        radius: [radius; 2],
        height: [radius / 3.0; 2],
        angle: [0.0, 360.0],
    };
    shot(path, Look::At(center))
}

/// Through `points`, looking ahead.
pub fn flyby(points: &[[f64; 3]]) -> Shot {
    shot(Path::Spline(points.to_vec()), Look::Ahead)
}

/// Along a straight line, looking ahead.
pub fn dolly(from: [f64; 3], to: [f64; 3]) -> Shot {
    shot(Path::Line(from, to), Look::Ahead)
}

/// Still at `eye`, looking the way the camera already does (`.look_at` aims it).
pub fn hold(eye: [f64; 3]) -> Shot {
    shot(Path::Still(eye), Look::Ahead)
}

/// Rising (or sinking) from `heights[0]` to `heights[1]` above `center`, `radius` away from it
/// toward `angle`, looking at it.
pub fn crane(center: [f64; 3], radius: f64, angle: f64, heights: [f64; 2]) -> Shot {
    let path = Path::Orbit {
        center,
        radius: [radius; 2],
        height: heights,
        angle: [angle; 2],
    };
    shot(path, Look::At(center))
}

/// `offset` away from the subject `name`, looking at it.
pub fn follow(name: &str, offset: [f64; 3]) -> Shot {
    shot(
        Path::Follow {
            subject: name.into(),
            offset,
        },
        Look::Subject(name.into()),
    )
}

impl Shot {
    pub fn secs(mut self, secs: f32) -> Shot {
        self.secs = secs;
        self
    }

    pub fn ease(mut self, ease: Ease) -> Shot {
        self.ease = ease;
        self
    }

    pub fn look(mut self, look: Look) -> Shot {
        self.look = look;
        self
    }

    pub fn look_at(self, point: [f64; 3]) -> Shot {
        self.look(Look::At(point))
    }

    pub fn fov(mut self, from: f32, to: f32) -> Shot {
        self.fov = [from, to];
        self
    }

    pub fn blend(mut self, secs: f32) -> Shot {
        self.blend = secs;
        self
    }

    /// Orbits only: the angles it goes between.
    pub fn arc(mut self, from: f64, to: f64) -> Shot {
        if let Path::Orbit { angle, .. } = &mut self.path {
            *angle = [from, to];
        }
        self
    }

    /// Orbits only: the distance from the center at the start and end.
    pub fn radius(mut self, from: f64, to: f64) -> Shot {
        if let Path::Orbit { radius, .. } = &mut self.path {
            *radius = [from, to];
        }
        self
    }

    /// Orbits only: the height above the center at the start and end.
    pub fn height(mut self, from: f64, to: f64) -> Shot {
        if let Path::Orbit { height, .. } = &mut self.path {
            *height = [from, to];
        }
        self
    }
}
