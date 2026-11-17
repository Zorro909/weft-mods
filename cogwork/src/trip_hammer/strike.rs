//! How a trip hammer works a bloom, with no world: a strike every `STRIKE_TICKS` while it has a
//! bloom, 10 mB of steam for the strike and room for the ingot in its output; the fifth strike
//! forges the bloom into a steel ingot. It takes one step a call, so time missed while its area
//! was away is lost, never struck for nothing.

use super::{Hammering, Status};

/// Strikes that forge a bloom into an ingot, as on the forge anvil.
pub const STRIKES: u32 = crate::forge_anvil::STRIKES;
/// Ticks from one strike to the next, and millibuckets of steam a strike takes.
pub const STRIKE_TICKS: u32 = 10;
pub const STEAM_A_STRIKE: u32 = 10;
/// Ticks before a strike that the hammer starts to fall: its clip's fall.
pub const FALL_TICKS: u32 = 2;

/// What a step of the hammer did.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    /// Waited, for the reason its status gives.
    Waited,
    /// Lifted the hammer a tick further.
    Lifted,
    /// Let the hammer fall: it strikes in `FALL_TICKS`.
    Fell,
    Struck,
    /// Struck the last time: an ingot is done.
    Forged,
}

impl Hammering {
    /// A tick's step with `bloom` saying whether a bloom waits in the input, `steam` the
    /// millibuckets it holds and `room` whether an ingot fits in the output.
    pub fn step(&mut self, bloom: bool, steam: u32, room: bool) -> Step {
        let status = match () {
            _ if !bloom => Status::Idle,
            _ if steam < STEAM_A_STRIKE => Status::NoSteam,
            _ if !room => Status::Full,
            _ => Status::Striking,
        };
        self.set_status(status);
        if status != Status::Striking {
            if !bloom {
                (self.strikes, self.timer) = (0, 0);
            }
            return Step::Waited;
        }
        self.timer += 1;
        if self.timer < STRIKE_TICKS {
            return if self.timer == STRIKE_TICKS - FALL_TICKS {
                Step::Fell
            } else {
                Step::Lifted
            };
        }
        self.timer = 0;
        self.strikes += 1;
        if self.strikes < STRIKES {
            return Step::Struck;
        }
        self.strikes = 0;
        Step::Forged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Steps until something other than lifting happens; the steps taken and what happened.
    fn until_struck(hammering: &mut Hammering) -> (u32, Step) {
        for ticks in 1..=STRIKE_TICKS {
            match hammering.step(true, 100, true) {
                Step::Lifted | Step::Fell => {}
                step => return (ticks, step),
            }
        }
        panic!("never struck")
    }

    #[test]
    fn five_strikes_forge_a_bloom() {
        let mut hammering = Hammering::default();
        for strike in 1..STRIKES {
            assert_eq!(
                until_struck(&mut hammering),
                (STRIKE_TICKS, Step::Struck),
                "{strike}"
            );
        }
        assert_eq!(until_struck(&mut hammering).1, Step::Forged, "the fifth");
        assert_eq!(hammering.strikes, 0, "the next bloom starts over");
    }

    #[test]
    fn the_hammer_falls_just_before_it_strikes() {
        let mut hammering = Hammering::default();
        let steps: Vec<Step> = (0..STRIKE_TICKS)
            .map(|_| hammering.step(true, 100, true))
            .collect();
        let fell = steps.iter().position(|&step| step == Step::Fell);
        assert_eq!(
            fell,
            Some((STRIKE_TICKS - FALL_TICKS - 1) as usize),
            "two ticks early"
        );
        assert_eq!(steps.last(), Some(&Step::Struck), "then strikes");
    }

    #[test]
    fn it_waits_without_steam_or_room_and_keeps_its_strikes() {
        let mut hammering = Hammering::default();
        until_struck(&mut hammering);
        let step = hammering.step(true, STEAM_A_STRIKE - 1, true);
        assert_eq!((step, hammering.status()), (Step::Waited, Status::NoSteam));
        let step = hammering.step(true, 100, false);
        assert_eq!((step, hammering.status()), (Step::Waited, Status::Full));
        assert_eq!(hammering.strikes, 1, "the strike kept");
    }

    #[test]
    fn a_bloom_taken_out_starts_the_next_over() {
        let mut hammering = Hammering::default();
        until_struck(&mut hammering);
        assert_eq!(hammering.step(false, 100, true), Step::Waited, "no bloom");
        assert_eq!(hammering.status(), Status::Idle, "waiting for one");
        assert_eq!((hammering.strikes, hammering.timer), (0, 0), "started over");
    }
}
