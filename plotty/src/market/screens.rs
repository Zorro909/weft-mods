//! A stall's two screens. Its owner's sets it up: per offer a sample of what it gives and gets,
//! each side with a Woof price that makes it Woof instead, then the stock and the till. Everyone
//! else's shops: the eight offers as text, how often each can still go through, and a button
//! that trades once (`trade.rs`). The shop shows no slots of the stall's, so nobody but its owner
//! can reach its items.

use weft_sdk::{Arg, Channel, ClientView, Menu, Packet, Registry, S, Slots, Ui, value};
use weftkit::{PlayerKey, Pos};

use super::offer::{Goods, OFFERS};
use super::owner::{self, owner_at};
use super::stall::{self, PRICES, STALL, STOCK, Stall, Status};
use super::trade::{TRADE, Trade};

pub static SETUP: Menu = Menu::named("stall_setup");
pub static SHOP: Menu = Menu::named("stall_shop");

/// Width of a price field, in GUI pixels.
const PRICE_WIDTH: f32 = 30.0;
/// Offers each screen shows at once; the others are a scroll away.
const SETUP_ROWS: f32 = 5.0;
const SHOP_ROWS: f32 = 4.0;
/// Height of an offer's row in GUI pixels: a slot in the owner's screen, a button and the gap
/// under it in the shop.
const SETUP_ROW: f32 = 18.0;
const SHOP_ROW: f32 = 24.0;

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

/// The owner's screen, its offers scrolling five at a time:
///
/// ```text
/// +----------------------------------------------------------------------+
/// | Oak Stall Register                                                   |
/// | Open                                                                 |
/// | Offers: gives, for gets   Stock                Till                  |
/// | [ ] [  0] for [ ] [ 40] | [ ][ ][ ][ ][ ][ ].. [ ][ ][ ][ ][ ][ ]..  |
/// | ... five of eight       |                                            |
/// | [the player's inventory]                                             |
/// +----------------------------------------------------------------------+
/// ```
fn setup_screen(ui: &mut Ui) {
    let shown = ui.view().menu_component(&STALL).unwrap_or_default();
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        status_line(ui);
        ui.row(S::row().gap(10.0).stretch(), |ui| {
            ui.column(S::column().gap(2.0), |ui| {
                ui.text_styled(S::column().class("dim"), "plotty.setup.offers", &[]);
                let rows = S::column().h(SETUP_ROWS * SETUP_ROW).scroll();
                ui.column(rows, |ui| {
                    for offer in 0..OFFERS as u32 {
                        offer_row(ui, offer, &shown);
                    }
                });
            });
            for (label, group) in [
                ("plotty.setup.stock", "stock"),
                ("plotty.setup.till", "till"),
            ] {
                ui.column(S::column().gap(2.0), |ui| {
                    ui.text_styled(S::column().class("dim"), label, &[]);
                    ui.group(group);
                });
            }
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

/// Everyone else's screen, its offers scrolling four at a time:
///
/// ```text
/// +-----------------------------------------------+
/// | Oak Stall Register                            |
/// | Open                                          |
/// | 16 [bread]  for  3 [iron]   12 left  [Trade]  |
/// | 32 [wheat]  for  40 Woof     3 left  [Trade]  |
/// | ... four of eight, a dash where none          |
/// | [the player's inventory]                      |
/// +-----------------------------------------------+
/// ```
fn shop_screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        status_line(ui);
        let rows = S::column().gap(4.0).h(SHOP_ROWS * SHOP_ROW).scroll();
        ui.column(rows, |ui| {
            for offer in 0..OFFERS {
                ui.row(S::row().gap(6.0), |ui| shop_row(ui, offer));
            }
        });
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
        let part = stall::missing(&stall).key();
        out.set(Status::of(&stall).key(), &[Arg::Key(part)]);
    });
}

/// Sets a Woof price the owner typed, where they stand near.
fn set_price(packet: &Packet, asked: SetPrice) {
    let world = packet.world();
    let player = packet.player();
    let (Some(register), Some(you)) = (world.entity(asked.register), world.entity(player)) else {
        return;
    };
    let pos = Pos::from(register.block_pos());
    let near = stall::within_reach(register.pos, you.pos);
    let mine = owner::is_owner(
        owner_at(&world, pos).as_ref(),
        PlayerKey::of(&world, player).as_ref(),
    );
    if !near || !mine || asked.offer as usize >= OFFERS {
        return;
    }
    let Some(saved) = world.component_mut(asked.register, &PRICES) else {
        return;
    };
    let prices = &mut saved.offers[asked.offer as usize];
    if asked.gets == 0 {
        prices.gives = asked.woof;
    } else {
        prices.gets = asked.woof;
    }
    if let Some(stall) = world.component_mut(asked.register, &STALL) {
        stall::look(&world, asked.register, pos, stall);
    }
}
