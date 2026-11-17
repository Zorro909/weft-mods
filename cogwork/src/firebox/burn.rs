//! How a firebox boils the boiler on it, with no world: each tick of boiling turns 20 mB of the
//! boiler's water into as much steam and burns two ticks of fuel, so a coal that keeps an ember
//! engine going for 1,600 ticks boils for 800. A tick boils only with water, room for its steam
//! and fuel to burn: no fuel burns for nothing, and a tick that stood for missed time is paid for
//! like any other. Fuel left burning when it stops stays for later.

use super::{Fire, Status};

/// Millibuckets of water a tick of boiling takes, and of steam it makes.
pub const BOIL: u32 = 20;
/// Ticks of fuel a tick of boiling burns.
pub const BURN_A_TICK: u32 = 2;
/// Millibuckets of steam a boiler holds.
pub const STEAM_HELD: u32 = 4_000;

/// The boiler on a firebox: millibuckets of water and of steam in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Kettle {
    pub water: u32,
    pub steam: u32,
}

impl Fire {
    /// Boils `kettle` for up to `ticks`, lighting more fuel from `light` (its ticks, or why there
    /// is none) whenever what burns runs short. The ticks it boiled, and what stopped it, if
    /// anything did.
    pub fn boil(
        &mut self,
        ticks: u32,
        kettle: &mut Kettle,
        mut light: impl FnMut() -> Result<u32, Status>,
    ) -> (u32, Option<Status>) {
        for boiled in 0..ticks {
            if let Some(lack) = lacks(kettle) {
                return (boiled, Some(lack));
            }
            while self.burn < BURN_A_TICK {
                match light() {
                    Ok(fuel) => {
                        self.burn += fuel;
                        self.burn_max = self.burn;
                    }
                    Err(lack) => return (boiled, Some(lack)),
                }
            }
            self.burn -= BURN_A_TICK;
            kettle.water -= BOIL;
            kettle.steam += BOIL;
        }
        (ticks, None)
    }
}

/// What keeps `kettle` from boiling a tick, besides fuel.
fn lacks(kettle: &Kettle) -> Option<Status> {
    if kettle.water < BOIL {
        Some(Status::NoWater)
    } else if kettle.steam + BOIL > STEAM_HELD {
        Some(Status::Full)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A boiler full of water with no steam.
    fn filled() -> Kettle {
        Kettle {
            water: 8_000,
            steam: 0,
        }
    }

    #[test]
    fn a_coal_boils_for_800_ticks() {
        let (mut fire, mut kettle) = (Fire::default(), filled());
        let mut coal = 1;
        let (boiled, lack) = fire.boil(1_000, &mut kettle, || {
            if coal == 0 {
                return Err(Status::NoFuel);
            }
            coal -= 1;
            Ok(1_600)
        });
        assert_eq!(boiled, 200, "the boiler fills with steam first");
        assert_eq!(lack, Some(Status::Full), "and stops");
        assert_eq!(kettle.steam, STEAM_HELD, "4,000 mB of steam");
        assert_eq!(kettle.water, 8_000 - STEAM_HELD, "from as much water");
        assert_eq!(fire.burn, 1_600 - 400, "the rest of the coal kept");
    }

    #[test]
    fn missed_time_burns_fuel_tick_for_tick() {
        let (mut fire, mut kettle) = (Fire::default(), filled());
        let mut lit = 0;
        let (boiled, _) = fire.boil(150, &mut kettle, || {
            lit += 1;
            Ok(100)
        });
        assert_eq!(boiled, 150, "all of it");
        assert_eq!(lit, 3, "three fuels of 50 ticks' boiling each");
        assert_eq!(fire.burn, 0, "used up");
    }

    #[test]
    fn nothing_burns_without_water_or_room() {
        let mut fire = Fire::default();
        let never = || -> Result<u32, Status> { panic!("lit for nothing") };
        let mut dry = Kettle {
            water: BOIL - 1,
            steam: 0,
        };
        assert_eq!(fire.boil(10, &mut dry, never), (0, Some(Status::NoWater)));
        let mut full = Kettle {
            water: 8_000,
            steam: STEAM_HELD - BOIL + 1,
        };
        assert_eq!(fire.boil(10, &mut full, never), (0, Some(Status::Full)));
    }

    #[test]
    fn an_odd_tick_of_fuel_waits_for_the_next() {
        let (mut fire, mut kettle) = (Fire::default(), filled());
        fire.burn = 1;
        let (boiled, lack) = fire.boil(5, &mut kettle, || Err(Status::NoFuel));
        assert_eq!(
            (boiled, lack),
            (0, Some(Status::NoFuel)),
            "half a tick boils nothing"
        );
        assert_eq!(fire.burn, 1, "and is kept");
        let (boiled, _) = fire.boil(1, &mut kettle, || Ok(1));
        assert_eq!((boiled, fire.burn), (1, 0), "with the next it does");
    }
}
