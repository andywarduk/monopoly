use std::collections::VecDeque;

use crate::chance::CHCard;
use crate::commchest::CCCard;
use crate::space::{SPACES, Space};
use crate::strategy::Strategy;
use movereason::MoveReason;
use rand::prelude::*;
use strum::EnumCount;

pub mod movereason;

#[derive(Debug)]
pub struct Board {
    strategy: Strategy,
    position: usize,
    jailroll: u8,
    ccdeck: VecDeque<CCCard>,
    cccardchoose: fn(board: &mut Board) -> CCCard,
    chdeck: VecDeque<CHCard>,
    chcardchoose: fn(board: &mut Board) -> CHCard,
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
        Self::new(Strategy::PayJail, false)
    }
}

impl Board {
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
            cccardchoose: if randomcard { Self::random_cc } else { Self::next_cc },
            chdeck,
            chcardchoose: if randomcard { Self::random_ch } else { Self::next_ch },
            arrivals: [0; SPACES.len()],
            arrival_reason: [[0; MoveReason::COUNT]; SPACES.len()],
            moves: 0,
            turns: 0,
            doubles: [0; 3],
            rollfreq: [0; 11],
            rng,
        }
    }

    pub fn turn(&mut self) {
        self.turn_with_dice(|board, _| (board.dice_roll(), board.dice_roll()));
    }

    pub fn turn_with_dice<F>(&mut self, dice: F)
    where
        F: Fn(&mut Self, usize) -> (u8, u8),
    {
        self.turns += 1;

        let mut doubles = 0;

        loop {
            // Roll the dice
            let (d1, d2) = dice(self, doubles);

            // Calculate total
            let total = d1 + d2;

            // Count roll
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
                        // TODO currently marked as a normal roll
                        self.update_arrivals(MoveReason::Roll);
                    } else {
                        self.update_arrivals(MoveReason::NoDouble);
                    }
                }

                break;
            }

            if double {
                // Count doubles
                doubles += 1;

                if doubles == 3 {
                    // 3 doubles in a row - go to jail
                    self.move_to(Space::find(Space::Jail), MoveReason::TripleDouble);
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

    pub fn strategy(&self) -> Strategy {
        self.strategy
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

    pub fn rollfreq(&self) -> &[u64] {
        &self.rollfreq
    }

    fn shuffle_deck<T: Copy>(rng: &mut ThreadRng, deck: &mut VecDeque<T>) {
        for _ in 0..(deck.len() * 4) {
            let elem = rng.gen_range(0..(deck.len()));

            let card = deck.remove(elem).unwrap();

            deck.push_back(card);
        }
    }

    fn move_to(&mut self, elem: usize, reason: MoveReason) {
        self.position = elem;

        match SPACES[self.position] {
            Space::GoToJail => self.move_to(Space::find(Space::Jail), MoveReason::GoToJail),
            Space::CommunityChest(_) => self.draw_community_chest(),
            Space::Chance(_) => self.draw_chance(),
            _ => (),
        }

        // Jumped to a different space?
        if self.position == elem {
            // No - update state and statistics
            if SPACES[self.position] == Space::Jail && reason != MoveReason::Roll {
                // Going in to jail
                self.jailroll = 1;
            } else {
                // Not in jail
                self.jailroll = 0;
            }

            self.update_arrivals(reason);
        }
    }

    fn update_arrivals(&mut self, reason: MoveReason) {
        self.arrivals[self.position] += 1;
        self.moves += 1;

        let reason_elem = reason as isize;

        if reason_elem >= 0 {
            self.arrival_reason[self.position][reason_elem as usize] += 1;
        }
    }

    fn random_ch(board: &mut Board) -> CHCard {
        let elem = board.rng.gen_range(0..board.chdeck.len());
        board.chdeck[elem]
    }

    fn next_ch(board: &mut Board) -> CHCard {
        let card = board.chdeck.pop_front().unwrap();
        board.chdeck.push_back(card);

        card
    }

    fn draw_chance(&mut self) {
        let card = (self.chcardchoose)(self);

        match card {
            CHCard::GoGo => self.move_to(Space::find(Space::Go), MoveReason::CHCard),
            CHCard::GoJail => self.move_to(Space::find(Space::Jail), MoveReason::CHCard),
            CHCard::GoProperty(set, n) => self.move_to(Space::find(Space::Property(set, n)), MoveReason::CHCard),
            CHCard::GoRail(n) => self.move_to(Space::find(Space::Rail(n)), MoveReason::CHCard),
            CHCard::GoNextRail => self.move_to(Space::next_rail(self.position), MoveReason::CHCard),
            CHCard::GoNextUtil => self.move_to(Space::next_util(self.position), MoveReason::CHCard),
            CHCard::Back3 => self.move_to(self.position - 3, MoveReason::CHCard),
            _ => (),
        }
    }

    fn random_cc(board: &mut Board) -> CCCard {
        let elem = board.rng.gen_range(0..board.chdeck.len());
        board.ccdeck[elem]
    }

    fn next_cc(board: &mut Board) -> CCCard {
        let card = board.ccdeck.pop_front().unwrap();
        board.ccdeck.push_back(card);

        card
    }

    fn draw_community_chest(&mut self) {
        let card = (self.cccardchoose)(self);

        match card {
            CCCard::GoGo => self.move_to(Space::find(Space::Go), MoveReason::CCCard),
            CCCard::GoJail => self.move_to(Space::find(Space::Jail), MoveReason::CCCard),
            _ => (),
        }
    }

    fn dice_roll(&mut self) -> u8 {
        self.rng.gen_range(1..=6)
    }
}

#[cfg(test)]
mod tests;
