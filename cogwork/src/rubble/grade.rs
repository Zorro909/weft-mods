//! How rich Ore Rubble is. Every quarry brings it up Poor; processing raises it a grade at a
//! time (washing, flotation, leaching), and grinding a richer grade gives more raw ore.

use strum::{FromRepr, IntoStaticStr, VariantArray};

/// The grade of Ore Rubble. Kept on a stack as its number (`GRADE`), so the order is part of the
/// save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Grade {
    #[default]
    Poor,
    Common,
    Rich,
    Pure,
}

impl Grade {
    /// The grade kept as `number`; one from a newer cogwork reads as Poor.
    pub fn from_saved(number: u32) -> Grade {
        Grade::from_repr(number).unwrap_or_default()
    }

    /// Raw ore one Ore Rubble of this grade grinds into, the fraction a chance of one more.
    pub fn raw_ore(self) -> f64 {
        match self {
            Grade::Poor => 1.0,
            Grade::Common => 1.5,
            Grade::Rich => 2.0,
            Grade::Pure => 2.5,
        }
    }

    /// The most raw ore one Ore Rubble of any grade grinds into.
    pub const MOST_RAW_ORE: u32 = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades_keep_their_saved_numbers() {
        let numbers: Vec<u32> = Grade::VARIANTS.iter().map(|&grade| grade as u32).collect();
        assert_eq!(numbers, [0, 1, 2, 3], "poor, common, rich, pure");
        assert_eq!(Grade::from_saved(2), Grade::Rich, "a saved rich");
    }

    #[test]
    fn an_unknown_saved_grade_reads_as_poor() {
        assert_eq!(Grade::from_saved(99), Grade::Poor, "from a newer cogwork");
    }

    #[test]
    fn a_richer_grade_grinds_into_more() {
        let yields: Vec<f64> = Grade::VARIANTS
            .iter()
            .map(|grade| grade.raw_ore())
            .collect();
        assert_eq!(yields, [1.0, 1.5, 2.0, 2.5], "poor 1 to pure 2.5");
        let most = yields.iter().map(|amount| amount.ceil() as u32).max();
        assert_eq!(most, Some(Grade::MOST_RAW_ORE), "never more than the most");
    }

    #[test]
    fn every_grade_has_a_name() {
        for &grade in Grade::VARIANTS {
            let key = format!("cogwork.grade.{}", <&str>::from(grade));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }
}
