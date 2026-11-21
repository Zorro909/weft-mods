//! The note that slides in when a player finishes a quest, in the box of base's advancements.

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use almanac_api::{Quest, book, quest_text};
use weft_sdk::*;

/// Server to the player who finished it: the quest's record id.
static TOAST: Channel<u32> = Channel::named("toast");
static CHIME: Sound = Sound::named("base:ui.advancement");

/// Seconds a toast shows.
const SHOWN_SECONDS: f32 = 5.0;
/// Seconds it shows while more wait behind it.
const HURRIED_SECONDS: f32 = 2.0;
/// Seconds it takes to slide in, and out again.
const SLIDE_SECONDS: f32 = 0.35;
/// Width of the box (`base:gui/advancement_toast`) in GUI pixels.
const WIDTH: f32 = 160.0;
/// Its height.
const HEIGHT: f32 = 32.0;
/// Its gap to the window's edge.
const MARGIN: f32 = 4.0;
/// Its top: under where base shows an advancement, which one quest can bring with it.
const TOP: f32 = MARGIN + HEIGHT + MARGIN;
/// Where the icon sits in the box: its middle from the left, and its top.
const ICON: [f32; 2] = [16.0, 9.0];
/// Left edge of the two lines in the box.
const TEXT_LEFT: f32 = 30.0;
/// Tops of the two lines.
const LINES: [f32; 2] = [6.0, 17.0];

/// The quest's name, as RGBA.
const WHITE: u32 = 0xffff_ffff;
/// The line above it, as RGBA.
const GOLD: u32 = 0xffe0_70ff;

pub fn register(r: &mut Registry) {
    r.client_packet(&TOAST, |_, quest| waiting().push_back((quest, None)));
    r.client_frame(draw);
}

/// Shows `player` that they finished the quest with record `quest`.
pub fn send(world: &World, player: u64, quest: u32) {
    world.send(player, &TOAST, &quest);
}

/// Toasts waiting to be shown, the first one with the moment it appeared.
static WAITING: Mutex<VecDeque<(u32, Option<Instant>)>> = Mutex::new(VecDeque::new());

fn waiting() -> MutexGuard<'static, VecDeque<(u32, Option<Instant>)>> {
    WAITING
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Draws the first waiting toast, and takes it off once its time is over.
fn draw(client: &Client) {
    let mut waiting = waiting();
    // They were about the world just left.
    if client.state().flags & client::client_flags::JOINED == 0 {
        return waiting.clear();
    }
    let stays = if waiting.len() > 1 {
        HURRIED_SECONDS
    } else {
        SHOWN_SECONDS
    };
    let Some((quest, since)) = waiting.front_mut() else {
        return;
    };
    let book = book();
    let Some(quest) = book.quest(*quest).map(|quest| &book.quests[quest]) else {
        waiting.pop_front();
        return;
    };
    let since = *since.get_or_insert_with(|| {
        client.play_sound(&CHIME, None, 1.0);
        Instant::now()
    });
    let shown = since.elapsed().as_secs_f32();
    if shown > stays {
        waiting.pop_front();
        return;
    }
    draw_toast(client, quest, slid_in(shown, stays));
}

/// The box with `quest` in it, `in_view` of it slid in from the window's right edge (0 to 1).
fn draw_toast(client: &Client, quest: &Quest, in_view: f32) {
    let state = client.state();
    let scale = state.gui_scale.max(1.0);
    let window = [
        state.window[0] as f32 / scale,
        state.window[1] as f32 / scale,
    ];
    let left = window[0] - (WIDTH + MARGIN) * in_view;
    // Screen text is placed by fractions of the window, centered on x with its top at y.
    let text = |x: f32, y: f32, key: &str, args: &[Arg], color: u32, size: f32| {
        client.draw_screen_text(
            (left + x) / window[0],
            (TOP + y) / window[1],
            key,
            args,
            color,
            size,
        )
    };
    text(WIDTH / 2.0, 0.0, "almanac.toast.box", &[], WHITE, 1.0);
    // An icon in text is as tall as a line: twice the size fills the slot.
    text(ICON[0], ICON[1], "", &[Arg::Item(quest.icon)], WHITE, 2.0);
    let middle = TEXT_LEFT + (WIDTH - TEXT_LEFT - MARGIN) / 2.0;
    text(middle, LINES[0], "almanac.toast", &[], GOLD, 1.0);
    text(middle, LINES[1], &quest_text(&quest.name), &[], WHITE, 1.0);
}

/// How much of a toast that stays `stays` seconds is in view `shown` seconds after it appeared,
/// 0 to 1: it slides in from the right edge, eased, and back out at the end.
fn slid_in(shown: f32, stays: f32) -> f32 {
    let part = (shown / SLIDE_SECONDS)
        .min((stays - shown) / SLIDE_SECONDS)
        .clamp(0.0, 1.0);
    1.0 - (1.0 - part).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_toast_slides_in_stays_and_slides_out() {
        assert_eq!(slid_in(0.0, SHOWN_SECONDS), 0.0, "out of view at first");
        assert_eq!(slid_in(2.5, SHOWN_SECONDS), 1.0, "all in view in between");
        assert_eq!(
            slid_in(SHOWN_SECONDS, SHOWN_SECONDS),
            0.0,
            "gone at the end"
        );
        assert!(
            slid_in(SLIDE_SECONDS / 2.0, SHOWN_SECONDS) > 0.5,
            "eased: most of the way in half the time"
        );
        assert_eq!(
            slid_in(HURRIED_SECONDS, HURRIED_SECONDS),
            0.0,
            "a hurried one is gone sooner"
        );
    }

    #[test]
    fn its_sound_and_box_are_named() {
        assert!(crate::translated("almanac.toast.box") && crate::translated("almanac.toast"));
    }
}
