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
    /// A market place the player rents, for a stall.
    OwnPlace,
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
    /// Placing or breaking a block of a market stall.
    BuildStall,
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
    let by_player = matches!(
        action,
        Action::Change | Action::Use | Action::BuildStall | Action::HurtCreature
    );
    if bypass && by_player {
        return true;
    }
    match zone {
        Zone::Wilderness => wild(action, wilderness),
        Zone::Spawn | Zone::Public => matches!(action, Action::Use | Action::UseStall),
        Zone::FreePlot | Zone::OthersPlot => action == Action::UseStall,
        Zone::OwnPlot => !matches!(action, Action::PlayerFight | Action::NaturalSpawn),
        Zone::OwnPlace => matches!(action, Action::Use | Action::UseStall | Action::BuildStall),
    }
}

/// Outside towns everything goes, except changing blocks when wilderness is closed, and stalls,
/// which only go on plots or rented market places.
fn wild(action: Action, wilderness: Wilderness) -> bool {
    match action {
        Action::Change => wilderness == Wilderness::Open,
        Action::BuildStall => false,
        _ => true,
    }
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
    fn stalls_stand_on_their_owners_plots_only() {
        assert!(may(Zone::OwnPlot, Action::BuildStall), "your plot");
        assert!(may(Zone::OwnPlace, Action::BuildStall), "your market place");
        for zone in [
            Zone::Wilderness,
            Zone::Spawn,
            Zone::Public,
            Zone::FreePlot,
            Zone::OthersPlot,
        ] {
            assert!(!may(zone, Action::BuildStall), "{zone:?}");
        }
        assert!(
            may(Zone::Wilderness, Action::UseStall),
            "trading goes anywhere"
        );
    }

    #[test]
    fn a_rented_place_takes_a_stall_and_nothing_else() {
        assert!(may(Zone::OwnPlace, Action::BuildStall), "a stall");
        assert!(!may(Zone::OwnPlace, Action::Change), "no other building");
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
