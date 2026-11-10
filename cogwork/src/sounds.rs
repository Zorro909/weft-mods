//! Cogwork's sound events. Their files are `assets/sounds/<file><n>.ogg`, one picked at random,
//! made with `tools/sfx.sh` from the prompts in `sounds.tsv`.

use weft_sdk::*;

pub static SPRING_STROKE: Sound = Sound::named("engine.spring_stroke");
pub static EMBER_STROKE: Sound = Sound::named("engine.ember_stroke");
pub static EMBER_IGNITE: Sound = Sound::named("engine.ember_ignite");
pub static OVERHEAT: Sound = Sound::named("engine.overheat");
pub static COOLED: Sound = Sound::named("engine.cooled");
pub static TUBE_PULL: Sound = Sound::named("tube.pull");
pub static TUBE_VOID: Sound = Sound::named("tube.void");
pub static TUBE_JAM: Sound = Sound::named("tube.jam_drop");
pub static FRAME_PLACE: Sound = Sound::named("excavator.frame_place");
pub static DRILL: Sound = Sound::named("excavator.drill");
pub static EXCAVATOR_DONE: Sound = Sound::named("excavator.done");
pub static CRAFT: Sound = Sound::named("bench.craft");
pub static SCRAPE: Sound = Sound::named("leveler.scrape");
pub static FILL: Sound = Sound::named("leveler.fill");
pub static WRENCH: Sound = Sound::named("tool.wrench");
pub static SURVEY: Sound = Sound::named("marker.survey");
pub static GHOST: Sound = Sound::named("ui.ghost");

/// The sounds heard in the world: files, volume, lowest and highest pitch, and the blocks they
/// carry (fading out on the way there): heavy machinery far, small parts close.
static IN_WORLD: [(&Sound, &str, f32, f32, f32, f32); 16] = [
    (&SPRING_STROKE, "engine/spring_stroke", 0.5, 0.9, 1.1, 24.0),
    (&EMBER_STROKE, "engine/ember_stroke", 0.5, 0.9, 1.05, 40.0),
    (&EMBER_IGNITE, "engine/ember_ignite", 0.7, 0.9, 1.1, 24.0),
    (&OVERHEAT, "engine/overheat", 1.0, 0.95, 1.05, 48.0),
    (&COOLED, "engine/cooled", 0.7, 0.95, 1.05, 32.0),
    (&TUBE_PULL, "tube/pull", 0.5, 0.85, 1.15, 16.0),
    (&TUBE_VOID, "tube/void", 0.6, 0.9, 1.1, 16.0),
    (&TUBE_JAM, "tube/jam_drop", 0.8, 0.9, 1.1, 20.0),
    (&FRAME_PLACE, "excavator/frame_place", 0.6, 0.85, 1.1, 40.0),
    (&DRILL, "excavator/drill", 0.7, 0.9, 1.1, 48.0),
    (&EXCAVATOR_DONE, "excavator/done", 1.0, 1.0, 1.0, 64.0),
    (&CRAFT, "bench/craft", 0.7, 0.95, 1.05, 24.0),
    (&SCRAPE, "leveler/scrape", 0.6, 0.85, 1.1, 32.0),
    (&FILL, "leveler/fill", 0.6, 0.85, 1.1, 32.0),
    (&WRENCH, "tool/wrench", 0.8, 0.95, 1.05, 12.0),
    (&SURVEY, "marker/survey", 0.8, 1.0, 1.0, 32.0),
];

pub fn register(r: &mut Registry) {
    for &(sound, file, volume, low, high, range) in &IN_WORLD {
        r.sound(sound)
            .files(&[&format!("{file}*")])
            .volume(volume)
            .pitch(low, high)
            .range(range);
    }
    r.sound(&GHOST)
        .files(&["ui/ghost*"])
        .category(&sound_categories::UI)
        .volume(0.5);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_has_files() {
        let sounds = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/sounds");
        let files = IN_WORLD.iter().map(|entry| entry.1).chain(["ui/ghost"]);
        for file in files {
            assert!(
                sounds.join(format!("{file}1.ogg")).is_file(),
                "no file for {file}"
            );
        }
    }
}
