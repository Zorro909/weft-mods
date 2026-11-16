//! Every table Plotty keeps on disk, side by side: one set for the whole server, under
//! `data/plotty/`. A changed key or value type takes a new table name, as a changed `SAVE`
//! component does.

use weftkit::{PlayerKey, Pos, Table};

use crate::admin::spawn::Spawn;
use crate::city::grid::CellXZ;
use crate::city::undo::Build;
use crate::city::{Cell, City, Counters, Plot};
use crate::market::{Rent, StallEntry};
use crate::players::PlayerEntry;

/// Everyone who has joined: their name when last seen, and what moderators gave them.
pub static PLAYERS: Table<PlayerKey, PlayerEntry> = Table::named("players");

/// The spawn, once a game master set it: one value, under the key `()`.
pub static SPAWN: Table<(), Spawn> = Table::named("spawn");

/// Cities by number.
pub static CITIES: Table<u32, City> = Table::named("cities");

/// Every cell some city has, by dimension key and cell: asked by every protection hook.
pub static CELLS: Table<(u64, CellXZ), Cell> = Table::named("cells");

/// Plots by number.
pub static PLOTS: Table<u32, Plot> = Table::named("plots");

/// What each build replaced, by number, kept until it is undone.
pub static BUILDS: Table<u32, Build> = Table::named("builds");

/// The next numbers for cities, plots and builds: one value, under the key `()`.
pub static COUNTERS: Table<(), Counters> = Table::named("counters");

/// Each player's Golden Woof. Written at every payment, from every region: write-optimized.
pub static BALANCES: Table<PlayerKey, u64> = Table::write_optimized("balances");

/// Each stall's owner, by dimension key and its register's position.
pub static STALLS: Table<(u64, Pos), StallEntry> = Table::named("stalls");

/// Who rents each market place, by dimension key, market square corner cell and place number.
pub static RENTS: Table<(u64, CellXZ, u8), Rent> = Table::named("rents");

/// Trades each player's stalls made since they last joined.
pub static SALES: Table<PlayerKey, u32> = Table::write_optimized("sales");
