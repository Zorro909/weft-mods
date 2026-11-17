//! The refinery: a steel column 3 by 3 by 6 that distils crude oil into gas, diesel, lubricant
//! and tar, with sulfur besides (`products`). Its controller, in the middle of the bottom of its
//! front, checks the column behind it every tick (`column`) and runs it once it is built
//! (`work`): crude comes into the controller by liquid tube, heat into a heater on the bottom
//! layer (40 mB of steam a tick, or 300 energy a tick for its burner), cooling water into a
//! condenser on the top, and each product leaves by the taps on its own layer.
//!
//! The challenge is its temperature (`heat`), kept by the controller: heat raises it, crude and
//! cooling bring it down. In its band it distils; too cold, everything goes to tar; too hot,
//! the gas vents. A product whose taps have no room stops the column.

mod column;
pub mod heat;
mod parts;
pub mod products;
mod work;

use column::Fault;
pub use parts::{
    REFINERY_CONDENSER, REFINERY_HEATER, REFINERY_TAP, STEAM_CAPACITY, TAP_CAPACITY, WATER_CAPACITY,
};
use products::Product;
use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, ClientView, Component, Facings, Holds, Menu, Particle, Registry, S, Slots, Toward,
    Ui, component_flags, value,
};

use crate::liquid::HELD;

pub static REFINERY: Block = Block::named("refinery");
static REFINERY_MENU: Menu = Menu::named("refinery");
/// The pale vapour of gas let out of the top of a column too hot to keep it.
static VAPOUR: Particle = Particle::named("refinery_vapour");

/// Millibuckets of crude its controller holds.
pub const CRUDE_CAPACITY: u32 = 4_000;
/// Its slots: the sulfur it makes.
const OUTPUT: u32 = 1;

value! {
    /// A refinery, kept by its controller: its `warmth` in degrees above the air, the crude
    /// distilled since its last sulfur (`toward_sulfur`), its status, the product whose taps
    /// have no room (`blocked`, a `Product` number), how it was heated and whether it was cooled
    /// last tick, while it is not built its fault and the block where, and `checked`, 1 once its
    /// first tick has shown its casing as it is. Its crude is its `HELD`.
    #[derive(Default)]
    pub struct Refinery {
        pub warmth: f32,
        pub toward_sulfur: u32,
        pub status: u32,
        pub blocked: u32,
        pub heating: u32,
        pub cooled: u32,
        pub fault: u32,
        pub fault_x: i32,
        pub fault_y: i32,
        pub fault_z: i32,
        pub checked: u32,
    }
}
static REFINING: Component<Refinery> = Component::named("cogwork:refinery");

/// What the column is doing; its screen says `cogwork.refinery.<name>`. Kept in
/// `Refinery::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// Its column is not built: `Refinery::fault` says what is wrong.
    #[default]
    Unbuilt,
    NoCrude,
    /// Too cold to take crude in, and heated.
    Heating,
    /// Too cold to take crude in, and not heated.
    NoHeat,
    Distilling,
    /// Warm, but below its band: everything goes to tar.
    TooCold,
    /// Above its band: the gas vents.
    TooHot,
    /// A product's taps have no room: `Refinery::blocked` says which.
    TapFull,
    /// Its sulfur has no room in its output.
    Full,
}

/// How a column was heated in a tick. Kept in `Refinery::heating` as its number.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Heating {
    #[default]
    Unheated,
    Steam,
    Burner,
}

impl Refinery {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }

    pub fn fault(&self) -> Fault {
        Fault::from_repr(self.fault).unwrap_or_default()
    }

    /// Keeps it from running for `problem`.
    fn set_unbuilt(&mut self, problem: column::Problem) {
        self.set_status(Status::Unbuilt);
        self.fault = problem.fault as u32;
        (self.fault_x, self.fault_y, self.fault_z) = (problem.at.x, problem.at.y, problem.at.z);
    }

    pub fn heating(&self) -> Heating {
        Heating::from_repr(self.heating).unwrap_or_default()
    }

    /// The product whose taps have no room, while one hasn't.
    pub fn blocked(&self) -> Option<Product> {
        Product::from_repr(self.blocked)
    }

    /// Its temperature in degrees.
    pub fn degrees(&self) -> f32 {
        heat::degrees(self.warmth)
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &REFINING,
        Refinery::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&REFINING, work::tick);
    products::register(r);
    parts::register(r);
    parts::part(r.block(&REFINERY))
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .states(|props, state| {
            let front = if props.is("working", "true") {
                "refinery_front_on"
            } else {
                "refinery_front"
            };
            // The front faces north: `facing` turns it.
            state.texture("refinery_casing").front(front)
        })
        .on_break(|world, change| work::broken(world, change.pos.into(), change.state.into()))
        .entity(|entity| entity.attach(&REFINING).attach(&HELD).inventory(OUTPUT))
        .opens(&REFINERY_MENU);
    r.menu(&REFINERY_MENU)
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    r.particle(&VAPOUR)
        .textures(&["core:generic_3", "core:generic_4", "core:generic_5"])
        .life(1.5, 2.5)
        .size(0.3, 0.5)
        .gravity(-0.1)
        .drag(0.93)
        .color(0xE8E4C8C0, 0xC8C4A800)
        .translucent();
}

/// The refinery's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Refinery Column                                                 |
/// | [######|###|...]  352 °C (distils at 320 to 380 °C)             |  the band marked on the bar
/// | Distilling 10 mB of crude a tick                                |  status
/// | Heated by steam                                                 |  or by its burner, or not
/// | Cooled by water                                                 |  or not
/// | [####......]  3000 / 4000 mB of Oil                             |  its crude
/// | Sulfur [ ]                                                      |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(6.0), |ui| {
            ui.progress_bind(
                "cogwork:refinery_heat",
                S::default().size(80.0, 8.0),
                |view| refinery_of(view).degrees() / heat::SCALE,
            );
            ui.text_bind(|view, out| {
                let degrees = Arg::Int(refinery_of(view).degrees().round() as i64);
                let low = Arg::Int(*heat::BAND.start() as i64);
                let high = Arg::Int(*heat::BAND.end() as i64);
                out.set("cogwork.refinery.temperature", &[degrees, low, high]);
            });
        });
        ui.text_bind(|view, out| {
            let (key, args) = status_line(&refinery_of(view));
            out.set(&key, &args);
        });
        ui.text_bind(|view, out| {
            let heating: &str = refinery_of(view).heating().into();
            out.set(&format!("cogwork.refinery.{heating}"), &[]);
        });
        ui.text_bind(|view, out| {
            let cooled = refinery_of(view).cooled != 0;
            let key = if cooled { "cooled" } else { "uncooled" };
            out.set(&format!("cogwork.refinery.{key}"), &[]);
        });
        crate::machine::tank_bar(ui, CRUDE_CAPACITY);
        ui.row(S::row().gap(6.0), |ui| {
            ui.text("cogwork.refinery.output");
            ui.group("output");
        });
        ui.player_inventory(true);
    });
}

/// The status line of `refinery`: its lang key and arguments.
fn status_line(refinery: &Refinery) -> (String, Vec<Arg<'static>>) {
    match refinery.status() {
        Status::Unbuilt => {
            let fault: &str = refinery.fault().into();
            let place = [refinery.fault_x, refinery.fault_y, refinery.fault_z];
            let args = place.map(|coordinate| Arg::Int(coordinate as i64));
            (format!("cogwork.refinery.{fault}"), Vec::from(args))
        }
        Status::TapFull => {
            let product = refinery.blocked().map_or("tar", Product::name);
            let name = Arg::Name(product_item(product));
            ("cogwork.refinery.tap_full".to_string(), vec![name])
        }
        status => {
            let name: &str = status.into();
            let crude = Arg::Int(products::CRUDE_A_TICK as i64);
            let warm = Arg::Int(heat::WARM as i64);
            (format!("cogwork.refinery.{name}"), vec![crude, warm])
        }
    }
}

/// The name of the item of the product named `product`, as a screen names it.
fn product_item(product: &str) -> &'static str {
    match product {
        "gas" => "item:cogwork:gas",
        "diesel" => "item:cogwork:diesel",
        "lubricant" => "item:cogwork:lubricant",
        _ => "item:cogwork:tar",
    }
}

fn refinery_of(view: &ClientView) -> Refinery {
    view.menu_component(&REFINING).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_fault_and_line_has_a_name() {
        let statuses = Status::VARIANTS.iter().map(|&status| <&str>::from(status));
        let faults = Fault::VARIANTS.iter().map(|&fault| <&str>::from(fault));
        let heating = Heating::VARIANTS
            .iter()
            .map(|&heating| <&str>::from(heating));
        let lines = ["temperature", "cooled", "uncooled", "output"];
        for name in statuses.chain(faults).chain(heating).chain(lines) {
            let key = format!("cogwork.refinery.{name}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn every_product_is_named_on_the_screen() {
        for &product in Product::VARIANTS {
            let item = product_item(product.name());
            assert_eq!(
                item,
                format!("item:cogwork:{}", product.name()),
                "{product:?}"
            );
        }
    }

    #[test]
    fn an_unknown_saved_number_reads_as_the_default() {
        let refinery = Refinery {
            status: 99,
            fault: 99,
            heating: 99,
            blocked: 99,
            ..Refinery::default()
        };
        assert_eq!(refinery.status(), Status::Unbuilt, "a newer status");
        assert_eq!(refinery.fault(), Fault::NotPart, "a newer fault");
        assert_eq!(refinery.heating(), Heating::Unheated, "a newer heating");
        assert_eq!(refinery.blocked(), None, "a newer product");
    }

    #[test]
    fn a_new_column_stands_at_the_air_s_warmth() {
        assert_eq!(Refinery::default().degrees(), heat::AIR, "cold");
    }

    #[test]
    fn what_a_refinery_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Refinery, warmth), 0, "its warmth");
        assert_eq!(offset_of!(Refinery, toward_sulfur), 4, "toward sulfur");
        assert_eq!(offset_of!(Refinery, status), 8, "its status");
        assert_eq!(offset_of!(Refinery, heating), 16, "how it was heated");
        assert_eq!(offset_of!(Refinery, fault_x), 28, "where its fault is");
        assert_eq!(offset_of!(Refinery, checked), 40, "whether checked");
    }
}
