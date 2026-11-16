//! The blocks a stall is built of, in three woods: oak with a red and white awning, spruce with
//! blue and white, birch with green and yellow. Each wood is a set of its own: register,
//! counter, crate, post and awning. Only the register has a block entity (`stall.rs`), which
//! using any part opens.

use weft_sdk::{Block, BlockBuilder, Facings, Registry, Toward, tool_tier};

use super::pattern::Part;
use super::stall::finish_register;

/// The woods a stall comes in.
pub const WOODS: [&str; 3] = ["oak", "spruce", "birch"];

pub static REGISTERS: [Block; 3] = [
    Block::named("oak_stall_register"),
    Block::named("spruce_stall_register"),
    Block::named("birch_stall_register"),
];
static COUNTERS: [Block; 3] = [
    Block::named("oak_stall_counter"),
    Block::named("spruce_stall_counter"),
    Block::named("birch_stall_counter"),
];
static CRATES: [Block; 3] = [
    Block::named("oak_stall_crate"),
    Block::named("spruce_stall_crate"),
    Block::named("birch_stall_crate"),
];
static POSTS: [Block; 3] = [
    Block::named("oak_stall_post"),
    Block::named("spruce_stall_post"),
    Block::named("birch_stall_post"),
];
static AWNINGS: [Block; 3] = [
    Block::named("oak_stall_awning"),
    Block::named("spruce_stall_awning"),
    Block::named("birch_stall_awning"),
];

/// Every part's blocks, one per wood.
const PARTS: [(Part, &[Block; 3]); 5] = [
    (Part::Register, &REGISTERS),
    (Part::Counter, &COUNTERS),
    (Part::Crate, &CRATES),
    (Part::Post, &POSTS),
    (Part::Awning, &AWNINGS),
];

/// Registers each wood's blocks. Using any of them opens the stall it belongs to.
pub fn register(r: &mut Registry) {
    for (wood, name) in WOODS.iter().enumerate() {
        let planks = format!("{name}_stall_planks");
        let awning = format!("{name}_stall_awning");
        finish_register(
            wooden(r.block(&REGISTERS[wood]))
                .facing(Facings::HORIZONTAL, Toward::Player)
                .states(move |_, state| {
                    state
                        .texture(&planks)
                        .model_box([0.0; 3], [16.0; 3])
                        .model_box_faces([3.0, 16.0, 4.0], [13.0, 22.0, 12.0], till_faces())
                }),
        );
        wooden(r.block(&COUNTERS[wood])).texture(&format!("{name}_stall_planks"));
        wooden(r.block(&CRATES[wood])).texture(&format!("{name}_stall_crate"));
        wooden(r.block(&POSTS[wood])).states(move |_, state| {
            state
                .texture(&format!("{}_stall_post", WOODS[wood]))
                .model_box([6.0, 0.0, 6.0], [10.0, 16.0, 10.0])
        });
        wooden(r.block(&AWNINGS[wood])).states(move |_, state| {
            state
                .texture(&awning)
                .model_box([0.0; 3], [16.0, 3.0, 16.0])
        });
    }
}

/// What a stall's blocks share: wood, cut with an axe, and usable, so the engine asks
/// `stall.rs` before a player uses one.
fn wooden(builder: BlockBuilder<'_>) -> BlockBuilder<'_> {
    builder
        .on_use(|_| false)
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.wood")
        .tab(&crate::TAB)
}

/// The till on the register's counter: its keys and drawer on every face.
fn till_faces() -> [(&'static str, [f32; 4]); 6] {
    [("stall_till", [0.0, 0.0, 16.0, 16.0]); 6]
}

/// Which part of a stall the block state `state` is, if any.
pub fn part_of(state: u32) -> Option<Part> {
    PARTS
        .iter()
        .find(|(_, blocks)| blocks.iter().any(|block| block.is(state)))
        .map(|(part, _)| *part)
}
