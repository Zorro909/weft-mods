//! `/almanac`, for whoever sets a world up: forget one's own progress, or finish a quest
//! without doing its tasks.

use almanac_api::{book, quest_text};
use weft_sdk::*;

use crate::progress;

const NAME: &str = "almanac";

pub fn register(r: &mut Registry) {
    r.command(NAME, "reset", |cmd| {
        progress::reset(&cmd.world(), cmd.player());
        cmd.reply("command.almanac.reset", &[]);
    });
    r.command_branch(NAME, "finish <quest:text>", finish);
}

/// `/almanac finish cogwork:wound_up`
fn finish(cmd: &mut ServerCommand) {
    let name = cmd.text("quest").unwrap_or_default().to_string();
    let Some(quest) = book().quest_named(&name) else {
        return cmd.reply("command.almanac.no_quest", &[Arg::Text(&name)]);
    };
    let title = quest_text(&name);
    if progress::finish(&cmd.world(), cmd.player(), quest) {
        cmd.reply("command.almanac.finished", &[Arg::Key(&title)]);
    } else {
        cmd.reply("command.almanac.already", &[Arg::Key(&title)]);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_answer_has_words() {
        for answer in ["reset", "no_quest", "finished", "already"] {
            assert!(
                crate::translated(&format!("command.almanac.{answer}")),
                "{answer}"
            );
        }
    }
}
