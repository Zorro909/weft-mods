//! A stall's two screens. Its owner's sets it up: per offer a sample of what it gives and gets,
//! each side with a Woof price that makes it Woof instead, then the stock and the till. Everyone
//! else's shops: the eight offers as text, how often each can still go through, and a button
//! that trades once (`trade.rs`). The shop shows no slots of the stall's, so nobody but its owner
//! can reach its items.

use weft_sdk::{Arg, Channel, ClientView, Menu, Packet, Registry, S, Slots, Ui, value};
use weftkit::Pos;

use super::offer::{Goods, OFFERS};
use super::stall::{self, PARTS, STALL, STOCK, Stall, Status};
use super::trade::{TRADE, Trade};

pub static SETUP: Menu = Menu::named("stall_setup");
pub static SHOP: Menu = Menu::named("stall_shop");

/// Width of a price field, in GUI pixels.
const PRICE_WIDTH: f32 = 30.0;

value! {
    /// The owner sets a Woof price of offer `offer` of the register `register`: what it gives
    /// (`gets` 0) or gets (1). 0 makes that side items again.
    pub struct SetPrice {
        pub register: u64,
        pub offer: u32,
        pub gets: u32,
        pub woof: u32,
    }
}

static PRICE: Channel<SetPrice> = Channel::named("stall_price");

pub fn register(r: &mut Registry) {
    let offers = OFFERS as u32;
    r.menu(&SETUP)
        .group("gives", Slots::column(offers).holder())
        .group("gets", Slots::column(offers).holder())
        .group("stock", Slots::chest(STOCK / 9).holder())
        .group("till", Slots::chest(STOCK / 9).holder())
        .player_inventory()
        .screen(setup_screen);
    r.menu(&SHOP).player_inventory().screen(shop_screen);
    r.packet(&PRICE, set_price);
}

/// The owner's screen:
///
/// ```text
/// +------------------------------------------------------+
/// | Oak Stall Register                                   |
/// | Open                                                 |
/// | Gives          Gets           Stock                  |
/// | [ ] [  0]  ->  [ ] [ 40]      [ ][ ][ ][ ][ ][ ][ ]..|
/// | ... eight offers                                     |
/// |                               Till                   |
/// |                               [ ][ ][ ][ ][ ][ ][ ]..|
/// | [the player's inventory]                             |
/// +------------------------------------------------------+
/// ```
fn setup_screen(ui: &mut Ui) {
    let shown = ui.view().menu_component(&STALL).unwrap_or_default();
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        status_line(ui);
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(0.0), |ui| {
                ui.text_styled(S::column().class("dim"), "plotty.setup.offers", &[]);
                for offer in 0..OFFERS as u32 {
                    offer_row(ui, offer, &shown);
                }
            });
            ui.column(S::column().gap(2.0), |ui| {
                ui.text_styled(S::column().class("dim"), "plotty.setup.stock", &[]);
                ui.group("stock");
                ui.text_styled(S::column().class("dim"), "plotty.setup.till", &[]);
                ui.group("till");
            });
        });
        ui.player_inventory(true);
    });
}

/// One offer in the owner's screen: each side's sample slot and Woof price.
fn offer_row(ui: &mut Ui, offer: u32, shown: &Stall) {
    let prices = shown.prices[offer as usize];
    ui.row(S::row().gap(2.0), |ui| {
        ui.group_slot("gives", offer);
        price_field(ui, offer, 0, prices.gives);
        ui.text_styled(S::column().class("dim"), "plotty.setup.for", &[]);
        ui.group_slot("gets", offer);
        price_field(ui, offer, 1, prices.gets);
    });
}

/// A field for one side's Woof price, sent at every edit; empty or 0 is items.
fn price_field(ui: &mut Ui, offer: u32, gets: u32, now: u32) {
    let shown = if now == 0 {
        String::new()
    } else {
        now.to_string()
    };
    let field = S::default().w(PRICE_WIDTH).tooltip("plotty.setup.price");
    ui.input(&shown, 7, field, move |view, text| {
        let Some(register) = view.menu_owner() else {
            return;
        };
        let woof = text.trim().parse().unwrap_or(0);
        let price = SetPrice {
            register,
            offer,
            gets,
            woof,
        };
        view.client().send(&PRICE, &price);
    });
}

/// Everyone else's screen:
///
/// ```text
/// +--------------------------------------------+
/// | Oak Stall Register                         |
/// | Open                                       |
/// | 16 [bread]  for  3 [iron]    12x  [Trade]  |
/// | 32 [wheat]  for  40 Woof      3x  [Trade]  |
/// | ... eight offers, blank where none         |
/// | [the player's inventory]                   |
/// +--------------------------------------------+
/// ```
fn shop_screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        status_line(ui);
        for offer in 0..OFFERS {
            ui.row(S::row().gap(6.0), |ui| shop_row(ui, offer));
        }
        ui.player_inventory(true);
    });
}

/// One offer in the shop: what the stall gives, what it gets, how often it can, and the button.
fn shop_row(ui: &mut Ui, offer: usize) {
    let goods = || S::column().w(90.0).no_wrap();
    ui.text_bind_styled(goods(), move |view, out| {
        let gives = shown_offer(view, offer).map(|offer| offer.gives);
        say_goods(out, gives);
    });
    ui.text_styled(S::column().class("dim"), "plotty.setup.for", &[]);
    ui.text_bind_styled(goods(), move |view, out| {
        let gets = shown_offer(view, offer).map(|offer| offer.gets);
        say_goods(out, gets);
    });
    ui.text_bind_styled(S::column().w(40.0).no_wrap(), move |view, out| {
        let stall = view.menu_component(&STALL).unwrap_or_default();
        match shown_offer(view, offer) {
            Some(_) => out.set("plotty.shop.left", &[Arg::Int(stall.left[offer].into())]),
            None => out.text(""),
        }
    });
    ui.button("plotty.shop.trade", S::default(), move |view| {
        let (Some(register), Some(stall)) = (view.menu_owner(), view.menu_component(&STALL)) else {
            return;
        };
        let trade = Trade {
            register,
            offer: offer as u32,
            revision: stall.revision,
        };
        view.client().send(&TRADE, &trade);
    });
}

/// Offer `offer` of the stall whose screen is open.
fn shown_offer(view: &ClientView, offer: usize) -> Option<super::offer::Offer> {
    let stall = view.menu_component(&STALL)?;
    stall::offer_of(&stall, offer)
}

/// One side of an offer, as text; nothing where there's no offer.
fn say_goods(out: &mut weft_sdk::TextOut, goods: Option<Goods>) {
    match goods {
        Some(Goods::Items(kind)) => out.set(
            "plotty.shop.items",
            &[Arg::Int(kind.count.into()), Arg::Item(kind.item)],
        ),
        Some(Goods::Woof(price)) => out.set("plotty.shop.woof", &[Arg::Int(price as i64)]),
        None => out.set("plotty.shop.no_offer", &[]),
    }
}

/// Whether the stall trades, and what is missing if a part is.
fn status_line(ui: &mut Ui) {
    ui.text_bind(|view, out| {
        let stall = view.menu_component(&STALL).unwrap_or_default();
        let status = Status::of(&stall);
        let part = PARTS
            .get(stall.missing as usize)
            .map_or(0, |_| stall.missing);
        out.set(status.key(), &[Arg::Key(&format!("plotty.part.{part}"))]);
    });
}

/// Sets a Woof price the owner typed, where the player may and stands near.
fn set_price(packet: &Packet, asked: SetPrice) {
    let world = packet.world();
    let player = packet.player();
    let (Some(register), Some(you)) = (world.entity(asked.register), world.entity(player)) else {
        return;
    };
    let pos = Pos::from(register.block_pos());
    let near = stall::within_reach(register.pos, you.pos);
    if !near || !stall::may_set_up(&world, player, pos) || asked.offer as usize >= OFFERS {
        return;
    }
    let Some(stall) = world.component_mut(asked.register, &STALL) else {
        return;
    };
    let prices = &mut stall.prices[asked.offer as usize];
    if asked.gets == 0 {
        prices.gives = asked.woof;
    } else {
        prices.gets = asked.woof;
    }
    stall::look(&world, asked.register, pos, stall);
}
