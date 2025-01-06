use std::collections::VecDeque;

use crate::chance::CHCard;
use crate::commchest::CCCard;
use crate::space::{SPACES, Space};
use crate::strategy::Strategy;
use movereason::MoveReason;
use rand::prelude::*;
use strum::EnumCount;

pub mod movereason;

type CardChoose<T> = fn(rng: &mut ThreadRng, &mut VecDeque<T>) -> T;
type DiceRollCb = fn(board: &mut Board, doubles: usize) -> (u8, u8);

#[derive(Debug)]
pub struct Board {
    strategy: Strategy,
    position: usize,
    jailroll: u8,
    ccdeck: VecDeque<CCCard>,
    cccardchoose: CardChoose<CCCard>,
    chdeck: VecDeque<CHCard>,
    chcardchoose: CardChoose<CHCard>,
    arrivals: [u64; SPACES.len()],
    arrival_reason: [[u64; MoveReason::COUNT]; SPACES.len()],
    moves: u64,
    turns: u64,
    doubles: [u64; 3],
    rollfreq: [u64; 11],
    rng: ThreadRng,
}

impl Default for Board {
    fn default() -> Self {
        // Default pay to get out of jail, cycle card decks when choosing
        Self::new(Strategy::PayJail, false)
    }
}

impl Board {
    /// Create a new board with a given strategy and car selection method
    pub fn new(strategy: Strategy, randomcard: bool) -> Self {
        // Create random number generator
        let mut rng = thread_rng();

        // Set up community chest card deck
        let mut ccdeck = CCCard::build_deck();
        Self::shuffle_deck(&mut rng, &mut ccdeck);

        // Set up chance card deck
        let mut chdeck = CHCard::build_deck();
        Self::shuffle_deck(&mut rng, &mut chdeck);

        Self {
            strategy,
            position: 0,
            jailroll: 0,
            ccdeck,
            cccardchoose: if randomcard { Self::random_card } else { Self::next_card },
            chdeck,
            chcardchoose: if randomcard { Self::random_card } else { Self::next_card },
            arrivals: [0; SPACES.len()],
            arrival_reason: [[0; MoveReason::COUNT]; SPACES.len()],
            moves: 0,
            turns: 0,
            doubles: [0; 3],
            rollfreq: [0; 11],
            rng,
        }
    }

    // Take a turn (may involve several moves when rolling double)
    pub fn turn(&mut self) {
        self.turn_with_dice(|board, _| board.roll_dice());
    }

    /// Take a turn with a callback to get dice rolls
    fn turn_with_dice(&mut self, dice: DiceRollCb) {
        // Increment turns
        self.turns += 1;

        // Keep track of doubles
        let mut doubles = 0;

        loop {
            // Roll the dice
            let (d1, d2) = dice(self, doubles);

            // Calculate total
            let total = d1 + d2;

            // Count rolled sum
            self.rollfreq[total as usize - 2] += 1;

            // Thrown a double?
            let double = d1 == d2;

            if self.jailroll > 0 && self.strategy == Strategy::JailWait {
                // In jail, rolling to exit
                if double {
                    // Rolled a double - player moves but does not get another go
                    self.move_to((self.position + total as usize) % SPACES.len(), MoveReason::Roll);
                } else {
                    // Not rolled a double
                    self.jailroll += 1;

                    if self.jailroll == 4 {
                        // Not rolled a double in 3 goes - move to just visiting
                        self.jailroll = 0;
                        self.update_arrivals(MoveReason::ExitJail);
                    } else {
                        self.update_arrivals(MoveReason::NoDouble);
                    }
                }

                // Player does not get another roll even if a double is rolled
                break;
            }

            if double {
                // Count doubles
                doubles += 1;

                if doubles == 3 {
                    // 3 doubles in a row - go to jail
                    self.move_to(Space::find(Space::Visit), MoveReason::TripleDouble);
                    break;
                }
            }

            // Make the move
            self.move_to((self.position + total as usize) % SPACES.len(), MoveReason::Roll);

            // If not rolled a double or in jail then go is over
            if !double || self.jailroll > 0 {
                break;
            }
        }

        // Count doubles (not cumulative)
        if doubles > 0 {
            self.doubles[doubles - 1] += 1;
        }
    }

    /// Returns the strategy in use
    pub fn strategy(&self) -> Strategy {
        self.strategy
    }

    /// Returns reference to the space arrival count array
    pub fn arrivals(&self) -> &[u64] {
        &self.arrivals
    }

    /// Returns number of arrivals for a given space
    pub fn arrivals_on(&self, elem: usize) -> u64 {
        self.arrivals[elem]
    }

    /// Returns reference to the space arrival reason array
    pub fn arrival_reasons(&self) -> &[[u64; MoveReason::COUNT]; SPACES.len()] {
        &self.arrival_reason
    }

    /// Returns arrival reasons for a given space
    pub fn arrival_reasons_on(&self, elem: usize) -> &[u64] {
        &self.arrival_reason[elem]
    }

    /// Returns the number of moves made
    pub fn moves(&self) -> u64 {
        self.moves
    }

    /// Returns the number of turns taken (can involve >1 move)
    pub fn turns(&self) -> u64 {
        self.turns
    }

    /// Returns a reference to the doubles count array
    pub fn doubles(&self) -> &[u64] {
        &self.doubles
    }

    /// Returns the number of turns with a given number of doubles thrown
    pub fn doubles_elem(&self, elem: usize) -> u64 {
        self.doubles[elem]
    }

    /// Returns a reference to the roll sum frequencies
    pub fn rollfreq(&self) -> &[u64] {
        &self.rollfreq
    }

    /// Move to a given space with a move reason
    fn move_to(&mut self, elem: usize, reason: MoveReason) {
        // Set current position
        self.position = elem;

        // Perform any actions necessary
        match SPACES[self.position] {
            Space::GoToJail => self.move_to(Space::find(Space::Visit), MoveReason::GoToJail),
            Space::CommunityChest(_) => self.draw_community_chest(),
            Space::Chance(_) => self.draw_chance(),
            _ => (),
        }

        // Jumped to a different space?
        if self.position == elem {
            // No - update state and statistics
            if SPACES[self.position] == Space::Visit && reason != MoveReason::Roll && reason != MoveReason::ExitJail {
                // Going in to jail
                self.jailroll = 1;
            } else {
                // Not in jail
                self.jailroll = 0;
            }

            // Update arrival counts and reasons
            self.update_arrivals(reason);
        }
    }

    /// Update statistics when arriving on a space
    fn update_arrivals(&mut self, reason: MoveReason) {
        let recordelem = if SPACES[self.position] == Space::Visit {
            match reason {
                MoveReason::Roll | MoveReason::ExitJail => self.position,
                _ => {
                    // Reasons for entering jail are recorded on the 'Go to jail' space
                    Space::find(Space::GoToJail)
                }
            }
        } else {
            self.position
        };

        // Record arrival at this space
        self.arrivals[recordelem] += 1;

        // Record move
        self.moves += 1;

        // Record move reason (all except Rolled)
        let reason_elem = reason as isize;

        if reason_elem >= 0 {
            self.arrival_reason[recordelem][reason_elem as usize] += 1;
        }
    }

    /// Shuffles a deck of cards
    fn shuffle_deck<T: Copy>(rng: &mut ThreadRng, deck: &mut VecDeque<T>) {
        for _ in 0..(deck.len() * 4) {
            let elem = rng.gen_range(0..(deck.len()));

            let card = deck.remove(elem).unwrap();

            deck.push_back(card);
        }
    }

    /// Choose a randon card
    fn random_card<T: Copy>(rng: &mut ThreadRng, deck: &mut VecDeque<T>) -> T {
        let elem = rng.gen_range(0..deck.len());
        deck[elem]
    }

    /// Choose the next card
    fn next_card<T: Copy>(_rng: &mut ThreadRng, deck: &mut VecDeque<T>) -> T {
        let card = deck.pop_front().unwrap();
        deck.push_back(card);

        card
    }

    /// Draw a chance card and action it
    fn draw_chance(&mut self) {
        let card = (self.chcardchoose)(&mut self.rng, &mut self.chdeck);

        match card {
            CHCard::GoGo => self.move_to(Space::find(Space::Go), MoveReason::CHCard),
            CHCard::GoJail => self.move_to(Space::find(Space::Visit), MoveReason::CHCard),
            CHCard::GoProperty(set, n) => self.move_to(Space::find(Space::Property(set, n)), MoveReason::CHCard),
            CHCard::GoRail(n) => self.move_to(Space::find(Space::Rail(n)), MoveReason::CHCard),
            CHCard::GoNextRail => self.move_to(Space::next_rail(self.position), MoveReason::CHCard),
            CHCard::GoNextUtil => self.move_to(Space::next_util(self.position), MoveReason::CHCard),
            CHCard::Back3 => self.move_to(self.position - 3, MoveReason::CHCard),
            _ => (),
        }
    }

    /// Draw a community chest card and action it
    fn draw_community_chest(&mut self) {
        let card = (self.cccardchoose)(&mut self.rng, &mut self.ccdeck);

        match card {
            CCCard::GoGo => self.move_to(Space::find(Space::Go), MoveReason::CCCard),
            CCCard::GoJail => self.move_to(Space::find(Space::Visit), MoveReason::CCCard),
            _ => (),
        }
    }

    /// Roll both dice and return values
    fn roll_dice(&mut self) -> (u8, u8) {
        (self.die_roll(), self.die_roll())
    }

    /// Roll a single die
    fn die_roll(&mut self) -> u8 {
        self.rng.gen_range(1..=6)
    }
}

#[cfg(test)]
mod tests;
