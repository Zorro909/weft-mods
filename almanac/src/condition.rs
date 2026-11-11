//! `almanac:done(<quest>)`: a condition any text, loot table or spawn rule can ask, true for a
//! player who finished the quest. `{if:almanac:done(cogwork:wound_up)}..{/if}` in a translation.

use almanac_api::{FINISHED, key_of};
use weft_sdk::*;

static DONE: Condition = Condition::named("done");

pub fn register(r: &mut Registry) {
    r.condition(&DONE, |asked| {
        let quest = key_of(asked.arg());
        // Text asks on the client, about its own player; loot and spawning on the server, about
        // whoever broke the block or stands there.
        let finished = match (asked.client(), asked.world()) {
            (Some(view), _) => view.component(&FINISHED),
            (None, Some(world)) => world.component(asked.entity(), &FINISHED),
            (None, None) => None,
        };
        finished.is_some_and(|finished| finished.has(quest))
    });
}
