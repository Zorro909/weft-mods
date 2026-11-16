//! What the server's admins run: the whitelist and the spawn.

pub mod spawn;
mod whitelist;
mod whitelist_file;

use weft_sdk::Registry;

pub fn register(r: &mut Registry) {
    whitelist::register(r);
    spawn::register(r);
}
