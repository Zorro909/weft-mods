//! How a steam engine runs, with no world: 20 mB of steam a tick for 160 energy, four times
//! what an ember engine makes. It runs a tick only with that steam and room for what it makes,
//! so no steam goes through it for nothing; a call that stands for missed time runs as many
//! ticks as its steam and room allow, and no more. Like a governor, once its store is full it
//! rests until half of it has gone out, so that a light load sees it run in steady spells
//! rather than a tick now and then.

use super::{Status, SteamEngine};

/// Energy it makes a tick, and millibuckets of steam that takes.
pub const MAKES: i64 = 160;
pub const STEAM_A_TICK: u32 = 20;
/// Energy it holds: 100 ticks of what it makes.
pub const ENERGY_STORED: i64 = 100 * MAKES;
/// Energy it holds at most to start again once its store was full.
const RESTARTS_AT: i64 = ENERGY_STORED / 2;

impl SteamEngine {
    /// Runs it up to `ticks` with `steam` millibuckets in and `stored` energy held. The ticks it
    /// ran, and its status after: `Full` from when its store has no room for another tick until
    /// half of it is out again, whether or not steam comes in meanwhile.
    pub fn run(&self, ticks: u32, steam: u32, stored: i64) -> (u32, Status) {
        if self.status() == Status::Full && stored > RESTARTS_AT {
            return (0, Status::Full);
        }
        let by_steam = steam / STEAM_A_TICK;
        let by_room = ((ENERGY_STORED - stored).max(0) / MAKES) as u32;
        let ran = ticks.min(by_steam).min(by_room);
        let status = match ran {
            _ if ran == by_room => Status::Full,
            0 => Status::NoSteam,
            _ => Status::Running,
        };
        (ran, status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn running() -> SteamEngine {
        let mut engine = SteamEngine::default();
        engine.set_status(Status::Running);
        engine
    }

    #[test]
    fn it_runs_a_tick_on_twenty_millibuckets() {
        let engine = running();
        assert_eq!(
            engine.run(10, 1_000, 0),
            (10, Status::Running),
            "steam for 50"
        );
        assert_eq!(
            engine.run(10, 99, 0),
            (4, Status::Running),
            "four ticks' steam"
        );
        assert_eq!(
            engine.run(1, STEAM_A_TICK - 1, 0),
            (0, Status::NoSteam),
            "not a tick's"
        );
    }

    #[test]
    fn it_stops_when_its_energy_has_nowhere_to_go() {
        let engine = running();
        let nearly = ENERGY_STORED - 2 * MAKES;
        assert_eq!(
            engine.run(10, 1_000, nearly),
            (2, Status::Full),
            "room for two"
        );
        assert_eq!(
            engine.run(10, 0, ENERGY_STORED),
            (0, Status::Full),
            "full, with no steam either"
        );
    }

    #[test]
    fn once_full_it_rests_until_half_its_energy_is_out() {
        let mut engine = running();
        engine.set_status(Status::Full);
        assert_eq!(
            engine.run(10, 1_000, RESTARTS_AT + 1),
            (0, Status::Full),
            "still resting"
        );
        assert_eq!(
            engine.run(10, 1_000, RESTARTS_AT),
            (10, Status::Running),
            "off again"
        );
    }

    #[test]
    fn filling_its_store_rests_it_through_a_lack_of_steam() {
        let mut engine = running();
        let (ran, status) = engine.run(1, STEAM_A_TICK, ENERGY_STORED - MAKES);
        assert_eq!((ran, status), (1, Status::Full), "the tick that fills it");
        engine.set_status(status);
        // A load drawing 20 energy a tick, with steam coming now and then.
        let drawn = (RESTARTS_AT + 1..ENERGY_STORED).rev().step_by(20);
        for stored in drawn {
            let (_, starved) = engine.run(1, 0, stored);
            assert_eq!(starved, Status::Full, "resting without steam at {stored}");
            engine.set_status(starved);
            assert_eq!(
                engine.run(1, STEAM_A_TICK, stored),
                (0, Status::Full),
                "steam back at {stored}, before half is out"
            );
        }
        let at_half = engine.run(1, STEAM_A_TICK, RESTARTS_AT);
        assert_eq!(at_half, (1, Status::Running), "off again at half");
    }

    #[test]
    fn missed_time_runs_only_on_steam_it_has() {
        let (ran, _) = running().run(6_000, 2_000, 0);
        assert_eq!(ran, 100, "2,000 mB is 100 ticks, however long it was away");
    }
}
