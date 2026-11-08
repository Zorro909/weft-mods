//! Adding to a plan: templates and blocks, each refused if it reaches outside the region.

use super::*;

impl Plan<'_> {
    /// Places `template`, turned by `turn`, with its lowest corner at `corner`. What's added
    /// later wins where two overlap.
    pub fn structure(
        &mut self,
        template: &Template,
        corner: [i32; 3],
        turn: Turn,
    ) -> Result<Placed<'_>, PlanError> {
        self.check()?;
        let entry = self.entry(template)?;
        let (from, to) = box_of(corner, turn.size(entry.size()));
        self.inside(&entry.name, from, to)?;
        self.parts.push(Part::Piece(Piece {
            template: entry,
            corner,
            turn,
            foundation: None,
            clear_above: false,
            ground: Vec::new(),
        }));
        let Some(Part::Piece(piece)) = self.parts.last_mut() else {
            unreachable!("a piece was just pushed");
        };
        Ok(Placed {
            piece,
            ground: self.ground,
        })
    }

    pub fn set(&mut self, pos: [i32; 3], block: &Block) -> Result<(), PlanError> {
        self.set_state(pos, block.id())
    }

    /// Sets a block state, like `BLOCK.with(..)` gives.
    pub fn set_state(&mut self, pos: [i32; 3], state: u32) -> Result<(), PlanError> {
        self.fill_state(pos, pos, state)
    }

    /// Every block from `from` to `to`, both included.
    pub fn fill(&mut self, from: [i32; 3], to: [i32; 3], block: &Block) -> Result<(), PlanError> {
        self.fill_state(from, to, block.id())
    }

    /// An unknown block (`u32::MAX`) is left out; the SDK has already said so.
    fn fill_state(&mut self, a: [i32; 3], b: [i32; 3], state: u32) -> Result<(), PlanError> {
        self.check()?;
        let (from, to) = (
            [0, 1, 2].map(|i| a[i].min(b[i])),
            [0, 1, 2].map(|i| a[i].max(b[i])),
        );
        self.inside("blocks", from, to)?;
        if state != u32::MAX {
            self.parts.push(Part::Fill { from, to, state });
        }
        Ok(())
    }

    fn inside(&mut self, what: &str, from: [i32; 3], to: [i32; 3]) -> Result<(), PlanError> {
        if self.region.holds(from, to) {
            return Ok(());
        }
        Err(self.fail(PlanError::OutsideRegion {
            what: what.to_string(),
            from,
            to,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::tests::{HUT, plan};

    #[test]
    fn a_structure_inside_the_region_is_added() {
        let mut plan = plan();
        assert!(
            plan.structure(&HUT, [60, 64, 62], Turn::None).is_ok(),
            "up to the last column"
        );
        assert_eq!(
            plan.finish(Ok(())).map(|parts| parts.len()),
            Ok(1),
            "one part"
        );
    }

    #[test]
    fn reaching_outside_fails_the_plan() {
        let mut plan = plan();
        let error = plan.structure(&HUT, [61, 64, 0], Turn::None).err();
        assert!(
            matches!(
                error,
                Some(PlanError::OutsideRegion {
                    to: [64, 65, 1],
                    ..
                })
            ),
            "one column too far, got {error:?}"
        );
        assert!(
            plan.set_state([1, 64, 1], 5).is_err(),
            "nothing after it is taken"
        );
        assert!(plan.finish(Ok(())).is_err(), "the whole plan is given up");
    }

    #[test]
    fn a_turn_changes_what_fits() {
        let mut plan = plan();
        assert!(
            plan.structure(&HUT, [62, 64, 0], Turn::Right).is_ok(),
            "2 wide once turned"
        );
        assert!(
            plan.structure(&HUT, [62, 64, 10], Turn::None).is_err(),
            "4 wide unturned"
        );
    }

    #[test]
    fn an_ignored_error_still_fails_the_plan() {
        let mut plan = plan();
        let _ = plan.set_state([-1, 64, 0], 5);
        assert!(plan.finish(Ok(())).is_err(), "the mistake is remembered");
    }

    #[test]
    fn fills_take_their_corners_in_any_order() {
        let mut plan = plan();
        assert!(plan.fill_state([5, 70, 9], [1, 64, 2], 5).is_ok(), "inside");
        let parts = plan.finish(Ok(())).unwrap_or_default();
        assert_eq!(
            parts.first().map(Part::bounds),
            Some(([1, 64, 2], [5, 70, 9])),
            "lowest corner first"
        );
    }
}
