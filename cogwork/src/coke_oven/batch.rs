//! How a coke oven cooks, with no world: up to eight coal at once, in 900 ticks, into as much
//! coke, 250 mB of creosote each and a sulfur every eighth coal. The coal of a batch stays in
//! the input until the batch is done and all it gives has room, so nothing is lost to a full
//! output; time missed while its area was away is cooked through, batch after batch.

use super::{Coking, Status};

/// Ticks a batch cooks.
pub const COKE_TICKS: u32 = 900;
/// The most coal a batch holds.
pub const MOST_AT_ONCE: u32 = 8;
/// Millibuckets of creosote a coal gives.
pub const CREOSOTE_PER_COAL: u32 = 250;
/// Coal coked for each sulfur.
pub const COAL_PER_SULFUR: u32 = 8;

/// What a finished batch takes and gives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Done {
    /// Coal it takes out of the input, and coke it gives: one for one.
    pub coal: u32,
    pub sulfur: u32,
    /// Millibuckets.
    pub creosote: u32,
}

impl Coking {
    /// Cooks for `ticks` with `coal` waiting in the input; `finish` puts each batch that is done
    /// away, or says what it lacks room for and leaves it waiting.
    pub fn cook(
        &mut self,
        mut ticks: u32,
        mut coal: u32,
        mut finish: impl FnMut(Done) -> Result<(), Status>,
    ) -> Status {
        loop {
            if !self.charge_from(coal) {
                return Status::Idle;
            }
            let step = ticks.min(COKE_TICKS - self.cooked.min(COKE_TICKS));
            self.cooked += step;
            ticks -= step;
            if self.cooked < COKE_TICKS {
                return Status::Coking;
            }
            let done = self.done();
            if let Err(lack) = finish(done) {
                return lack;
            }
            coal -= done.coal;
            self.toward_sulfur = (self.toward_sulfur + done.coal) % COAL_PER_SULFUR;
            (self.charge, self.cooked) = (0, 0);
            if ticks == 0 {
                return if coal > 0 {
                    Status::Coking
                } else {
                    Status::Idle
                };
            }
        }
    }

    /// Makes its batch fit the `coal` in the input, starting one when it has none; whether it
    /// has a batch. A batch whose coal was all taken out starts over.
    fn charge_from(&mut self, coal: u32) -> bool {
        self.charge = self.charge.min(coal);
        if self.charge == 0 {
            self.cooked = 0;
            self.charge = coal.min(MOST_AT_ONCE);
        }
        self.charge > 0
    }

    /// What its batch gives once cooked.
    fn done(&self) -> Done {
        Done {
            coal: self.charge,
            sulfur: (self.toward_sulfur + self.charge) / COAL_PER_SULFUR,
            creosote: self.charge * CREOSOTE_PER_COAL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Finishes every batch and keeps what it gave.
    fn cooked_all(coking: &mut Coking, ticks: u32, coal: u32) -> (Status, Vec<Done>) {
        let mut given = Vec::new();
        let status = coking.cook(ticks, coal, |done| {
            given.push(done);
            Ok(())
        });
        (status, given)
    }

    #[test]
    fn eight_coal_cook_at_once_in_900_ticks() {
        let mut coking = Coking::default();
        let (status, given) = cooked_all(&mut coking, COKE_TICKS - 1, 20);
        assert_eq!(status, Status::Coking, "a tick short");
        assert!(given.is_empty(), "nothing yet");
        assert_eq!(coking.charge, 8, "eight of the twenty");
        let (_, given) = cooked_all(&mut coking, 1, 20);
        let done = Done {
            coal: 8,
            sulfur: 1,
            creosote: 2_000,
        };
        assert_eq!(given, [done], "eight coke, a sulfur, 2,000 mB");
    }

    #[test]
    fn missed_time_cooks_batch_after_batch() {
        let mut coking = Coking::default();
        let (status, given) = cooked_all(&mut coking, 10 * COKE_TICKS, 20);
        let coal: Vec<u32> = given.iter().map(|done| done.coal).collect();
        assert_eq!(coal, [8, 8, 4], "all twenty, the last batch smaller");
        assert_eq!(status, Status::Idle, "then out of coal");
    }

    #[test]
    fn a_full_output_keeps_the_batch_waiting() {
        let mut coking = Coking::default();
        let status = coking.cook(COKE_TICKS, 8, |_| Err(Status::Full));
        assert_eq!(status, Status::Full, "no room");
        assert_eq!(
            (coking.charge, coking.cooked),
            (8, COKE_TICKS),
            "cooked and kept"
        );
        let (_, given) = cooked_all(&mut coking, 1, 8);
        assert_eq!(given.len(), 1, "it goes once there is room");
    }

    #[test]
    fn every_eighth_coal_gives_a_sulfur() {
        let mut coking = Coking::default();
        let mut sulfur = 0;
        for _ in 0..8 {
            let (_, given) = cooked_all(&mut coking, COKE_TICKS, 3);
            sulfur += given.iter().map(|done| done.sulfur).sum::<u32>();
        }
        assert_eq!(sulfur, 3, "24 coal in batches of three");
    }

    #[test]
    fn coal_taken_out_shrinks_the_batch() {
        let mut coking = Coking::default();
        cooked_all(&mut coking, 100, 8);
        cooked_all(&mut coking, 100, 5);
        assert_eq!(
            (coking.charge, coking.cooked),
            (5, 200),
            "five, still cooking"
        );
        let (status, _) = cooked_all(&mut coking, 100, 0);
        assert_eq!(status, Status::Idle, "all taken out");
        let (_, given) = cooked_all(&mut coking, COKE_TICKS - 1, 2);
        assert!(given.is_empty(), "a new batch starts from nothing");
    }
}
