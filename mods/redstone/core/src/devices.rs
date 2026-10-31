use crate::{Device, Dir, FULL, Power, Range, SidePower};

/// The built-in devices and their state.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Lever { on: bool, attached: Dir },
    Button { on: bool, attached: Dir, wood: bool },
    /// Attached below.
    Plate { on: bool },
    PowerBlock,
    /// `attached`: the side of the block it hangs on (`Down` = standing).
    Torch { lit: bool, attached: Dir },
    /// `facing`: the output side; `delay` 1..=4.
    Repeater { facing: Dir, delay: u8, powered: bool, locked: bool },
    Comparator { facing: Dir, subtract: bool, output: u8 },
    Lamp { lit: bool },
}

impl Kind {
    /// The same device turned a quarter clockwise seen from above.
    pub fn rotate_y(self) -> Kind {
        match self {
            Kind::Lever { on, attached } => Kind::Lever { on, attached: attached.rotate_y() },
            Kind::Button { on, attached, wood } => Kind::Button { on, attached: attached.rotate_y(), wood },
            Kind::Torch { lit, attached } => Kind::Torch { lit, attached: attached.rotate_y() },
            Kind::Repeater { facing, delay, powered, locked } => Kind::Repeater { facing: facing.rotate_y(), delay, powered, locked },
            Kind::Comparator { facing, subtract, output } => Kind::Comparator { facing: facing.rotate_y(), subtract, output },
            k => k,
        }
    }
}

/// Ticks a button stays on.
pub const BUTTON_STONE: u32 = 20;
pub const BUTTON_WOOD: u32 = 30;
/// Ticks a plate stays on after the last entity left it.
pub const PLATE_OFF: u32 = 20;

/// What a device emits to each side, clamped to `range`.
pub fn outputs(k: Kind, range: Range) -> [Power; 6] {
    let full = range.clamp(FULL);
    let weak = Power { strength: full, strong: false };
    let mut o = [Power::default(); 6];
    match k {
        Kind::Lever { on: true, attached } | Kind::Button { on: true, attached, .. } => {
            o = [weak; 6];
            o[attached.index()] = Power { strength: full, strong: true };
        }
        Kind::Plate { on: true } => {
            o = [weak; 6];
            o[Dir::Down.index()] = Power { strength: full, strong: true };
        }
        Kind::PowerBlock => o = [weak; 6],
        Kind::Torch { lit: true, attached } => {
            o = [weak; 6];
            o[attached.index()] = Power::default();
            o[Dir::Up.index()] = Power { strength: full, strong: true };
        }
        Kind::Repeater { facing, powered: true, .. } => o[facing.index()] = Power { strength: full, strong: true },
        Kind::Comparator { facing, output, .. } if output > 0 => o[facing.index()] = Power { strength: range.clamp(output), strong: true },
        _ => {}
    }
    o
}

/// The sides wire connects to, a bit per `Dir::index`.
pub fn connects(k: Kind) -> u8 {
    let bit = |d: Dir| 1u8 << d.index();
    match k {
        Kind::Lever { .. } | Kind::Button { .. } | Kind::Plate { .. } | Kind::PowerBlock | Kind::Torch { .. } => 0b111111,
        Kind::Repeater { facing, .. } => bit(facing) | bit(facing.opposite()),
        Kind::Comparator { facing, .. } => bit(facing) | bit(facing.opposite()) | sides_of(facing).iter().fold(0, |m, d| m | bit(*d)),
        Kind::Lamp { .. } => 0,
    }
}

/// The device cell of a built-in device.
pub fn device(k: Kind, range: Range) -> Device {
    Device {
        output: outputs(k, range),
        connects: connects(k),
        receives: matches!(k, Kind::Torch { .. } | Kind::Repeater { .. } | Kind::Comparator { .. } | Kind::Lamp { .. }),
        solid: matches!(k, Kind::PowerBlock | Kind::Lamp { .. }),
        kind: Some(k),
    }
}

/// The two horizontal sides of something facing `facing`.
pub(crate) fn sides_of(facing: Dir) -> [Dir; 2] {
    match facing {
        Dir::East | Dir::West => [Dir::North, Dir::South],
        _ => [Dir::West, Dir::East],
    }
}

/// How a device answers its inputs: `None` when its state already matches them, otherwise the
/// delay in ticks and the state it switches to. `container` is the comparator's container reading
/// behind it; `side_lockers` is true when a powered repeater or comparator feeds a repeater's side.
pub fn react(k: Kind, i: [SidePower; 6], range: Range, container: Option<u8>, side_lockers: bool) -> Option<(u32, Kind)> {
    match k {
        Kind::Torch { lit, attached } => {
            let lit_now = i[attached.index()].strength == 0;
            (lit_now != lit).then_some((1, Kind::Torch { lit: lit_now, attached }))
        }
        Kind::Repeater { facing, delay, powered, locked } => {
            if side_lockers != locked {
                Some((1, Kind::Repeater { facing, delay, powered, locked: side_lockers }))
            } else if locked {
                None
            } else {
                let on = i[facing.opposite().index()].strength > 0;
                (on != powered).then_some((delay as u32, Kind::Repeater { facing, delay, powered: on, locked }))
            }
        }
        Kind::Comparator { facing, subtract, output } => {
            let back = i[facing.opposite().index()].strength.max(container.unwrap_or(0));
            let side = sides_of(facing).iter().map(|d| i[d.index()]).filter(|s| s.direct).map(|s| s.strength).max().unwrap_or(0);
            let out = range.clamp(if subtract { back.saturating_sub(side) } else if back >= side { back } else { 0 });
            (out != output).then_some((1, Kind::Comparator { facing, subtract, output: out }))
        }
        Kind::Lamp { lit } => {
            let on = i.iter().any(|s| s.strength > 0);
            (on != lit).then_some((1, Kind::Lamp { lit: on }))
        }
        Kind::Lever { .. } | Kind::Button { .. } | Kind::Plate { .. } | Kind::PowerBlock => None,
    }
}

/// A comparator's reading of a container filled to `filled` (0..=1): 0 when empty, 1 to the
/// world's maximum otherwise.
pub fn container_level(filled: f64, range: Range) -> u8 {
    if filled <= 0.0 {
        return 0;
    }
    1 + (filled.min(1.0) * (range.max() - 1) as f64).floor() as u8
}

/// A compact, stable number for a device state, for saving pending changes.
pub fn encode(k: Kind) -> u32 {
    let d = |d: Dir| d.index() as u32;
    let b = |b: bool| b as u32;
    // Four bits of variant, then its fields.
    let (tag, fields) = match k {
        Kind::Lever { on, attached } => (0, b(on) | d(attached) << 1),
        Kind::Button { on, attached, wood } => (1, b(on) | d(attached) << 1 | b(wood) << 4),
        Kind::Plate { on } => (2, b(on)),
        Kind::PowerBlock => (3, 0),
        Kind::Torch { lit, attached } => (4, b(lit) | d(attached) << 1),
        Kind::Repeater { facing, delay, powered, locked } => (5, d(facing) | (delay as u32 - 1) << 3 | b(powered) << 5 | b(locked) << 6),
        Kind::Comparator { facing, subtract, output } => (6, d(facing) | b(subtract) << 3 | (output as u32) << 4),
        Kind::Lamp { lit } => (7, b(lit)),
    };
    tag | fields << 4
}

pub fn decode(v: u32) -> Option<Kind> {
    let f = v >> 4;
    let dir = |shift: u32| {
        let i = (f >> shift & 7) as usize;
        (i < 6).then(|| Dir::from_index(i))
    };
    let bit = |shift: u32| f >> shift & 1 != 0;
    let k = match v & 15 {
        0 if f < 1 << 4 => Kind::Lever { on: bit(0), attached: dir(1)? },
        1 if f < 1 << 5 => Kind::Button { on: bit(0), attached: dir(1)?, wood: bit(4) },
        2 if f < 2 => Kind::Plate { on: bit(0) },
        3 if f == 0 => Kind::PowerBlock,
        4 if f < 1 << 4 => Kind::Torch { lit: bit(0), attached: dir(1)? },
        5 if f < 1 << 7 => Kind::Repeater { facing: dir(0)?, delay: (f >> 3 & 3) as u8 + 1, powered: bit(5), locked: bit(6) },
        6 if f < 1 << 10 => Kind::Comparator { facing: dir(0)?, subtract: bit(3), output: (f >> 4) as u8 },
        7 if f < 2 => Kind::Lamp { lit: bit(0) },
        _ => return None,
    };
    Some(k)
}

#[cfg(test)]
mod tests {
    use super::*;

    const R15: Range = Range::from_index_const(0);

    fn input(side: Dir, strength: u8, strong: bool, direct: bool) -> [SidePower; 6] {
        let mut i = [SidePower::default(); 6];
        i[side.index()] = SidePower { strength, strong, direct };
        i
    }

    #[test]
    fn torch_inverts_with_one_tick() {
        let t = Kind::Torch { lit: true, attached: Dir::Down };
        assert_eq!(react(t, input(Dir::Down, 4, false, false), R15, None, false), Some((1, Kind::Torch { lit: false, attached: Dir::Down })));
        assert_eq!(react(t, [SidePower::default(); 6], R15, None, false), None);
        let off = Kind::Torch { lit: false, attached: Dir::Down };
        assert_eq!(react(off, [SidePower::default(); 6], R15, None, false), Some((1, t)));
    }

    #[test]
    fn torch_reads_only_attached() {
        let t = Kind::Torch { lit: true, attached: Dir::West };
        assert_eq!(react(t, input(Dir::East, 15, true, true), R15, None, false), None);
        assert_eq!(react(t, input(Dir::Down, 15, true, true), R15, None, false), None);
        assert!(react(t, input(Dir::West, 1, false, false), R15, None, false).is_some());
    }

    #[test]
    fn repeater_full_strength_output() {
        let r = Kind::Repeater { facing: Dir::East, delay: 1, powered: false, locked: false };
        let (_, on) = react(r, input(Dir::West, 3, false, true), R15, None, false).unwrap();
        let out = outputs(on, R15);
        assert_eq!(out[Dir::East.index()], Power { strength: 15, strong: true });
        assert_eq!(out[Dir::West.index()], Power::default());
    }

    #[test]
    fn repeater_delay_is_setting() {
        for delay in 1..=4u8 {
            let r = Kind::Repeater { facing: Dir::North, delay, powered: false, locked: false };
            assert_eq!(react(r, input(Dir::South, 9, false, true), R15, None, false), Some((delay as u32, Kind::Repeater { facing: Dir::North, delay, powered: true, locked: false })));
        }
    }

    #[test]
    fn repeater_locks_then_ignores_input() {
        let r = Kind::Repeater { facing: Dir::East, delay: 2, powered: true, locked: false };
        let (d, locked) = react(r, [SidePower::default(); 6], R15, None, true).unwrap();
        assert_eq!((d, locked), (1, Kind::Repeater { facing: Dir::East, delay: 2, powered: true, locked: true }));
        assert_eq!(react(locked, [SidePower::default(); 6], R15, None, true), None);
        assert_eq!(react(locked, [SidePower::default(); 6], R15, None, false), Some((1, r)));
    }

    #[test]
    fn comparator_compare_and_subtract() {
        let c = |subtract, output| Kind::Comparator { facing: Dir::East, subtract, output };
        let mut i = input(Dir::West, 10, false, true);
        i[Dir::North.index()] = SidePower { strength: 4, strong: false, direct: true };
        assert_eq!(react(c(false, 0), i, R15, None, false), Some((1, c(false, 10))));
        assert_eq!(react(c(true, 0), i, R15, None, false), Some((1, c(true, 6))));
        i[Dir::West.index()].strength = 3;
        assert_eq!(react(c(false, 5), i, R15, None, false), Some((1, c(false, 0))));
        assert_eq!(react(c(true, 0), i, R15, None, false), None);
    }

    #[test]
    fn comparator_side_ignores_powered_block() {
        let c = Kind::Comparator { facing: Dir::East, subtract: true, output: 0 };
        let mut i = input(Dir::West, 10, false, true);
        i[Dir::South.index()] = SidePower { strength: 15, strong: true, direct: false };
        assert_eq!(react(c, i, R15, None, false), Some((1, Kind::Comparator { facing: Dir::East, subtract: true, output: 10 })));
    }

    #[test]
    fn comparator_container_scaled() {
        let c = Kind::Comparator { facing: Dir::East, subtract: false, output: 0 };
        assert_eq!(react(c, [SidePower::default(); 6], R15, Some(7), false), Some((1, Kind::Comparator { facing: Dir::East, subtract: false, output: 7 })));
        assert_eq!(outputs(Kind::Comparator { facing: Dir::East, subtract: false, output: 7 }, R15)[Dir::East.index()], Power { strength: 7, strong: true });
    }

    #[test]
    fn lamp_both_ways_one_tick() {
        assert_eq!(react(Kind::Lamp { lit: false }, input(Dir::Up, 1, false, true), R15, None, false), Some((1, Kind::Lamp { lit: true })));
        assert_eq!(react(Kind::Lamp { lit: true }, [SidePower::default(); 6], R15, None, false), Some((1, Kind::Lamp { lit: false })));
        assert_eq!(react(Kind::Lamp { lit: true }, input(Dir::Up, 1, false, true), R15, None, false), None);
    }

    #[test]
    fn levers_buttons_plates_dont_react() {
        for k in [Kind::Lever { on: false, attached: Dir::Down }, Kind::Button { on: true, attached: Dir::North, wood: true }, Kind::Plate { on: false }, Kind::PowerBlock] {
            assert_eq!(react(k, input(Dir::Up, 15, true, true), R15, None, false), None);
        }
    }

    #[test]
    fn source_outputs() {
        let lever = outputs(Kind::Lever { on: true, attached: Dir::North }, R15);
        assert_eq!(lever[Dir::North.index()], Power { strength: 15, strong: true });
        assert_eq!(lever[Dir::Up.index()], Power { strength: 15, strong: false });
        assert_eq!(outputs(Kind::Lever { on: false, attached: Dir::North }, R15), [Power::default(); 6]);
        let torch = outputs(Kind::Torch { lit: true, attached: Dir::West }, R15);
        assert_eq!(torch[Dir::Up.index()], Power { strength: 15, strong: true });
        assert_eq!(torch[Dir::West.index()], Power::default());
        assert_eq!(torch[Dir::East.index()], Power { strength: 15, strong: false });
        assert!(outputs(Kind::PowerBlock, R15).iter().all(|p| *p == Power { strength: 15, strong: false }));
        assert_eq!(outputs(Kind::Lamp { lit: true }, R15), [Power::default(); 6]);
    }

    #[test]
    fn connect_sides() {
        let bit = |d: Dir| 1u8 << d.index();
        assert_eq!(connects(Kind::Repeater { facing: Dir::East, delay: 1, powered: false, locked: false }), bit(Dir::East) | bit(Dir::West));
        assert_eq!(connects(Kind::Comparator { facing: Dir::North, subtract: false, output: 0 }), bit(Dir::North) | bit(Dir::South) | bit(Dir::East) | bit(Dir::West));
        assert_eq!(connects(Kind::Lamp { lit: false }), 0);
        assert_eq!(connects(Kind::Torch { lit: false, attached: Dir::Down }), 0b111111);
    }

    #[test]
    fn outputs_clamped_to_range() {
        for (i, max) in [(0, 15), (1, 31), (2, 63)] {
            let r = Range::from_index(i);
            assert_eq!(outputs(Kind::PowerBlock, r)[0].strength, max);
            assert_eq!(outputs(Kind::Comparator { facing: Dir::East, subtract: false, output: 40 }, r)[Dir::East.index()].strength, 40.min(max));
        }
    }

    #[test]
    fn container_level_scales() {
        for i in 0..3 {
            let r = Range::from_index(i);
            assert_eq!(container_level(0.0, r), 0);
            assert!(container_level(1.0 / 1728.0, r) >= 1);
            assert_eq!(container_level(1.0, r), r.max());
        }
    }

    #[test]
    fn encode_round_trip() {
        let mut all = vec![Kind::PowerBlock, Kind::Plate { on: false }, Kind::Plate { on: true }, Kind::Lamp { lit: false }, Kind::Lamp { lit: true }];
        for d in Dir::ALL {
            for b in [false, true] {
                all.push(Kind::Lever { on: b, attached: d });
                all.push(Kind::Button { on: b, attached: d, wood: !b });
                all.push(Kind::Button { on: b, attached: d, wood: b });
                all.push(Kind::Torch { lit: b, attached: d });
                for delay in 1..=4 {
                    all.push(Kind::Repeater { facing: d, delay, powered: b, locked: !b });
                    all.push(Kind::Repeater { facing: d, delay, powered: b, locked: b });
                }
                for output in [0, 1, 31, 63] {
                    all.push(Kind::Comparator { facing: d, subtract: b, output });
                }
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for k in all {
            let v = encode(k);
            assert_eq!(decode(v), Some(k));
            assert!(seen.insert(v), "{k:?} collides");
        }
        assert_eq!(decode(u32::MAX), None);
    }
}
