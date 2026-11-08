//! Cuboid loaders: the cuboid loader keeps its own cube loaded, the ultra cuboid loader every
//! cube up to three away at its height. Their screen switches between keeping only machines
//! running there and everything, and shows how many cubes cogwork keeps of the world's limit.

use strum::FromRepr;
use weft_sdk::*;

use super::tickets::{self, Keeper, Kept};
use super::{Cubes, Role};
use crate::lib::Pos;

pub static CUBOID_LOADER: Block = Block::named("cuboid_loader");
pub static ULTRA_CUBOID_LOADER: Block = Block::named("ultra_cuboid_loader");
static LOADER_MENU: Menu = Menu::named("cuboid_loader");
static ULTRA_MENU: Menu = Menu::named("ultra_cuboid_loader");

/// Cubes the ultra cuboid loader reaches each way along x and z.
const ULTRA_RANGE: u32 = 3;
/// From this share of the limit on, the count on the screen warns.
const NEAR_LIMIT: f64 = 0.9;

/// What a loader keeps running in its cubes, picked on its screen. Kept in `Loader::mode` as
/// its number, so the order is part of the save format.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u32)]
pub enum Mode {
    #[default]
    Machines,
    Everything,
}

impl Mode {
    fn keeper(self) -> Keeper {
        match self {
            Mode::Machines => Keeper::Loader,
            Mode::Everything => Keeper::LoaderAll,
        }
    }
}

value! {
    /// A loader: the cubes it reaches each way, what it keeps running (`Mode`), what came of
    /// keeping them (`Kept`), and how many cubes cogwork keeps loaded of the world's `limit`.
    #[derive(Default)]
    pub struct Loader {
        pub range: u32,
        pub mode: u32,
        pub kept: u32,
        pub used: i32,
        pub limit: i32,
    }
}
static LOADER: Component<Loader> = Component::named("cogwork:loader");

impl Loader {
    pub fn mode(&self) -> Mode {
        Mode::from_repr(self.mode).unwrap_or_default()
    }
}

value! {
    /// The loader `owner` is to keep what `mode` says running, as `Mode`'s number.
    pub struct SetMode {
        pub owner: u64,
        pub mode: u32,
    }
}
static SET_MODE: Channel<SetMode> = Channel::named("loader_mode");

pub fn register(r: &mut Registry) {
    r.entity_component(
        &LOADER,
        Loader::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&LOADER, tick);
    r.packet(&SET_MODE, set_mode);
    loader(r, &CUBOID_LOADER, &LOADER_MENU, "cuboid_loader", 0);
    loader(
        r,
        &ULTRA_CUBOID_LOADER,
        &ULTRA_MENU,
        "ultra_cuboid_loader",
        ULTRA_RANGE,
    );
}

fn loader(
    r: &mut Registry,
    block: &'static Block,
    menu: &'static Menu,
    texture: &'static str,
    range: u32,
) {
    r.block(block)
        .texture(texture)
        .hardness(5.0)
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .light(7)
        .on_break(super::let_go)
        .tab(&crate::TAB)
        .entity(move |e| {
            e.attach_with(
                &LOADER,
                Loader {
                    range,
                    ..Loader::default()
                },
            )
        })
        .opens(menu);
    r.menu(menu).screen(screen);
}

fn tick(b: &mut SystemBatch<Loader>) {
    let w = b.world();
    for (id, l, _) in b.iter_mut() {
        if let Some(st) = w.entity(id) {
            keep_loaded(&w, Pos::from(st.block_pos()), l);
        }
    }
}

/// Keeps the cubes of the loader at `pos` loaded, and notes for its screen what came of it and
/// how many cubes cogwork keeps of the world's limit.
fn keep_loaded(w: &World, pos: Pos, l: &mut Loader) {
    let key = super::key(w, pos, Role::Home);
    let cubes = Cubes::around(pos, l.range as i32);
    l.kept = tickets::keep(w, key, cubes, l.mode().keeper()) as u32;
    l.used = tickets::used(w.game_time());
    l.limit = tickets::limit();
}

/// Server side: switches the loader to the mode asked for, when the player is at it.
fn set_mode(p: &Packet, mode: SetMode) {
    let w = p.world();
    let (Some(loader), Some(player)) = (w.entity(mode.owner), w.entity(p.player())) else {
        return;
    };
    if !crate::ghost::within_reach(loader.pos, player.pos) {
        return;
    }
    if let (Some(l), Some(picked)) = (
        w.component_mut(mode.owner, &LOADER),
        Mode::from_repr(mode.mode),
    ) {
        l.mode = picked as u32;
    }
}

/// Client side: a button that switches the loader whose screen is open to `mode`.
fn mode_button(ui: &mut Ui, key: &str, mode: Mode) {
    ui.button(key, S::default().size(80.0, 20.0), move |c| {
        if let Some(owner) = c.menu_owner() {
            let mode = SetMode {
                owner,
                mode: mode as u32,
            };
            c.client().send(&SET_MODE, &mode);
        }
    });
}

/// A loader's screen:
///
/// ```text
/// +-----------------------------------------------+
/// | Ultra Cuboid Loader                           |
/// | Keeps 7 x 7 cubes loaded at its height        |  its reach
/// | Keeping machines running                      |  or why it isn't loading
/// | [ Machines only ] [  Everything  ]            |
/// | Cogwork keeps 49 of at most 4096 cubes loaded |  warns near the limit
/// +-----------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.text_bind(|c, out| {
            let l = c.menu_component(&LOADER).unwrap_or_default();
            let across = Arg::Int(2 * l.range as i64 + 1);
            let key = if l.range == 0 { "own" } else { "area" };
            out.set(&format!("cogwork.loader.{key}"), &[across]);
        });
        ui.text_bind(|c, out| {
            let l = c.menu_component(&LOADER).unwrap_or_default();
            out.set(status_key(&l), &[]);
        });
        ui.row(S::row().gap(4.0), |ui| {
            mode_button(ui, "cogwork.loader.machines", Mode::Machines);
            mode_button(ui, "cogwork.loader.everything", Mode::Everything);
        });
        ui.text_bind(|c, out| {
            let l = c.menu_component(&LOADER).unwrap_or_default();
            let key = if near_limit(l.used, l.limit) {
                "cogwork.loader.used_near"
            } else {
                "cogwork.loader.used"
            };
            let (used, limit) = (i64::from(l.used), i64::from(l.limit));
            out.set(key, &[Arg::Int(used), Arg::Int(limit)]);
        });
    });
}

/// What the loader is doing: what it keeps running, or why it keeps nothing.
fn status_key(l: &Loader) -> &'static str {
    match Kept::from_repr(l.kept).unwrap_or_default() {
        Kept::Loading => match l.mode() {
            Mode::Machines => "cogwork.loader.loading",
            Mode::Everything => "cogwork.loader.loading_all",
        },
        Kept::OverLimit => "cogwork.loader.over_limit",
        Kept::Refused => "cogwork.loader.refused",
    }
}

fn near_limit(used: i32, limit: i32) -> bool {
    used as f64 >= limit as f64 * NEAR_LIMIT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_count_warns_from_nine_tenths_of_the_limit() {
        assert!(!near_limit(3685, 4096), "just under");
        assert!(near_limit(3687, 4096), "nine tenths");
        assert!(near_limit(0, 0), "no loading allowed at all");
    }

    #[test]
    fn every_screen_line_has_a_name() {
        let keys = ["own", "area", "machines", "everything", "used", "used_near"];
        let modes = [Mode::Machines, Mode::Everything].map(|mode| Loader {
            mode: mode as u32,
            ..Loader::default()
        });
        let keys = keys
            .map(|k| format!("cogwork.loader.{k}"))
            .into_iter()
            .chain(modes.iter().map(|l| status_key(l).to_string()));
        for key in keys {
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }
}
