use std::fmt::{Display, Formatter, Result};

use itertools::Itertools;
use strum::{EnumIter, IntoEnumIterator};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum DiceValue {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}

impl Display for DiceValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let char = match self {
            DiceValue::One => '⚀',
            DiceValue::Two => '⚁',
            DiceValue::Three => '⚂',
            DiceValue::Four => '⚃',
            DiceValue::Five => '⚄',
            DiceValue::Six => '⚅',
        };

        write!(f, "{}", char)
    }
}

pub fn dice_rolls() -> impl Iterator<Item = (DiceValue, DiceValue, u8, bool)> {
    DiceValue::iter()
        .cartesian_product(DiceValue::iter())
        .sorted_by(|&(a1, a2), &(b1, b2)| {
            (a1 as u8 + a2 as u8)
                .cmp(&(b1 as u8 + b2 as u8))
                .then((a1 as u8).cmp(&(b1 as u8)))
                .then((a2 as u8).cmp(&(b2 as u8)))
        })
        .map(|(d1, d2)| {
            let sum = d1 as u8 + d2 as u8;
            let double = d1 == d2;

            (d1, d2, sum, double)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        let count = dice_rolls().count();

        assert_eq!(count, 36);
    }

    #[test]
    fn test_doubles() {
        let count = dice_rolls().filter(|(_, _, _, double)| *double).count();

        assert_eq!(count, 6);
    }
}
