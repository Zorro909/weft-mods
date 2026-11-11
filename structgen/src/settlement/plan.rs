//! What the planner works with: the engine's plan of one start, the start's random numbers, and
//! the buildings' sizes. It reads the terrain's columns and never blocks, so the server and every
//! client lay a settlement out alike.

use weft_sdk::terrain::Column;
use weft_sdk::{Block, LootTable, Palette, Template, Transform};
use weftkit::{BlockState, Pos};

use super::fit::Fit;
use super::random::Random;
use crate::buildings;
use crate::turn::Turn;

pub struct Plan<'a> {
    start: &'a mut weft_sdk::Plan,
    random: Random,
}

impl<'a> Plan<'a> {
    pub fn new(start: &'a mut weft_sdk::Plan) -> Plan<'a> {
        let random = Random::new(start.random());
        Plan { start, random }
    }

    /// The column the start is in: the block above the ground.
    pub fn start(&self) -> Pos {
        self.start.pos().into()
    }

    /// The same numbers every time this start is laid out.
    pub fn random(&mut self) -> &mut Random {
        &mut self.random
    }

    /// The ground of column (`x`, `z`): `None` where nothing may stand (no terrain, a cave
    /// mouth).
    pub fn ground(&self, x: i32, z: i32) -> Option<Column> {
        let column = self.start.ground(x, z);
        (column.height != i32::MIN && !self.start.surface_carved(x, z)).then_some(column)
    }

    /// The size of building `template` along x, y and z once turned by `turn`.
    pub fn size(&self, template: &Template, turn: Turn) -> Option<[i32; 3]> {
        buildings::size(template).map(|size| turn.size(size))
    }

    /// Where building `template`, turned by `turn` and with its lowest corner at `corner` (x,
    /// z), would stand: `None` where the ground doesn't meet `fit`.
    pub fn fit(&self, template: &Template, [x, z]: [i32; 2], turn: Turn, fit: &Fit) -> Option<Pos> {
        let [sx, _, sz] = self.size(template, turn)?;
        let y = fit.floor([x, z], [sx, sz], &|x, z| self.ground(x, z))?;
        Some(Pos::new(x, y, z))
    }

    /// Building `template` turned by `turn` with its lowest corner at `corner`, its slots from
    /// `palette` and its ground paved if `paved`: standing on the ground, the hill it is set
    /// into cut away.
    pub fn building(
        &mut self,
        template: &Template,
        corner: Pos,
        turn: Turn,
        palette: Option<&Palette>,
        paved: bool,
    ) {
        let Some(placed) = buildings::placed(template, turn, paved) else {
            return;
        };
        let piece = self.start.template(placed, corner, Transform::NONE);
        let piece = piece.foundation_of_ground().clear_above();
        match palette {
            Some(palette) => piece.palette(palette).put(),
            None => piece.put(),
        };
    }

    /// The chest at `pos` filled from `table` once it joins the world.
    pub fn loot(&mut self, pos: Pos, table: &LootTable) {
        self.start.loot(pos, table);
    }

    pub fn set(&mut self, pos: Pos, block: &Block) {
        self.start.set(pos, block.id());
    }

    pub fn set_state(&mut self, pos: Pos, state: BlockState) {
        self.start.set(pos, state.id());
    }

    /// Every block from `from` to `to`, both included.
    pub fn fill(&mut self, from: Pos, to: Pos, block: &Block) {
        self.start.fill(from, to, block.id());
    }
}
