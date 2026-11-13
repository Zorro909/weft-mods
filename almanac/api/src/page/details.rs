//! What the shown quest asks and gives.

use weft_sdk::Item;

use super::*;
use crate::record::{Kind, Reward, Task};

/// Side of a reward's slot in GUI pixels, as an inventory slot.
const SLOT: f32 = 18.0;
/// Side of a task's light.
const LIGHT: f32 = 6.0;
/// Space between the parts of a task's line.
const GAP: f32 = 3.0;
/// Width of a digit of a task's count, which is written in cells of one width.
const DIGIT: f32 = 6.0;
/// Side of an item's icon.
const ICON: f32 = 16.0;

impl Page {
    /// The shown quest, `width` GUI pixels wide: its name and description, `tasks`, `rewards`,
    /// and the quest it waits for while it is locked. Its texts wrap at that width.
    pub fn details(&self, ui: &mut Ui, width: f32) {
        let Some(quest) = self.shown else {
            return ui.text_styled(S::column().class("dim").w(width), "almanac.no_quests", &[]);
        };
        let held = &self.book.quests[quest];
        let title = quest_text(&held.name);
        ui.column(S::column().gap(4.0).w(width), |ui| {
            ui.row(S::row().gap(4.0), |ui| {
                ui.item(held.icon, 1, S::default());
                let beside = S::column().class("title").w(width - ICON - 4.0);
                ui.text_styled(beside, &title, &[]);
            });
            ui.text_styled(
                S::column().class("dim").w(width),
                &format!("{title}.desc"),
                &[],
            );
            self.tasks(ui, width);
            self.rewards(ui);
            if let Some(before) = self.book.waits_for(quest, &self.finished) {
                let before = quest_text(&self.book.quests[before].name);
                ui.text_styled(S::column().w(width), "almanac.first", &[Arg::Key(&before)]);
            }
        });
    }

    /// The shown quest's tasks, `width` GUI pixels wide, a line each: a light, what to do, and
    /// how far the player is.
    pub fn tasks(&self, ui: &mut Ui, width: f32) {
        let Some(quest) = self.shown else { return };
        let held = &self.book.quests[quest];
        let done = self.finished.has(held.key);
        let counts = self.counting.of(held.key);
        ui.column(S::column().gap(2.0).w(width), |ui| {
            ui.text("almanac.todo");
            for (task, count) in held.tasks.iter().zip(counts) {
                let count = if done {
                    task.count
                } else {
                    count.min(task.count)
                };
                ui.row(S::row().gap(GAP).w(width), |ui| {
                    task_line(ui, task, count, width)
                });
            }
        });
    }

    /// The items the shown quest hands over, a slot each; nothing for a quest that gives none.
    pub fn rewards(&self, ui: &mut Ui) {
        let Some(quest) = self.shown else { return };
        let rewards = &self.book.quests[quest].rewards;
        if rewards.is_empty() {
            return;
        }
        ui.column(S::column().gap(2.0), |ui| {
            ui.text("almanac.rewards");
            ui.row(S::row().gap(1.0), |ui| {
                for reward in rewards {
                    reward_slot(ui, reward);
                }
            });
        });
    }
}

/// A task's line, `width` wide: its light, its words wrapping in what the count leaves them,
/// and the count at the right edge.
fn task_line(ui: &mut Ui, task: &Task, count: u32, width: f32) {
    let light = match count {
        0 => "base:gui/led_off",
        count if count < task.count => "base:gui/led_wait",
        _ => "base:gui/led_on",
    };
    ui.image(light, LIGHT, LIGHT);
    let numbers = format!("{count}/{}", task.count);
    let numbers_wide = numbers.len() as f32 * DIGIT;
    task_text(
        ui,
        task,
        S::column().w(width - LIGHT - numbers_wide - 2.0 * GAP),
    );
    ui.spacer();
    ui.text_styled(
        S::column().class("dim").no_wrap().mono(),
        "almanac.count",
        &[Arg::Text(&numbers)],
    );
}

/// What `task` asks, in the task's own words or in ones made from its kind and what it counts.
fn task_text(ui: &mut Ui, task: &Task, style: S) {
    if !task.text.is_empty() {
        return ui.text_styled(style, &task.text, &[]);
    }
    let kind = task.kind().unwrap_or(Kind::Count);
    let key = format!("almanac.task.{}", kind.name());
    let name = match kind {
        Kind::Kill => format!("entity.{}", task.what.replace(':', ".")),
        Kind::Place | Kind::Dig => format!("block:{}", task.what),
        _ => format!("item:{}", task.what),
    };
    let what = match kind {
        // A tag has no name: it shows as written unless the task has words of its own.
        _ if task.what.starts_with('#') => Arg::Text(&task.what),
        Kind::Count => Arg::Text(&task.name),
        Kind::Kill => Arg::Key(&name),
        _ => Arg::Name(&name),
    };
    ui.text_styled(style, &key, &[what]);
}

fn reward_slot(ui: &mut Ui, reward: &Reward) {
    let item = Item::lookup(&reward.item).unwrap_or(0);
    let name = format!("item:{}", reward.item);
    let tip = [Arg::Name(&name), Arg::Int(reward.count as i64)];
    let slot = S::row()
        .size(SLOT, SLOT)
        .center()
        .tooltip_args("almanac.reward.tip", &tip);
    ui.panel("slot", slot, |ui| ui.item(item, reward.count, S::default()));
}
