//! When the Bit Magazine swaps a quarry's bit, and for which of its own: decided over plain
//! values, so tests reach it.

use super::Status;

/// Uses a bit may have left when the magazine takes it out: its last block, so the quarry never
/// wears it away to nothing and the worn bit comes back to the player.
pub const SWAP_AT: u32 = 1;

/// A bit as the magazine sees it: whether the quarry beside it takes it, and its uses left.
#[derive(Clone, Copy, Debug)]
pub struct Seen {
    pub fits: bool,
    pub left: u32,
}

impl Seen {
    /// Whether the quarry can dig on with it.
    fn serves(self) -> bool {
        self.fits && self.left > SWAP_AT
    }
}

/// What the magazine does for the quarry beside it now.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Swap {
    /// Puts the bit in this slot of the magazine into the quarry's empty bit slot.
    Fit(u32),
    /// Takes the quarry's bit out into the output and puts the bit in this slot in.
    Replace(u32),
}

/// What the magazine does for a quarry with `fitted` in its bit slot (none when empty), holding
/// `fresh` bits (none for a slot without one), with `room` for the fitted bit in its output; and
/// its status. A bit the quarry doesn't take, or that is down to its last use, is swapped for the
/// first of the magazine's that the quarry takes with uses to spare; the bit taken out stays where
/// it is while the output is full, so none is ever lost.
pub fn plan(fitted: Option<Seen>, fresh: &[Option<Seen>], room: bool) -> (Status, Option<Swap>) {
    if fitted.is_some_and(Seen::serves) {
        return (Status::Ready, None);
    }
    let Some(slot) = (0..)
        .zip(fresh)
        .find_map(|(slot, bit)| bit.is_some_and(Seen::serves).then_some(slot))
    else {
        return (Status::NoBit, None);
    };
    match fitted {
        None => (Status::Ready, Some(Swap::Fit(slot))),
        Some(_) if room => (Status::Ready, Some(Swap::Replace(slot))),
        Some(_) => (Status::Full, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: Seen = Seen {
        fits: true,
        left: 300,
    };
    const WORN: Seen = Seen {
        fits: true,
        left: SWAP_AT,
    };
    const WRONG: Seen = Seen {
        fits: false,
        left: 300,
    };

    #[test]
    fn a_bit_with_uses_to_spare_stays() {
        assert_eq!(plan(Some(GOOD), &[Some(GOOD)], true), (Status::Ready, None));
    }

    #[test]
    fn an_empty_slot_gets_the_first_bit_that_fits() {
        let fresh = [None, Some(WRONG), Some(WORN), Some(GOOD)];
        assert_eq!(
            plan(None, &fresh, false),
            (Status::Ready, Some(Swap::Fit(3))),
            "past the empty slot, the bit of another quarry and the worn one"
        );
    }

    #[test]
    fn a_worn_bit_is_swapped_out_at_its_last_use() {
        assert_eq!(
            plan(Some(WORN), &[Some(GOOD)], true),
            (Status::Ready, Some(Swap::Replace(0)))
        );
        let one_more = Seen {
            left: SWAP_AT + 1,
            ..GOOD
        };
        assert_eq!(plan(Some(one_more), &[Some(GOOD)], true).1, None, "not yet");
    }

    #[test]
    fn a_worn_bit_waits_for_room_in_the_output() {
        assert_eq!(plan(Some(WORN), &[Some(GOOD)], false), (Status::Full, None));
    }

    #[test]
    fn only_bits_the_quarry_takes_go_in() {
        assert_eq!(plan(None, &[Some(WRONG)], true), (Status::NoBit, None));
        assert_eq!(
            plan(Some(WRONG), &[Some(GOOD)], true),
            (Status::Ready, Some(Swap::Replace(0))),
            "and one it doesn't take comes out"
        );
    }
}
