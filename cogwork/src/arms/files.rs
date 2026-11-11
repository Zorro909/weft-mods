//! The model files of hubs with arms. Sixty-four each for tubes, liquid tubes and torque shafts
//! are too many to draw one by one, so they are written from here (`blockbench`). After a
//! change, `WRITE_ASSETS=1 cargo test --release arms` writes them again; without it, the test
//! fails on a file that is not what is written here.

use weft_sdk::*;

use crate::blockbench::{Corners, WRITE, element, folder, model, save_or_check};

/// A block made of a hub and arms.
#[derive(Clone, Copy)]
enum Fitting {
    Tube,
    Shaft,
    LiquidTube,
}

impl Fitting {
    /// Its folder of models, as `arms::model` takes it.
    fn block(self) -> &'static str {
        match self {
            Fitting::Tube => "tube",
            Fitting::Shaft => "torque_shaft",
            Fitting::LiquidTube => "liquid_tube",
        }
    }

    /// Its hub, in the middle of the block: a tube's 8 texels wide, a shaft's 4. The arms are
    /// as thick.
    fn hub(self) -> Corners {
        match self {
            Fitting::Tube | Fitting::LiquidTube => ([4.0; 3], [12.0; 3]),
            Fitting::Shaft => ([6.0; 3], [10.0; 3]),
        }
    }

    /// The textures the file names, and the picture Blockbench shows for each. A tube has one a
    /// side, which each tube sets to its own (`tube::shape`); the file shows a sorting tube's.
    fn textures(self) -> Vec<(&'static str, String)> {
        match self {
            Fitting::Tube => {
                let side = |dir| format!("block/{}", crate::tube::sorting::arm_texture(dir));
                let sides = Dir::ALL.into_iter().map(|dir| (dir.name(), side(dir)));
                sides
                    .chain([("particle", "block/tube_sorting".into())])
                    .collect()
            }
            Fitting::Shaft => ["particle", "shaft"]
                .map(|name| (name, "block/torque_shaft".into()))
                .to_vec(),
            Fitting::LiquidTube => vec![
                ("particle", "block/liquid_tube".into()),
                ("hub", "block/liquid_tube_hub".into()),
                ("arm", "block/liquid_tube".into()),
            ],
        }
    }

    /// The texture on `face` of the arm toward `arm`, or of the hub: a tube's arm shows its
    /// side's all round, its hub each side's on the face that way. A liquid tube's hub is
    /// reinforced glass.
    fn texture(self, arm: Option<Dir>, face: Dir) -> Option<&'static str> {
        Some(match (self, arm) {
            (Fitting::Tube, _) => arm.unwrap_or(face).name(),
            (Fitting::Shaft, _) => "shaft",
            (Fitting::LiquidTube, None) => "hub",
            (Fitting::LiquidTube, Some(_)) => "arm",
        })
    }
}

/// The box as thick as `hub` that reaches from its face toward `side` out to the edge of the
/// block.
fn arm((mut from, mut to): Corners, side: Dir) -> Corners {
    let axis = side.face() as usize / 2;
    // Even faces are the low side of their axis.
    if side.face().is_multiple_of(2) {
        (from[axis], to[axis]) = (0.0, from[axis]);
    } else {
        (from[axis], to[axis]) = (to[axis], 16.0);
    }
    (from, to)
}

/// The model file of `fitting` with `arms`.
fn model_file(fitting: Fitting, arms: &[Dir]) -> String {
    let hub = fitting.hub();
    let mut elements = vec![element("hub", hub, |face| fitting.texture(None, face))];
    for &side in arms {
        let texture = |face| fitting.texture(Some(side), face);
        elements.push(element(side.name(), arm(hub, side), texture));
    }
    model(fitting.textures(), elements)
}

#[test]
fn arms_reach_from_the_hub_to_the_edge() {
    let hub = Fitting::Tube.hub();
    let west = ([0.0, 4.0, 4.0], [4.0, 12.0, 12.0]);
    assert_eq!(arm(hub, Dir::West), west, "west");
    let up = ([4.0, 12.0, 4.0], [12.0, 16.0, 12.0]);
    assert_eq!(arm(hub, Dir::Up), up, "up");
}

#[test]
fn the_model_files_are_what_is_written_here() {
    for fitting in [Fitting::Tube, Fitting::Shaft, Fitting::LiquidTube] {
        for set in 0..64 {
            let arms: Vec<Dir> = Dir::ALL
                .into_iter()
                .filter(|dir| set & (1 << dir.face()) != 0)
                .collect();
            let name = super::model(fitting.block(), |dir| arms.contains(&dir));
            save_or_check(
                &folder("").join(format!("{name}.json")),
                &model_file(fitting, &arms),
            );
        }
        let files = std::fs::read_dir(folder(fitting.block())).expect("no folder of models");
        assert_eq!(
            files.count(),
            64,
            "a model no set of arms draws: {WRITE}=1 writes them"
        );
    }
}
