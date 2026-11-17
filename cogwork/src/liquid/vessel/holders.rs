//! Where each holder's `Held` is kept during a tick. A system works on copies of its values,
//! written back when it returns, so a holder changed during the tick goes through `Holders`,
//! which changes the copy when there is one.

use std::collections::HashMap;

use weft_sdk::World;

use crate::liquid::{HELD, Held};

/// The `Held` of every holder, read and written through this tick's copies where they are.
pub struct Holders<'a> {
    world: World,
    index: HashMap<u64, usize>,
    values: &'a mut [Held],
}

impl<'a> Holders<'a> {
    /// The holders of a system's batch, the others reached through the world.
    pub(in crate::liquid) fn of_batch(
        world: World,
        ids: &[u64],
        values: &'a mut [Held],
    ) -> Holders<'a> {
        let index = ids.iter().copied().zip(0..).collect();
        Holders {
            world,
            index,
            values,
        }
    }

    /// Every holder through the world: outside a system's tick.
    pub fn outside(world: World) -> Holders<'static> {
        Holders {
            world,
            index: HashMap::new(),
            values: &mut [],
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// What the holder `id` keeps now.
    pub fn get(&self, id: u64) -> Held {
        match self.index.get(&id) {
            Some(&i) => self.values[i],
            None => self.world.component(id, &HELD).unwrap_or_default(),
        }
    }

    /// Makes the holder `id` keep `held`.
    pub fn set(&mut self, id: u64, held: Held) {
        match self.index.get(&id) {
            Some(&i) => self.values[i] = held,
            None => {
                if let Some(stored) = self.world.component_mut(id, &HELD) {
                    *stored = held;
                }
            }
        }
    }
}
