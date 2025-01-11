use num_derive::FromPrimitive;
pub use strum::IntoEnumIterator;
use strum::{EnumCount, EnumIter};

#[derive(Debug, PartialEq, Eq, FromPrimitive, EnumCount, EnumIter, Clone, Copy)]
pub enum MoveReason {
    Roll = -1,        // Normal roll
    CHCard = 0,       // Chance card
    CCCard = 1,       // Community chest card
    CHCardCCCard = 2, //Chance card -> Community chest card
    GoToJail = 3,     // Go to jail space
    TripleDouble = 4, // Triple double rolled
    NoDouble = 5,     // In jail and not rolled a double
    ExitJail = 6,     // Exited jail
}

impl MoveReason {
    pub const fn uint_count() -> usize {
        MoveReason::COUNT - 1
    }
}

impl std::fmt::Display for MoveReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            MoveReason::Roll => "Rolled",
            MoveReason::CHCard => "Chance",
            MoveReason::CCCard => "Community Chest",
            MoveReason::CHCardCCCard => "Chance ➔ Community Chest",
            MoveReason::GoToJail => "Go to Jail",
            MoveReason::TripleDouble => "Triple Double",
            MoveReason::NoDouble => "Double Not Rolled",
            MoveReason::ExitJail => "Released from Jail",
        };

        desc.fmt(f)
    }
}
