//! Who may do what, where: one decision over plain values, which the hooks in `protect.rs` ask.

use crate::settings::Wilderness;

/// What a block is part of, for the player asking.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Zone {
    /// Outside every city and the spawn.
    Wilderness,
    /// The protected circle around the spawn.
    Spawn,
    /// A city's road, plaza or market ground outside the places.
    Public,
    /// A plot nobody has claimed.
    FreePlot,
    /// A plot the player owns or is trusted on.
    OwnPlot,
    /// Someone else's plot.
    OthersPlot,
}

/// What is being done.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    /// Breaking or placing a block.
    Change,
    /// Using a block: a door, a chest, a lever.
    Use,
    /// Opening a market stall's screen.
    UseStall,
    /// One player hurting another.
    PlayerFight,
    /// A player hurting an animal or another creature.
    HurtCreature,
    /// A creature appearing by natural spawning.
    NaturalSpawn,
}

/// Whether `action` may happen in `zone`. `bypass` is a moderator who switched protection off for
/// themselves; it lets them change and use anything, never fight where fighting is off.
pub fn allowed(zone: Zone, action: Action, bypass: bool, wilderness: Wilderness) -> bool {
    let by_player = matches!(action, Action::Change | Action::Use | Action::HurtCreature);
    if bypass && by_player {
        return true;
    }
    match zone {
        Zone::Wilderness => wild(action, wilderness),
        Zone::Spawn | Zone::Public => matches!(action, Action::Use | Action::UseStall),
        Zone::FreePlot | Zone::OthersPlot => action == Action::UseStall,
        Zone::OwnPlot => !matches!(action, Action::PlayerFight | Action::NaturalSpawn),
    }
}

/// Outside towns everything goes, except changing blocks where the wilderness is closed.
fn wild(action: Action, wilderness: Wilderness) -> bool {
    action != Action::Change || wilderness == Wilderness::Open
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPEN: Wilderness = Wilderness::Open;

    fn may(zone: Zone, action: Action) -> bool {
        allowed(zone, action, false, OPEN)
    }

    #[test]
    fn the_wilderness_is_open_unless_closed() {
        assert!(may(Zone::Wilderness, Action::Change), "open: anyone builds");
        assert!(
            !allowed(Zone::Wilderness, Action::Change, false, Wilderness::Closed),
            "closed: nobody"
        );
        assert!(
            may(Zone::Wilderness, Action::PlayerFight),
            "fights happen outside towns"
        );
    }

    #[test]
    fn public_ground_is_for_walking_and_using() {
        for zone in [Zone::Spawn, Zone::Public] {
            assert!(!may(zone, Action::Change), "{zone:?}: no building");
            assert!(may(zone, Action::Use), "{zone:?}: doors work");
            assert!(!may(zone, Action::PlayerFight), "{zone:?}: no fights");
            assert!(!may(zone, Action::NaturalSpawn), "{zone:?}: nothing spawns");
        }
    }

    #[test]
    fn a_plot_is_its_owners() {
        assert!(may(Zone::OwnPlot, Action::Change), "the owner builds");
        assert!(!may(Zone::OthersPlot, Action::Change), "others don't");
        assert!(!may(Zone::OthersPlot, Action::Use), "nor open its chests");
        assert!(
            may(Zone::OthersPlot, Action::UseStall),
            "but anyone trades at its stall"
        );
        assert!(
            !may(Zone::FreePlot, Action::Change),
            "a free plot waits for its claim"
        );
    }

    #[test]
    fn bypass_lets_a_moderator_change_but_not_fight() {
        assert!(
            allowed(Zone::OthersPlot, Action::Change, true, OPEN),
            "changes anything"
        );
        assert!(
            !allowed(Zone::Spawn, Action::PlayerFight, true, OPEN),
            "fighting stays off"
        );
    }
}
