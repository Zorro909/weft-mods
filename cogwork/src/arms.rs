//! The model of a hub with arms toward some of its six sides, as tubes and torque shafts have.
//! A block state draws one model file, so each set of arms is a Blockbench model of its own,
//! named after its arms: `assets/models/block/<block>/<arms>.json`.

use weft_sdk::*;

/// The model file of `block` with an arm toward every side `has_arm` says: its arms in the
/// order of `Dir::ALL` (`tube/west_up_north`), or `alone` without any.
pub fn model(block: &str, has_arm: impl Fn(Dir) -> bool) -> String {
    let arms: Vec<&str> = Dir::ALL
        .into_iter()
        .filter(|&dir| has_arm(dir))
        .map(Dir::name)
        .collect();
    if arms.is_empty() {
        format!("{block}/alone")
    } else {
        format!("{block}/{}", arms.join("_"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_model_is_named_after_its_arms() {
        let has_arm = |dir| matches!(dir, Dir::Up | Dir::West | Dir::North);
        assert_eq!(model("tube", has_arm), "tube/west_up_north", "in order");
        assert_eq!(model("tube", |_| false), "tube/alone", "no arms");
    }

    #[test]
    fn tubes_and_shafts_have_a_model_for_every_set_of_arms() {
        for set in 0..64 {
            let has_arm = |dir: Dir| set & (1 << dir.face()) != 0;
            for block in ["tube", "torque_shaft"] {
                let file = format!("models/block/{}.json", model(block, has_arm));
                assert!(crate::has_asset(&file), "no {file}");
            }
        }
    }
}
