//! `Plan`: what a planner gets to lay out one region of a structure. It sees the terrain's
//! columns and never blocks, so every cube and every client works out the same plan.

use std::fmt;
use std::sync::Arc;

use weft_sdk::{Block, Template};

use crate::fit::{Fit, Ground};
use crate::part::{Part, Piece, Placed, box_of};
use crate::random::Random;
use crate::region::Region;
use crate::templates::{self, Entry};
use crate::turn::Turn;

/// Why a plan was given up. Nothing of it is placed, and generation logs why.
#[derive(Clone, Debug, PartialEq)]
pub enum PlanError {
    /// `what` (a template's name, or `blocks`) would cover `from..=to`, reaching outside the
    /// region.
    OutsideRegion {
        what: String,
        from: [i32; 3],
        to: [i32; 3],
    },
    /// A template this mod didn't register with `structures_api::template` or `place`.
    UnknownTemplate(String),
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlanError::OutsideRegion { what, from, to } => {
                write!(
                    f,
                    "{what} from {from:?} to {to:?} reaches outside the region"
                )
            }
            PlanError::UnknownTemplate(name) => write!(
                f,
                "template {name} is not registered with structures_api::template"
            ),
        }
    }
}

/// One region of a structure being laid out. Everything added must stay inside the region,
/// seen from above; the first thing that doesn't fails the plan.
pub struct Plan<'a> {
    region: Region,
    random: Random,
    ground: &'a dyn Fn(i32, i32) -> Option<Ground>,
    parts: Vec<Part>,
    error: Option<PlanError>,
}

impl<'a> Plan<'a> {
    pub(crate) fn new(
        region: Region,
        random: Random,
        ground: &'a dyn Fn(i32, i32) -> Option<Ground>,
    ) -> Plan<'a> {
        Plan {
            region,
            random,
            ground,
            parts: Vec::new(),
            error: None,
        }
    }

    pub fn region(&self) -> Region {
        self.region
    }

    /// The same numbers every time this region is planned.
    pub fn random(&mut self) -> &mut Random {
        &mut self.random
    }

    /// A random column at least `margin` blocks inside every edge of the region; its middle when
    /// the region is too small for that.
    pub fn spot(&mut self, margin: i32) -> [i32; 2] {
        let [x, z] = self.region.min;
        let size = self.region.size();
        let room = (size - 2 * margin) as u64;
        if size <= 2 * margin {
            return [x + size / 2, z + size / 2];
        }
        let dx = self.random.below(room) as i32;
        let dz = self.random.below(room) as i32;
        [x + margin + dx, z + margin + dz]
    }

    /// The ground of column (`x`, `z`): `None` where nothing may stand (no terrain, a cave mouth).
    pub fn ground(&self, x: i32, z: i32) -> Option<Ground> {
        (self.ground)(x, z)
    }

    /// The size of `template` along x, y and z once turned by `turn`.
    pub fn size(&mut self, template: &Template, turn: Turn) -> Option<[i32; 3]> {
        Some(turn.size(self.entry(template).ok()?.size()))
    }

    /// Where `template`, turned by `turn` and with its lowest corner at `corner` (x, z), would
    /// stand: `None` where the ground doesn't meet `fit` or it would leave the region.
    pub fn fit(
        &mut self,
        template: &Template,
        [x, z]: [i32; 2],
        turn: Turn,
        fit: &Fit,
    ) -> Option<[i32; 3]> {
        let entry = self.entry(template).ok()?;
        let [sx, _, sz] = turn.size(entry.size());
        if !self.region.holds([x, 0, z], [x + sx - 1, 0, z + sz - 1]) {
            return None;
        }
        let y = fit.floor([x, z], [sx, sz], self.ground)?;
        Some([x, y, z])
    }

    /// Whether `template`, turned by `turn` with its lowest corner at `corner`, would overlap
    /// nothing already in this plan.
    pub fn is_free(&mut self, template: &Template, corner: [i32; 3], turn: Turn) -> bool {
        let Ok(entry) = self.entry(template) else {
            return false;
        };
        let (from, to) = box_of(corner, turn.size(entry.size()));
        self.parts.iter().all(|part| {
            let (other_from, other_to) = part.bounds();
            (0..3).any(|i| to[i] < other_from[i] || other_to[i] < from[i])
        })
    }

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

    /// The parts, or why the plan was given up: `result` is what the planner returned, and a
    /// mistake it went past fails the plan too.
    pub(crate) fn finish(self, result: Result<(), PlanError>) -> Result<Vec<Part>, PlanError> {
        result?;
        match self.error {
            Some(error) => Err(error),
            None => Ok(self.parts),
        }
    }

    /// The first mistake, again: once a plan failed, everything after fails too.
    fn check(&self) -> Result<(), PlanError> {
        self.error.clone().map_or(Ok(()), Err)
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

    /// The template registered as `template`; the plan fails without one.
    fn entry(&mut self, template: &Template) -> Result<Arc<Entry>, PlanError> {
        templates::find(template.name())
            .ok_or_else(|| self.fail(PlanError::UnknownTemplate(template.name().to_string())))
    }

    /// Remembers the plan's first mistake.
    fn fail(&mut self, error: PlanError) -> PlanError {
        self.error.get_or_insert(error.clone());
        error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::Structure;

    static HUT: Template = Template::named("test:plan_hut");
    static MISSING: Template = Template::named("test:plan_missing");

    fn flat(_: i32, _: i32) -> Option<Ground> {
        Some(Ground {
            height: 63,
            biome: 0,
        })
    }

    /// A plan of region [0, 0] of a 64-block grid, with a 4 x 2 x 2 hut registered.
    fn plan() -> Plan<'static> {
        templates::add(
            HUT.name(),
            Structure::from_names([4, 2, 2], ["core:air"; 16]),
        );
        Plan::new(Region::at([0, 0], 64), Random::new(1), &flat)
    }

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
    fn an_unknown_template_fails_the_plan() {
        let mut plan = plan();
        assert_eq!(
            plan.fit(&MISSING, [0, 0], Turn::None, &Fit::new()),
            None,
            "no answer"
        );
        assert!(
            matches!(plan.finish(Ok(())), Err(PlanError::UnknownTemplate(_))),
            "and the plan says why"
        );
    }

    #[test]
    fn fit_stays_inside_the_region() {
        let mut plan = plan();
        assert_eq!(
            plan.fit(&HUT, [10, 10], Turn::None, &Fit::new()),
            Some([10, 63, 10]),
            "level with the ground"
        );
        assert_eq!(
            plan.fit(&HUT, [62, 10], Turn::None, &Fit::new()),
            None,
            "over the edge"
        );
        assert!(plan.finish(Ok(())).is_ok(), "asking is no mistake");
    }

    #[test]
    fn overlaps_are_seen() {
        let mut plan = plan();
        let _ = plan.structure(&HUT, [10, 64, 10], Turn::None);
        assert!(!plan.is_free(&HUT, [12, 65, 11], Turn::None), "overlapping");
        assert!(
            plan.is_free(&HUT, [14, 64, 10], Turn::None),
            "right next to it"
        );
    }

    #[test]
    fn spots_keep_their_margin() {
        let mut plan = plan();
        for _ in 0..200 {
            let [x, z] = plan.spot(10);
            assert!(
                (10..54).contains(&x) && (10..54).contains(&z),
                "inside the margin"
            );
        }
        assert_eq!(plan.spot(40), [32, 32], "the middle when there is no room");
    }
}
