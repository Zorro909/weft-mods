//! One kind of offer for both directions: the stall gives something and gets something for it,
//! each side items or Golden Woof. Where each side comes from and goes to follows from that alone:
//! items the stall gives come out of its stock and items it gets go into its till; Woof it gives
//! comes from its owner's balance and Woof it gets goes there. So selling and buying are one
//! trade, and how often an offer can go through is one sum.

use weft_sdk::ItemStack;

use super::stacks;

/// Offers a stall has.
pub const OFFERS: usize = 8;

/// One side of an offer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Goods {
    /// A stack's worth of items: its kind and its count.
    Items(ItemStack),
    Woof(u64),
}

/// What a stall gives, and what it gets for it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Offer {
    pub gives: Goods,
    pub gets: Goods,
}

/// Why a trade can't go through, by whose side falls short.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Short {
    /// The stall's stock has too few of what it gives.
    Stock,
    /// The stall's till has no room for what it gets.
    Till,
    /// The owner's balance is lower than the Woof the stall gives.
    OwnerWoof,
    /// The customer has too few of what the stall gets.
    YourItems,
    /// The customer's inventory has no room for what the stall gives.
    YourRoom,
    /// The customer's balance is lower than the Woof the stall gets.
    YourWoof,
}

impl Short {
    /// The lang key saying so.
    pub fn key(self) -> &'static str {
        match self {
            Short::Stock => "plotty.trade.stock",
            Short::Till => "plotty.trade.till",
            Short::OwnerWoof => "plotty.trade.owner_woof",
            Short::YourItems => "plotty.trade.your_items",
            Short::YourRoom => "plotty.trade.your_room",
            Short::YourWoof => "plotty.trade.your_woof",
        }
    }
}

/// What both ends of a trade hold, as far as an offer needs: the stall's stock and till, its
/// owner's balance, and the customer's inventory and balance.
pub struct Holdings<'a> {
    pub stock: &'a [ItemStack],
    pub till: &'a [ItemStack],
    pub owner_woof: u64,
    pub yours: &'a [ItemStack],
    pub your_woof: u64,
    /// The most a stack of an item holds.
    pub max_stack: fn(ItemStack) -> u32,
}

impl Offer {
    /// The offer set up from its two sample stacks and Woof prices: a side with a price is
    /// Woof, else its sample's items. `None` while a side is empty, and for Woof for Woof.
    pub fn from(samples: [ItemStack; 2], woof: [u32; 2]) -> Option<Offer> {
        let side = |sample: ItemStack, woof: u32| match (woof, sample.is_empty()) {
            (0, true) => None,
            (0, false) => Some(Goods::Items(sample)),
            (woof, _) => Some(Goods::Woof(woof.into())),
        };
        let offer = Offer {
            gives: side(samples[0], woof[0])?,
            gets: side(samples[1], woof[1])?,
        };
        let woof_for_woof = matches!((offer.gives, offer.gets), (Goods::Woof(_), Goods::Woof(_)));
        (!woof_for_woof).then_some(offer)
    }

    /// How many times the stall's side allows the offer: its stock, till and owner's balance.
    pub fn times_for_stall(
        &self,
        stock: &[ItemStack],
        till: &[ItemStack],
        owner_woof: u64,
        max_stack: fn(ItemStack) -> u32,
    ) -> u64 {
        let gives = match self.gives {
            Goods::Items(kind) => u64::from(stacks::count(stock, kind) / kind.count),
            Goods::Woof(price) => owner_woof / price,
        };
        let gets = match self.gets {
            Goods::Items(kind) => u64::from(stacks::room(till, kind, max_stack(kind)) / kind.count),
            Goods::Woof(_) => u64::MAX,
        };
        gives.min(gets)
    }

    /// Whether the offer can go through once; else whose side falls short.
    pub fn check(&self, held: &Holdings) -> Result<(), Short> {
        match self.gives_short(held).or(self.gets_short(held)) {
            Some(short) => Err(short),
            None => Ok(()),
        }
    }

    /// What keeps the stall from giving its side once: from its stock into the customer's
    /// inventory, or from its owner's balance.
    fn gives_short(&self, held: &Holdings) -> Option<Short> {
        match self.gives {
            Goods::Items(kind) => (stacks::count(held.stock, kind) < kind.count)
                .then_some(Short::Stock)
                .or((self.your_room_after_paying(held, kind) < kind.count)
                    .then_some(Short::YourRoom)),
            Goods::Woof(price) => (held.owner_woof < price).then_some(Short::OwnerWoof),
        }
    }

    /// Room for `kind` in the customer's inventory once they paid: paying may empty a slot.
    fn your_room_after_paying(&self, held: &Holdings, kind: ItemStack) -> u32 {
        let mut yours = held.yours.to_vec();
        if let Goods::Items(paid) = self.gets {
            stacks::take(&mut yours, paid, paid.count);
        }
        stacks::room(&yours, kind, (held.max_stack)(kind))
    }

    /// What keeps the customer from paying the stall's price once: from their inventory into
    /// its till, or from their balance.
    fn gets_short(&self, held: &Holdings) -> Option<Short> {
        match self.gets {
            Goods::Items(kind) => (stacks::count(held.yours, kind) < kind.count)
                .then_some(Short::YourItems)
                .or(
                    (stacks::room(held.till, kind, (held.max_stack)(kind)) < kind.count)
                        .then_some(Short::Till),
                ),
            Goods::Woof(price) => (held.your_woof < price).then_some(Short::YourWoof),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BREAD: ItemStack = ItemStack {
        item: 7,
        count: 16,
        data: 0,
    };
    const IRON: ItemStack = ItemStack {
        item: 9,
        count: 3,
        data: 0,
    };

    fn of(kind: ItemStack, count: u32) -> ItemStack {
        ItemStack { count, ..kind }
    }

    fn held<'a>(stock: &'a [ItemStack], yours: &'a [ItemStack]) -> Holdings<'a> {
        Holdings {
            stock,
            till: &[ItemStack::EMPTY; 3],
            owner_woof: 100,
            yours,
            your_woof: 50,
            max_stack: |_| 64,
        }
    }

    #[test]
    fn a_side_with_a_price_is_woof() {
        let offer = Offer::from([BREAD, ItemStack::EMPTY], [0, 40]);
        assert_eq!(
            offer,
            Some(Offer {
                gives: Goods::Items(BREAD),
                gets: Goods::Woof(40)
            }),
            "sell bread for Woof"
        );
    }

    #[test]
    fn empty_sides_and_woof_for_woof_are_no_offer() {
        assert_eq!(
            Offer::from([BREAD, ItemStack::EMPTY], [0, 0]),
            None,
            "nothing asked"
        );
        assert_eq!(
            Offer::from([ItemStack::EMPTY; 2], [5, 6]),
            None,
            "Woof for Woof"
        );
    }

    #[test]
    fn a_sale_needs_stock_and_the_customers_payment() {
        let offer = Offer::from([BREAD, IRON], [0, 0]).expect("an offer");
        let stock = [of(BREAD, 20)];
        assert_eq!(
            offer.check(&held(&stock, &[of(IRON, 3), ItemStack::EMPTY])),
            Ok(()),
            "enough"
        );
        assert_eq!(
            offer.check(&held(&[of(BREAD, 15)], &[of(IRON, 3), ItemStack::EMPTY])),
            Err(Short::Stock),
            "15 of 16"
        );
        assert_eq!(
            offer.check(&held(&stock, &[of(IRON, 2), ItemStack::EMPTY])),
            Err(Short::YourItems),
            "2 of 3 iron"
        );
        assert_eq!(
            offer.check(&held(&stock, &[of(IRON, 4)])),
            Err(Short::YourRoom),
            "no room for bread"
        );
        assert_eq!(
            offer.check(&held(&stock, &[of(IRON, 3)])),
            Ok(()),
            "paying empties the slot the bread goes into"
        );
    }

    #[test]
    fn a_purchase_needs_the_owners_woof() {
        let offer = Offer::from([ItemStack::EMPTY, IRON], [120, 0]).expect("buy iron for Woof");
        assert_eq!(
            offer.check(&held(&[], &[of(IRON, 3)])),
            Err(Short::OwnerWoof),
            "the owner has 100"
        );
    }

    #[test]
    fn the_stall_side_counts_how_often() {
        let offer = Offer::from([BREAD, ItemStack::EMPTY], [0, 1]).expect("bread for Woof");
        assert_eq!(
            offer.times_for_stall(&[of(BREAD, 64), of(BREAD, 20)], &[], 0, |_| 64),
            5,
            "84 bread, 16 a time"
        );
        let buying = Offer::from([ItemStack::EMPTY, IRON], [10, 0]).expect("buy iron");
        assert_eq!(
            buying.times_for_stall(&[], &[ItemStack::EMPTY], 35, |_| 64),
            3,
            "35 Woof, 10 a time"
        );
    }
}
