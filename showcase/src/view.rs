//! What the player sees while a film plays: the film camera, their own player moved along with
//! it, and the film's text on screen.

use std::sync::Mutex;

use weft_sdk::abi::{CameraArgs, CameraPose, PlayerControl, control_mask as cm};
use weft_sdk::settings::FOV;
use weft_sdk::*;

use crate::lock;
use crate::playback::Title;
use crate::rig::Pose;
use crate::rig::math::{add, forward, sub};

/// The camera mode a film is seen through; the server holds the player in it while one plays.
pub static FILM: CameraMode = CameraMode::named("film");

/// Where the film puts the camera this frame. The last one stays after the film, for the
/// camera to ease out from until the server lets go of it; `None` before the first film.
static SHOWN: Mutex<Option<Pose>> = Mutex::new(None);

/// Seconds a title takes to fade in, and to fade out.
const FADE: f32 = 0.6;
/// Blocks in front of the camera that titles are drawn at.
const TITLE_DISTANCE: f64 = 12.0;
/// Where titles sit, as a share of the half picture below the middle.
const TITLE_LOW: f64 = 0.78;

pub fn register(r: &mut Registry) {
    // No body, hand, HUD or effects: what is seen is the film's alone, its field of view too.
    r.camera(&FILM).shot(Shot::new(0, film_shot));
}

/// The film camera's frame: what `put` showed last, or the player's eyes before any film. Frame
/// hooks run before the camera moves, so a film's pose shows the same frame, not the next.
fn film_shot(args: &mut CameraArgs) {
    args.out = match *lock(&SHOWN) {
        Some(pose) => CameraPose {
            pos: pose.eye.into(),
            yaw: pose.yaw.to_radians() as f32,
            pitch: pose.pitch.to_radians() as f32,
            // The camera's field of view is a multiple of the player's.
            fov: pose.fov / FOV.get(),
            ..CameraPose::DEFAULT
        },
        None => CameraPose {
            pos: args.eye,
            yaw: args.eye_yaw,
            pitch: args.eye_pitch,
            ..CameraPose::DEFAULT
        },
    };
}

/// The camera now, from the local player and its settings.
pub fn camera(c: &Client) -> Pose {
    let s = c.state();
    Pose {
        eye: add(s.pos.into(), [0.0, eye_height(c), 0.0]),
        yaw: (s.yaw as f64).to_degrees(),
        pitch: (s.pitch as f64).to_degrees(),
        fov: FOV.get(),
    }
}

/// Shows `pose` through the film camera, and moves the local player's eyes there: the world loads
/// around the player, not the camera.
pub fn put(c: &Client, pose: Pose) {
    *lock(&SHOWN) = Some(pose);
    c.control(&PlayerControl {
        mask: cm::POS | cm::VEL | cm::LOOK | cm::FLYING,
        pos: sub(pose.eye, [0.0, eye_height(c), 0.0]).into(),
        vel: Vec3::ZERO,
        yaw: pose.yaw.to_radians() as f32,
        pitch: pose.pitch.to_radians() as f32,
        flying: 1,
        ..PlayerControl::DEFAULT
    });
}

fn eye_height(c: &Client) -> f64 {
    c.entity(0).map_or(1.62, |e| e.size.eye as f64)
}

/// The film's titles at `now` seconds into it, faded in and out, near the bottom of the picture.
/// They are drawn as labels in the world in front of the camera, which stay visible while the
/// HUD is hidden for recording.
pub fn titles(c: &Client, titles: &[Title], now: f32, pose: Pose) {
    // The field of view is vertical: this many degrees down is that far down the picture.
    let down = (TITLE_LOW * (pose.fov as f64 / 2.0).to_radians().tan())
        .atan()
        .to_degrees();
    let ahead = forward(pose.yaw, pose.pitch - down);
    let at = add(pose.eye, ahead.map(|v| v * TITLE_DISTANCE));
    let size = title_size(c);
    for title in titles {
        let alpha = (shown(title, now) * 255.0) as u32;
        c.draw_text(None, at, &title.key, &[], 0xFFFFFF00 | alpha, size);
    }
}

/// How much of `title` shows at `now`, 0 to 1, fading in from its start and out to its end.
fn shown(title: &Title, now: f32) -> f32 {
    let since_start = (now - title.from) / FADE;
    let to_end = (title.until - now) / FADE;
    since_start.min(to_end).clamp(0.0, 1.0)
}

/// How big titles are drawn, as a multiple of menu text, which grows with the GUI scale: the
/// same share of the picture's width at any window size and scale, about 70 letters across.
fn title_size(c: &Client) -> f32 {
    // A letter of menu text is about this many GUI pixels wide.
    const LETTER: f32 = 6.4;
    const LETTERS_ACROSS: f32 = 70.0;
    let s = c.state();
    s.window[0] as f32 / LETTERS_ACROSS / (LETTER * s.gui_scale.max(1.0))
}

/// Says on screen that the film `film` was asked for and never came.
pub fn missing_notice(c: &Client, film: &str) {
    c.draw_screen_text(
        0.5,
        0.3,
        "showcase.missing",
        &[Arg::Text(film)],
        0xFF8080FF,
        1.5,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_fades_in_and_out_within_its_time() {
        let title = Title {
            key: "mymod.title".into(),
            from: 2.0,
            until: 6.0,
        };
        assert_eq!(shown(&title, 1.0), 0.0, "before it");
        assert!((shown(&title, 2.3) - 0.5).abs() < 1e-6, "halfway in");
        assert_eq!(shown(&title, 4.0), 1.0, "in the middle");
        assert!((shown(&title, 5.7) - 0.5).abs() < 1e-6, "halfway out");
        assert_eq!(shown(&title, 6.5), 0.0, "after it");
    }
}
