//! A film's menu on screen: the server opens it on its block, as if the camera had used it, and
//! the client closes it once its time is up or the film ends.

use showcase_api::name;
use weft_sdk::*;

use crate::playback::OpenMenu;
use crate::server::{OPEN_MENU, OpenMenu as Open};

/// The menu this client asked for and hasn't closed yet: its name and block.
#[derive(Default)]
pub struct Menus {
    open: Option<(String, [i32; 3])>,
}

impl Menus {
    /// Opens `wanted` if it isn't open yet, closing the one before; `None` closes it.
    pub fn show(&mut self, c: &Client, wanted: Option<&OpenMenu>) {
        let wanted = wanted.map(|m| (m.menu.clone(), m.at));
        if self.open == wanted {
            return;
        }
        if self.open.take().is_some() {
            c.close_screen();
        }
        if let Some((menu, at)) = &wanted {
            c.send(
                &OPEN_MENU,
                &Open {
                    menu: name(menu),
                    at: *at,
                },
            );
        }
        self.open = wanted;
    }
}
