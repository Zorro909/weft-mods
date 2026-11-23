//! The blocks a city stands on: a plot post at a corner of each plot, beside the street, and the
//! city stone on the plaza. Right-clicked, each says what it stands for. Neither can be broken:
//! `/city undo` takes them away with the rest of their build.

use weft_sdk::{Arg, Block, Output, Registry, Use, World};
use weftkit::{BlockState, Pos, TypedWorld};

use crate::city::grid::{Footprint, Side};
use crate::city::{CellKind, PlotSize, cell_at};
use crate::tables::{CELLS, CITIES, PLOTS};
use crate::{dims, players};

pub static PLOT_POST: Block = Block::named("plot_post");
pub static CITY_STONE: Block = Block::named("city_stone");

pub fn register(r: &mut Registry) {
    r.block(&PLOT_POST)
        .texture("plot_post")
        .unbreakable()
        .on_use(tell_plot)
        .tab(&crate::TAB);
    r.block(&CITY_STONE)
        .texture("city_stone")
        .unbreakable()
        .light(6)
        .on_use(tell_city)
        .tab(&crate::TAB);
}

/// The side of `footprint` a street, plaza or market lies along; south where none does yet.
pub fn street_side(dim: u64, footprint: Footprint) -> Side {
    let public = |side: Side| {
        footprint.beyond(side).into_iter().any(|cell| {
            CELLS
                .get(&(dim, cell))
                .is_some_and(|cell| cell.kind != CellKind::Plot)
        })
    };
    Side::ALL
        .into_iter()
        .find(|side| public(*side))
        .unwrap_or(Side::South)
}

/// Sets plot `plot`'s post at its corner on the street side, on the ground's kerb.
pub fn set(world: &World, plot: u32, ground_y: i32) {
    let Some(entry) = PLOTS.get(&plot) else {
        return;
    };
    let side = street_side(entry.dim, entry.footprint);
    world.set_block_typed(
        post_pos(entry.footprint, side, ground_y + 1),
        BlockState::of(&PLOT_POST),
    );
}

/// The corner block of `footprint` on `side`, at height `y`: the west end of a north or south
/// side, the north end of a west or east one.
fn post_pos(footprint: Footprint, side: Side, y: i32) -> Pos {
    let corner = footprint.corner.corner(y);
    let last = footprint.blocks() - 1;
    match side {
        Side::North | Side::West => corner,
        Side::South => corner.add_z(last),
        Side::East => corner.add_x(last),
    }
}

/// A post's answer: the plot's number, size and owner, or how to claim it.
fn tell_plot(used: &mut Use) -> bool {
    let world = used.world();
    let Some(plot_id) =
        cell_at(dims::key_of(&world), Pos::from(used.pos())).and_then(|cell| cell.plot)
    else {
        return true;
    };
    let Some(plot) = PLOTS.get(&plot_id) else {
        return true;
    };
    let city = CITIES
        .get(&plot.city)
        .map(|city| city.name)
        .unwrap_or_default();
    let size = size_key(plot.size);
    let owner = plot.owner.as_ref().map(players::name_of);
    let mut args = vec![Arg::Text(&city), Arg::Int(plot_id.into()), Arg::Key(size)];
    let key = match &owner {
        Some(owner) => {
            args.push(Arg::Text(owner));
            "plotty.post.owned"
        }
        None => "plotty.post.free",
    };
    world.chat(key, &args).to(used.player());
    true
}

/// The city stone's answer: the city and how many of its plots are free.
fn tell_city(used: &mut Use) -> bool {
    let world = used.world();
    let dim = dims::key_of(&world);
    let Some(cell) = cell_at(dim, Pos::from(used.pos())) else {
        return true;
    };
    let name = CITIES
        .get(&cell.city)
        .map(|city| city.name)
        .unwrap_or_default();
    let mut free = [0i64; 3];
    PLOTS.for_each(|_, plot| {
        if plot.city == cell.city && plot.owner.is_none() {
            free[plot.size as usize] += 1;
        }
    });
    let args = [
        Arg::Text(&name),
        Arg::Int(free[0]),
        Arg::Int(free[1]),
        Arg::Int(free[2]),
    ];
    world.chat("plotty.stone", &args).to(used.player());
    true
}

/// The lang key naming a plot size.
pub fn size_key(size: PlotSize) -> &'static str {
    match size {
        PlotSize::Small => "plotty.size.small",
        PlotSize::Normal => "plotty.size.normal",
        PlotSize::Large => "plotty.size.large",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::grid::CellXZ;

    #[test]
    fn a_post_stands_at_a_corner_on_the_street_side() {
        let footprint = Footprint {
            corner: CellXZ::new(1, 0),
            side: 2,
        };
        assert_eq!(
            post_pos(footprint, Side::North, 65),
            Pos::new(16, 65, 0),
            "north-west corner"
        );
        assert_eq!(
            post_pos(footprint, Side::South, 65),
            Pos::new(16, 65, 31),
            "south-west corner"
        );
        assert_eq!(
            post_pos(footprint, Side::East, 65),
            Pos::new(47, 65, 0),
            "north-east corner"
        );
    }
}
