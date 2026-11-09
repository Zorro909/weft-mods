//! Adding to a plan: templates and blocks, each refused if it reaches outside the region.

use super::*;

impl Plan<'_> {
    /// Places `template`, turned by `turn`, with its lowest corner at `corner`. What's added
    /// later wins where two overlap.
    pub fn structure(
        &mut self,
        template: &Template,
        corner: Pos,
        turn: Turn,
    ) -> Result<Placed<'_>, PlanError> {
        self.check()?;
        let entry = self.entry(template)?;
        self.inside(&entry.name, Area::sized(corner, turn.size(entry.size())))?;
        self.parts.push(Part::Piece(Piece {
            template: entry,
            corner,
            turn,
            materials: Default::default(),
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

    pub fn set(&mut self, pos: Pos, block: &Block) -> Result<(), PlanError> {
        self.set_state(pos, BlockState::of(block))
    }

    /// Sets a block state: `BlockState::from(FURNACE.with(&[("lit", "true")]))`, or
    /// `BlockState::of(&FURNACE).with_prop(..)`.
    pub fn set_state(&mut self, pos: Pos, state: BlockState) -> Result<(), PlanError> {
        self.fill_state(pos, pos, state)
    }

    /// Every block from `from` to `to`, both included.
    pub fn fill(&mut self, from: Pos, to: Pos, block: &Block) -> Result<(), PlanError> {
        self.fill_state(from, to, BlockState::of(block))
    }

    /// Every block from `from` to `to` that has `tag` as the world generates there becomes
    /// `block`: leaves to air beside a road, so a tree it cut leaves no crown floating.
    pub fn replace(
        &mut self,
        from: Pos,
        to: Pos,
        tag: &'static Tag,
        block: &Block,
    ) -> Result<(), PlanError> {
        self.fill_where(from, to, BlockState::of(block), Some(tag))
    }

    pub(crate) fn fill_state(
        &mut self,
        a: Pos,
        b: Pos,
        state: BlockState,
    ) -> Result<(), PlanError> {
        self.fill_where(a, b, state, None)
    }

    /// An unknown block (`u32::MAX`) is left out; the SDK has already said so.
    fn fill_where(
        &mut self,
        a: Pos,
        b: Pos,
        state: BlockState,
        only: Option<&'static Tag>,
    ) -> Result<(), PlanError> {
        self.check()?;
        let area = Area::between(a, b);
        self.inside("blocks", area)?;
        let state = state.id();
        if state != u32::MAX {
            self.parts.push(Part::Fill { area, state, only });
        }
        Ok(())
    }

    fn inside(&mut self, what: &str, area: Area) -> Result<(), PlanError> {
        if self.region.holds(&area) {
            return Ok(());
        }
        Err(self.fail(PlanError::OutsideRegion {
            what: what.to_string(),
            area,
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
            plan.structure(&HUT, Pos::new(60, 64, 62), Turn::None)
                .is_ok(),
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
        let error = plan.structure(&HUT, Pos::new(61, 64, 0), Turn::None).err();
        assert!(
            matches!(
                error,
                Some(PlanError::OutsideRegion {
                    area: Area {
                        hi: Pos { x: 64, y: 65, z: 1 },
                        ..
                    },
                    ..
                })
            ),
            "one column too far, got {error:?}"
        );
        let stone = BlockState::from(5);
        assert!(
            plan.set_state(Pos::new(1, 64, 1), stone).is_err(),
            "nothing after it is taken"
        );
        assert!(plan.finish(Ok(())).is_err(), "the whole plan is given up");
    }

    #[test]
    fn a_turn_changes_what_fits() {
        let mut plan = plan();
        assert!(
            plan.structure(&HUT, Pos::new(62, 64, 0), Turn::Right)
                .is_ok(),
            "2 wide once turned"
        );
        assert!(
            plan.structure(&HUT, Pos::new(62, 64, 10), Turn::None)
                .is_err(),
            "4 wide unturned"
        );
    }

    #[test]
    fn an_ignored_error_still_fails_the_plan() {
        let mut plan = plan();
        let _ = plan.set_state(Pos::new(-1, 64, 0), BlockState::from(5));
        assert!(plan.finish(Ok(())).is_err(), "the mistake is remembered");
    }

    #[test]
    fn fills_take_their_corners_in_any_order() {
        let mut plan = plan();
        let (a, b) = (Pos::new(5, 70, 9), Pos::new(1, 64, 2));
        assert!(plan.fill_state(a, b, BlockState::from(5)).is_ok(), "inside");
        let parts = plan.finish(Ok(())).unwrap_or_default();
        assert_eq!(
            parts.first().map(Part::bounds),
            Some(Area { lo: b, hi: a }),
            "lowest corner first"
        );
    }
}
