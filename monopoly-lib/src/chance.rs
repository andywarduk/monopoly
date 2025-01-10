use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CHCard {
    GoGo,
    GoJail,
    GoProperty(u8, u8),
    GoRail(u8),
    GoNextRail,
    GoNextUtil,
    Back3,
    Inconsequential,
}

pub const CHCARDS: usize = 16;

impl CHCard {
    pub fn build_deck() -> VecDeque<CHCard> {
        let mut deck = VecDeque::new();

        deck.extend([
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
        ]);

        while deck.len() < CHCARDS {
            deck.push_back(CHCard::Inconsequential);
        }

        deck
    }
}
