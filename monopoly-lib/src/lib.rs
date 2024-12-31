use std::collections::VecDeque;

use num_derive::FromPrimitive;
use rand::prelude::*;

// Constants
const SPACES: usize = 40;
const CARDS: usize = 16;
const REASONS: usize = 4;

pub struct Board {
    position: usize,
    spaces: [Space; SPACES],
    ccdeck: VecDeque<CCCard>,
    chdeck: VecDeque<CHCard>,
    arrivals: [u64; SPACES],
    arrival_reason: [[u64; REASONS]; SPACES],
    moves: u64,
    turns: u64,
    doubles: [u64; 3],
    rng: ThreadRng,
}

impl Default for Board {
    fn default() -> Self {
        // Create random number generator
        let mut rng = thread_rng();

        // Set up spaces
        use Space::*;

        let spaces = [
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
            Jail,
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

        // Set up community chest card deck
        let ccdeck = Self::shuffle_deck(
            &mut rng,
            vec![CCCard::GoJail, CCCard::GoGo],
            CCCard::Inconsequential,
        );

        // Set up chance card deck
        let chdeck = Self::shuffle_deck(
            &mut rng,
            vec![
                CHCard::GoGo,
                CHCard::GoJail,
                CHCard::GoProperty(2, 0),
                CHCard::GoProperty(4, 2),
                CHCard::GoProperty(7, 1),
                CHCard::GoRail(0),
                CHCard::GoNextRail,
                CHCard::GoNextRail,
                CHCard::GoNextUtil,
                CHCard::Back3,
            ],
            CHCard::Inconsequential,
        );

        Self {
            position: 0,
            spaces,
            ccdeck,
            chdeck,
            arrivals: [0; SPACES],
            arrival_reason: [[0; REASONS]; SPACES],
            moves: 0,
            turns: 0,
            doubles: [0; 3],
            rng,
        }
    }
}

impl Board {
    pub fn turn(&mut self) {
        self.turns += 1;

        let mut doubles = 0;

        loop {
            // Roll the dice
            let d1 = self.dice_roll();
            let d2 = self.dice_roll();

            // Calculate total
            let total = d1 + d2;

            // Thrown a double?
            let double = d1 == d2;

            if double {
                // Count doubles
                doubles += 1;

                if doubles == 3 {
                    // 2 doubles in a row - go to jail
                    self.move_to(self.find_space(Space::Jail), MoveReason::TripleDouble);
                    break;
                }
            }

            // Make the move
            self.move_to((self.position + total as usize) % SPACES, MoveReason::Roll);

            // If not rolled a double then go is over
            if !double {
                break;
            }
        }

        // Count doubles (not cumulative)
        if doubles > 0 {
            self.doubles[doubles - 1] += 1;
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn space(&self, elem: usize) -> &Space {
        &self.spaces[elem]
    }

    pub fn spaces(&self) -> &[Space] {
        &self.spaces
    }

    pub fn arrivals(&self) -> &[u64] {
        &self.arrivals
    }

    pub fn arrivals_on(&self, elem: usize) -> u64 {
        self.arrivals[elem]
    }

    pub fn arrival_reasons(&self, elem: usize) -> &[u64] {
        &self.arrival_reason[elem]
    }

    pub fn moves(&self) -> u64 {
        self.moves
    }

    pub fn turns(&self) -> u64 {
        self.turns
    }

    pub fn doubles(&self) -> &[u64] {
        &self.doubles
    }

    pub fn doubles_elem(&self, elem: usize) -> u64 {
        self.doubles[elem]
    }

    fn shuffle_deck<T: Copy>(rng: &mut ThreadRng, mut add: Vec<T>, default: T) -> VecDeque<T> {
        let mut deck = VecDeque::new();

        while add.len() < CARDS {
            add.push(default);
        }

        while !add.is_empty() {
            let elem = rng.gen_range(0..(add.len()));

            let card = add.swap_remove(elem);

            deck.push_back(card);
        }

        deck
    }

    fn move_to(&mut self, elem: usize, reason: MoveReason) {
        self.position = elem;

        match self.spaces[self.position] {
            Space::GoToJail => self.move_to(self.find_space(Space::Jail), MoveReason::GoToJail),
            Space::CommunityChest(_) => self.draw_community_chest(),
            Space::Chance(_) => self.draw_chance(),
            _ => (),
        }

        if self.position == elem {
            self.arrivals[self.position] += 1;
            self.moves += 1;

            let reason_elem = reason as isize;

            if reason_elem >= 0 {
                self.arrival_reason[self.position][reason_elem as usize] += 1;
            }
        }
    }

    fn find_space(&self, space: Space) -> usize {
        self.spaces.iter().position(|s| *s == space).unwrap()
    }

    fn find_next<F>(&self, check: F) -> usize
    where
        F: Fn(&Space) -> bool,
    {
        for i in (self.position)..(self.position + SPACES) {
            let elem = i % SPACES;

            if check(&self.spaces[elem]) {
                return elem;
            }
        }

        panic!("Next space not found")
    }

    fn draw_chance(&mut self) {
        let card = self.chdeck.pop_front().unwrap();

        match card {
            CHCard::GoGo => self.move_to(self.find_space(Space::Go), MoveReason::CHCard),
            CHCard::GoJail => self.move_to(self.find_space(Space::Jail), MoveReason::CHCard),
            CHCard::GoProperty(set, n) => {
                self.move_to(self.find_space(Space::Property(set, n)), MoveReason::CHCard)
            }
            CHCard::GoRail(n) => self.move_to(self.find_space(Space::Rail(n)), MoveReason::CHCard),
            CHCard::GoNextRail => self.move_to(
                self.find_next(|s| matches!(s, Space::Rail(_))),
                MoveReason::CHCard,
            ),
            CHCard::GoNextUtil => self.move_to(
                self.find_next(|s| matches!(s, Space::Utility(_))),
                MoveReason::CHCard,
            ),
            CHCard::Back3 => self.move_to(self.position - 3, MoveReason::CHCard),
            _ => (),
        }

        self.chdeck.push_back(card)
    }

    fn draw_community_chest(&mut self) {
        let card = self.ccdeck.pop_front().unwrap();

        match card {
            CCCard::GoGo => self.move_to(self.find_space(Space::Go), MoveReason::CCCard),
            CCCard::GoJail => self.move_to(self.find_space(Space::Jail), MoveReason::CCCard),
            _ => (),
        }

        self.ccdeck.push_back(card)
    }

    fn dice_roll(&mut self) -> u8 {
        self.rng.gen_range(1..=6)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Space {
    Go,
    Jail,
    FreeParking,
    GoToJail,
    Property(u8, u8),
    Rail(u8),
    Utility(u8),
    CommunityChest(u8),
    Chance(u8),
    Tax(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CCCard {
    GoGo,
    GoJail,
    Inconsequential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CHCard {
    GoGo,
    GoJail,
    GoProperty(u8, u8),
    GoRail(u8),
    GoNextRail,
    GoNextUtil,
    Back3,
    Inconsequential,
}

#[derive(FromPrimitive)]
pub enum MoveReason {
    Roll = -1,
    CHCard = 0,
    CCCard = 1,
    GoToJail = 2,
    TripleDouble = 3,
}

impl std::fmt::Display for MoveReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveReason::Roll => f.write_str("Rolled"),
            MoveReason::CHCard => f.write_str("Chance"),
            MoveReason::CCCard => f.write_str("Community Chest"),
            MoveReason::GoToJail => f.write_str("Go to Jail"),
            MoveReason::TripleDouble => f.write_str("Triple Double"),
        }
    }
}
