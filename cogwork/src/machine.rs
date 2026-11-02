//! What this mod's machines share: a block whose front lights up while it works, energy put into
//! a job a little every tick, and the energy bar on their screens.

use weft_sdk::*;

/// A machine's block, textured `<name>_front` (`<name>_front_on` while it works), `<name>_top`
/// and `<name>_side`, also underneath. Its front faces the player who places it.
pub fn block<'a>(b: BlockBuilder<'a>, name: &'static str) -> BlockBuilder<'a> {
    b.facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .states(move |p, s| {
            let (side, top) = (format!("{name}_side"), format!("{name}_top"));
            let on = if p.is("working", "true") { "_on" } else { "" };
            let front = format!("{name}_front{on}");
            let mut faces = [side.as_str(); 6];
            faces[Dir::Up.face() as usize] = &top;
            faces[p.facing().face() as usize] = &front;
            s.faces(faces)
        })
        .mineable("pickaxe")
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
}

/// The way the front of the machine at `pos` faces.
pub fn front(w: &World, pos: [i32; 3]) -> Dir {
    w.prop(pos, "facing")
        .and_then(Dir::from_name)
        .unwrap_or(Dir::North)
}

/// Lights the front of the machine at `pos` while it works.
pub fn show_working(w: &World, pos: [i32; 3], working: bool) {
    let value = if working { "true" } else { "false" };
    if w.prop(pos, "working").is_some_and(|now| now != value) {
        w.set_prop(pos, "working", value);
    }
}

/// Puts up to `rate` of the machine `id`'s stored energy into a job of `cost`, of which `spent`
/// is paid already; returns how much it put in.
pub fn draw(w: &World, id: u64, spent: &mut i64, cost: i64, rate: i64) -> i64 {
    let Some(e) = w.energy_mut(id) else {
        return 0;
    };
    let drawn = e.stored.min(rate).min((cost - *spent).max(0));
    e.stored -= drawn;
    *spent += drawn;
    drawn
}

/// The energy of the machine whose menu is open: a bar and the numbers.
pub fn power_bar(ui: &mut Ui) {
    ui.progress_bind("cogwork:power", S::default().size(80.0, 8.0), |c| {
        let e = c.menu_component(&ENERGY).unwrap_or_default();
        e.stored as f32 / e.capacity.max(1) as f32
    });
    ui.text_bind(|c, out| {
        let e = c.menu_component(&ENERGY).unwrap_or_default();
        out.set(
            "cogwork.energy",
            &[Arg::Int(e.stored), Arg::Int(e.capacity)],
        );
    });
}
