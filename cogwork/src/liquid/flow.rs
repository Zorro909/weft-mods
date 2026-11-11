//! How much liquid goes from one holder to another: liquid runs from where it stands or pushes
//! higher to where it stands lower, as between vessels joined at the bottom. A holder's head is
//! the height its liquid reaches, in millionths of a block (`UNIT`): its surface, or, when it is
//! full, the head of what pushes into it from beside, less `LOSS`. A pump holding liquid pushes
//! `LIFT` blocks above itself.

/// Millionths of a block: what heads are measured in.
pub const UNIT: i64 = 1_000_000;
/// How high above itself a pump holding liquid pushes it, in blocks.
pub const LIFT: i64 = 16;
/// The head lost from one full holder to the next: a block in sixteen. Pressure that nothing
/// keeps up so dies away, also in a ring of full tubes.
pub const LOSS: i64 = UNIT / 16;

/// Where a holder's liquid stands or pushes to, and how many millibuckets raise it by a block.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Level {
    pub head: i64,
    pub per_block: u32,
}

/// The head of `amount` millibuckets in a holder whose floor is `floor` blocks up, `per_block`
/// of them to a block of height.
pub fn surface(floor: i32, amount: u32, per_block: u32) -> i64 {
    floor as i64 * UNIT + amount as i64 * UNIT / per_block.max(1) as i64
}

/// Millibuckets that go from `from` to `to` in one turn: as many as leave their heads even,
/// rounded down so two holders a little apart don't pass it to and fro, and at most `max`. None
/// when `to` stands as high as `from` or higher.
pub fn flow(from: Level, to: Level, max: u32) -> u32 {
    let difference = from.head - to.head;
    if difference <= 0 {
        return 0;
    }
    // Each millibucket lowers `from` by UNIT / from.per_block and raises `to` by UNIT / to.per_block.
    let (a, b) = (from.per_block as i128, to.per_block as i128);
    let even = difference as i128 * a * b / (UNIT as i128 * (a + b)).max(1);
    even.min(max as i128) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tube(head: i64) -> Level {
        Level {
            head,
            per_block: 1000,
        }
    }

    #[test]
    fn two_tubes_side_by_side_level_out() {
        let half_way = flow(tube(800_000), tube(200_000), 1000);
        assert_eq!(half_way, 300, "half the difference each way");
        assert_eq!(flow(tube(200_000), tube(800_000), 1000), 0, "never uphill");
    }

    #[test]
    fn liquid_falls_into_the_tube_below() {
        // A tube a quarter full above an empty one: its surface stands a block and a quarter
        // above the other's floor.
        let above = surface(1, 250, 1000);
        let below = surface(0, 0, 1000);
        assert_eq!(flow(tube(above), tube(below), 100), 100, "the most a turn");
    }

    #[test]
    fn a_bigger_holder_takes_more_to_stand_as_high() {
        let tank = Level {
            head: 0,
            per_block: 3000,
        };
        assert_eq!(flow(tube(UNIT), tank, 1000), 750, "both a quarter up after");
    }

    #[test]
    fn a_last_millibucket_stays_put() {
        assert_eq!(flow(tube(1000), tube(0), 1000), 0, "no to and fro");
        assert_eq!(flow(tube(2000), tube(0), 1000), 1, "two even out");
    }

    #[test]
    fn a_pump_lifts_liquid_its_lift_and_no_higher() {
        let pump = LIFT * UNIT;
        let top_of_reach = surface(LIFT as i32 - 1, 0, 1000);
        assert!(
            flow(tube(pump - LOSS), tube(top_of_reach), 100) > 0,
            "still up"
        );
        let beyond = surface(LIFT as i32, 0, 1000);
        assert_eq!(flow(tube(pump - LOSS), tube(beyond), 100), 0, "too high");
    }
}
