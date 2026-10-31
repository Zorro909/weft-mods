use crate::Dir;

/// The built-in devices and their state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
