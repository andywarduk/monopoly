use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CCCard {
    GoGo,
    GoJail,
    Inconsequential,
}

pub const CCCARDS: usize = 16;

impl CCCard {
    pub fn build_deck() -> VecDeque<CCCard> {
        let mut deck = VecDeque::new();

        deck.extend([CCCard::GoJail, CCCard::GoGo]);

        while deck.len() < CCCARDS {
            deck.push_back(CCCard::Inconsequential);
        }

        deck
    }
}
