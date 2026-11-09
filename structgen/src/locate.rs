//! `/structure locate <structure> [radius]`: where the nearest structure of this mod is,
//! generated yet or not.

use structgen_api::Pos;
use weft_sdk::*;

/// Blocks searched around the player without a radius.
const DEFAULT_RADIUS: i64 = 4096;
/// So one command can't stall the server planning a whole world.
const MAX_RADIUS: i64 = 16384;

pub fn run(cmd: &mut ServerCommand) {
    let Some(name) = cmd.text("structure").map(full_name) else {
        return;
    };
    if !structgen_api::names().contains(&name) {
        let known = short_names().join(", ");
        return cmd.reply("structgen.unknown", &[Arg::Text(&name), Arg::Text(&known)]);
    }
    let radius = cmd.int("radius").unwrap_or(DEFAULT_RADIUS);
    if !(1..=MAX_RADIUS).contains(&radius) {
        return cmd.reply("structgen.bad_radius", &[Arg::Int(MAX_RADIUS)]);
    }
    let world = cmd.world();
    // From the world's middle when no player ran it.
    let from = world
        .entity(cmd.player())
        .map_or(Pos::default(), |player| player.pos.block().into());
    match structgen_api::locate(&world, &name, from, radius as i32) {
        Some(found) => {
            let [x, y, z] = <[i32; 3]>::from(found).map(|v| Arg::Int(v as i64));
            let distance = Arg::Int(horizontal_distance(from, found));
            cmd.reply("structgen.located", &[Arg::Text(&name), x, y, z, distance]);
        }
        None => cmd.reply("structgen.not_found", &[Arg::Text(&name), Arg::Int(radius)]),
    }
}

/// The structures this mod registered, without its namespace: what the command completes.
pub fn short_names() -> Vec<String> {
    let prefix = format!("{}:", mod_id());
    let names = structgen_api::names();
    names
        .iter()
        .map(|name| name.strip_prefix(&prefix).unwrap_or(name).to_string())
        .collect()
}

/// `name` with this mod's namespace, unless it has one.
fn full_name(name: &str) -> String {
    if name.contains(':') {
        name.to_string()
    } else {
        format!("{}:{name}", mod_id())
    }
}

fn horizontal_distance(from: Pos, to: Pos) -> i64 {
    let (dx, dz) = ((to.x - from.x) as f64, (to.z - from.z) as f64);
    (dx * dx + dz * dz).sqrt().round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_seen_from_above() {
        assert_eq!(
            horizontal_distance(Pos::new(0, 64, 0), Pos::new(3, -20, 4)),
            5,
            "height left out"
        );
    }
}
