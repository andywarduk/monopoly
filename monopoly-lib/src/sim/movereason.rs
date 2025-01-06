use num_derive::FromPrimitive;
use strum::{EnumCount, EnumIter};

#[derive(Debug, PartialEq, Eq, FromPrimitive, EnumCount, EnumIter, Clone)]
pub enum MoveReason {
    Roll = -1,        // Normal roll
    CHCard = 0,       // Chance card
    CCCard = 1,       // Community chest card
    GoToJail = 2,     // Go to jail space
    TripleDouble = 3, // Triple double rolled
    NoDouble = 4,     // In jail and not rolled a double
    ExitJail = 5,     // Exited jail
}

impl std::fmt::Display for MoveReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveReason::Roll => f.write_str("Rolled"),
            MoveReason::CHCard => f.write_str("Chance"),
            MoveReason::CCCard => f.write_str("Community Chest"),
            MoveReason::GoToJail => f.write_str("Go to Jail"),
            MoveReason::TripleDouble => f.write_str("Triple Double"),
            MoveReason::NoDouble => f.write_str("Double Not Rolled"),
            MoveReason::ExitJail => f.write_str("Released from Jail"),
        }
    }
}
