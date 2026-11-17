//! A machine's own pipes: liquid it keeps apart from its `HELD` handed straight into the holder
//! beside it, or drawn straight out of one, as a boiler lets out its steam. Only whole
//! millibuckets move, and only what the other side has or has room for, so none is made or lost.

use weft_sdk::{Dir, World};

use super::LiquidKey;
use super::vessel::{Holders, Kind, Vessel, reaches};
use crate::lib::Pos;

/// Hands up to `most` of `liquid` from the machine at `from` into the holder toward `side`, where
/// it reaches back and takes that liquid (`Held::takes`); the millibuckets it took. A pump never
/// takes.
pub fn pour(world: &World, from: Pos, side: Dir, liquid: LiquidKey, most: u32) -> u32 {
    let at = from.toward(side);
    let Some(into) = Vessel::at(world, at).filter(|_| reaches(world, at, side.opposite())) else {
        return 0;
    };
    let mut holders = Holders::outside(*world);
    let has = into.held(&holders);
    if into.kind().pushes() || !into.admits(liquid) || !has.takes(liquid, into.capacity()) {
        return 0;
    }
    let had = if has.liquid == liquid { has.amount } else { 0 };
    let amount = most.min(into.capacity().saturating_sub(had));
    if amount > 0 {
        into.put(&mut holders, liquid, amount);
    }
    amount
}

/// Draws up to `most` of `liquid` into the machine at `into` out of the holder toward `side`: a
/// liquid tube, a tank, a coke oven's door or drain or a refinery's tap that reaches back and holds that liquid.
/// The millibuckets it gave.
pub fn draw(world: &World, into: Pos, side: Dir, liquid: LiquidKey, most: u32) -> u32 {
    let at = into.toward(side);
    let Some(from) = Vessel::at(world, at).filter(|_| reaches(world, at, side.opposite())) else {
        return 0;
    };
    if !gives_up(from.kind()) {
        return 0;
    }
    let mut holders = Holders::outside(*world);
    let has = from.held(&holders);
    if has.liquid != liquid {
        return 0;
    }
    let amount = most.min(has.amount);
    if amount > 0 {
        from.take(&mut holders, amount);
    }
    amount
}

/// Whether a holder of `kind` lets a machine draw on it: one that hands its liquid on anyway, not
/// a pump, which pushes, nor a machine keeping what it holds for its own work.
fn gives_up(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Tube | Kind::Sealed | Kind::Pressure | Kind::Drain
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_what_hands_liquid_on_is_drawn_on() {
        assert!(gives_up(Kind::Tube), "a tube or a tank");
        assert!(
            gives_up(Kind::Sealed) && gives_up(Kind::Pressure),
            "any tube"
        );
        assert!(gives_up(Kind::Drain), "a coke oven's drain");
        for kind in [
            Kind::Pump,
            Kind::Driven,
            Kind::Intake,
            Kind::Mill,
            Kind::Firebox,
            Kind::Steam,
            Kind::Settler,
            Kind::Still,
            Kind::Lubed,
            Kind::Polymer,
        ] {
            assert!(!gives_up(kind), "{kind:?} keeps what it holds");
        }
    }
}
