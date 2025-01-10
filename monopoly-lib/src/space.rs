use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Space {
    Go,
    Visit,
    FreeParking,
    GoToJail,
    Property(u8, u8),
    Rail(u8),
    Utility(u8),
    CommunityChest(u8),
    Chance(u8),
    Tax(u8),
}

impl Space {
    pub fn shortdesc(&self) -> String {
        match self {
            Go => "Go".to_string(),
            Visit => "Jail".to_string(),
            FreeParking => "Free".to_string(),
            GoToJail => "ToJail".to_string(),
            Property(set, i) => format!("{}{}", (set + b'A') as char, i + 1),
            Rail(i) => format!("R{}", i + 1),
            Utility(i) => format!("U{}", i + 1),
            CommunityChest(i) => format!("CC{}", i + 1),
            Chance(i) => format!("CH{}", i + 1),
            Tax(i) => format!("T{}", i + 1),
        }
    }

    pub fn set(&self) -> PropertySet {
        match self {
            Go | Visit | FreeParking | GoToJail => PropertySet::Other,
            Property(set, _) => match *set {
                0 => PropertySet::Brown,
                1 => PropertySet::LightBlue,
                2 => PropertySet::Pink,
                3 => PropertySet::Orange,
                4 => PropertySet::Red,
                5 => PropertySet::Yellow,
                6 => PropertySet::Green,
                7 => PropertySet::DarkBlue,
                _ => panic!("Invalid set"),
            },
            Rail(_) => PropertySet::Station,
            Utility(_) => PropertySet::Utility,
            CommunityChest(_) => PropertySet::CommunityChest,
            Chance(_) => PropertySet::Chance,
            Tax(_) => PropertySet::Tax,
        }
    }

    pub fn find(space: Space) -> usize {
        SPACES
            .iter()
            .position(|s| *s == space)
            .unwrap_or_else(|| panic!("Space {space:?} not found"))
    }

    pub fn next_rail(position: usize) -> usize {
        Self::find_next(position, |s| matches!(s, Space::Rail(_)))
    }

    pub fn next_util(position: usize) -> usize {
        Self::find_next(position, |s| matches!(s, Space::Utility(_)))
    }

    fn find_next<F>(position: usize, check: F) -> usize
    where
        F: Fn(&Space) -> bool,
    {
        for i in (position + 1)..(position + SPACES.len()) {
            let elem = i % SPACES.len();

            if check(&SPACES[elem]) {
                return elem;
            }
        }

        panic!("Next space not found")
    }
}

use Space::*;

pub const SPACECOUNT: usize = 40;

pub const SPACES: [Space; SPACECOUNT] = [
    Go,
    Property(0, 0),
    CommunityChest(0),
    Property(0, 1),
    Tax(0),
    Rail(0),
    Property(1, 0),
    Chance(0),
    Property(1, 1),
    Property(1, 2),
    Visit,
    Property(2, 0),
    Utility(0),
    Property(2, 1),
    Property(2, 2),
    Rail(1),
    Property(3, 0),
    CommunityChest(1),
    Property(3, 1),
    Property(3, 2),
    FreeParking,
    Property(4, 0),
    Chance(1),
    Property(4, 1),
    Property(4, 2),
    Rail(2),
    Property(5, 0),
    Property(5, 1),
    Utility(1),
    Property(5, 2),
    GoToJail,
    Property(6, 0),
    Property(6, 1),
    CommunityChest(2),
    Property(6, 2),
    Rail(3),
    Chance(2),
    Property(7, 0),
    Tax(1),
    Property(7, 1),
];

#[repr(u8)]
#[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum PropertySet {
    Brown,
    LightBlue,
    Pink,
    Orange,
    Red,
    Yellow,
    Green,
    DarkBlue,
    Station,
    Utility,
    Chance,
    CommunityChest,
    Tax,
    Other,
}
