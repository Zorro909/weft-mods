//! A film's parts as bytes. Each enum starts with a tag byte for its variant; its `put` and `get`
//! are the only two places a tag is named, and `get` reads every field as the type the variant
//! declares.

use super::Wire;
use crate::film::{Ease, Look, Path, Shot, Step};

impl Wire for Step {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Step::Shot(shot) => {
                0u8.put(out);
                shot.put(out);
            }
            Step::Wait(secs) => {
                1u8.put(out);
                secs.put(out);
            }
            Step::WaitFor { name, max } => {
                2u8.put(out);
                name.put(out);
                max.put(out);
            }
            Step::Title { key, secs } => {
                3u8.put(out);
                key.put(out);
                secs.put(out);
            }
            Step::Cue(name) => {
                4u8.put(out);
                name.put(out);
            }
            Step::Menu { menu, at, secs } => {
                5u8.put(out);
                menu.put(out);
                at.put(out);
                secs.put(out);
            }
        }
    }

    fn get(bytes: &mut &[u8]) -> Option<Step> {
        Some(match u8::get(bytes)? {
            0 => Step::Shot(Wire::get(bytes)?),
            1 => Step::Wait(Wire::get(bytes)?),
            2 => Step::WaitFor {
                name: Wire::get(bytes)?,
                max: Wire::get(bytes)?,
            },
            3 => Step::Title {
                key: Wire::get(bytes)?,
                secs: Wire::get(bytes)?,
            },
            4 => Step::Cue(Wire::get(bytes)?),
            5 => Step::Menu {
                menu: Wire::get(bytes)?,
                at: Wire::get(bytes)?,
                secs: Wire::get(bytes)?,
            },
            _ => return None,
        })
    }
}

impl Wire for Shot {
    fn put(&self, out: &mut Vec<u8>) {
        self.path.put(out);
        self.look.put(out);
        self.secs.put(out);
        self.ease.put(out);
        self.fov.put(out);
        self.blend.put(out);
    }

    fn get(bytes: &mut &[u8]) -> Option<Shot> {
        Some(Shot {
            path: Wire::get(bytes)?,
            look: Wire::get(bytes)?,
            secs: Wire::get(bytes)?,
            ease: Wire::get(bytes)?,
            fov: Wire::get(bytes)?,
            blend: Wire::get(bytes)?,
        })
    }
}

impl Wire for Ease {
    fn put(&self, out: &mut Vec<u8>) {
        let tag: u8 = match self {
            Ease::Linear => 0,
            Ease::Smooth => 1,
            Ease::In => 2,
            Ease::Out => 3,
        };
        tag.put(out);
    }

    fn get(bytes: &mut &[u8]) -> Option<Ease> {
        Some(match u8::get(bytes)? {
            0 => Ease::Linear,
            1 => Ease::Smooth,
            2 => Ease::In,
            3 => Ease::Out,
            _ => return None,
        })
    }
}

impl Wire for Path {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Path::Still(eye) => {
                0u8.put(out);
                eye.put(out);
            }
            Path::Line(from, to) => {
                1u8.put(out);
                from.put(out);
                to.put(out);
            }
            Path::Spline(points) => {
                2u8.put(out);
                points.put(out);
            }
            Path::Orbit {
                center,
                radius,
                height,
                angle,
            } => {
                3u8.put(out);
                center.put(out);
                radius.put(out);
                height.put(out);
                angle.put(out);
            }
            Path::Follow { subject, offset } => {
                4u8.put(out);
                subject.put(out);
                offset.put(out);
            }
        }
    }

    fn get(bytes: &mut &[u8]) -> Option<Path> {
        Some(match u8::get(bytes)? {
            0 => Path::Still(Wire::get(bytes)?),
            1 => Path::Line(Wire::get(bytes)?, Wire::get(bytes)?),
            2 => Path::Spline(Wire::get(bytes)?),
            3 => Path::Orbit {
                center: Wire::get(bytes)?,
                radius: Wire::get(bytes)?,
                height: Wire::get(bytes)?,
                angle: Wire::get(bytes)?,
            },
            4 => Path::Follow {
                subject: Wire::get(bytes)?,
                offset: Wire::get(bytes)?,
            },
            _ => return None,
        })
    }
}

impl Wire for Look {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Look::At(point) => {
                0u8.put(out);
                point.put(out);
            }
            Look::Pan(from, to) => {
                1u8.put(out);
                from.put(out);
                to.put(out);
            }
            Look::Subject(name) => {
                2u8.put(out);
                name.put(out);
            }
            Look::Ahead => 3u8.put(out),
            Look::Angles { yaw, pitch } => {
                4u8.put(out);
                yaw.put(out);
                pitch.put(out);
            }
            Look::Along(angles) => {
                5u8.put(out);
                angles.put(out);
            }
        }
    }

    fn get(bytes: &mut &[u8]) -> Option<Look> {
        Some(match u8::get(bytes)? {
            0 => Look::At(Wire::get(bytes)?),
            1 => Look::Pan(Wire::get(bytes)?, Wire::get(bytes)?),
            2 => Look::Subject(Wire::get(bytes)?),
            3 => Look::Ahead,
            4 => Look::Angles {
                yaw: Wire::get(bytes)?,
                pitch: Wire::get(bytes)?,
            },
            5 => Look::Along(Wire::get(bytes)?),
            _ => return None,
        })
    }
}
