//! AI module
//!
//! Defines the interface for the AI players and reexports them from their respective submodules.
//!
mod greedy;
mod simple;

use crate::core::{self, Action, Addr, CardView, Suit, Value};
pub use greedy::GreedyAi;
pub use simple::SimpleAi;
use std::hash::Hash;

pub trait Ai {
    /// Ask the AI to suggest an action
    ///
    /// The action must be valid for the current game state
    fn make_move(&mut self) -> Action;

    /// The name of the AI.
    /// Used for reporting and statistics.
    fn name(&self) -> &'static str;

    /// Update the AI with the result of an action
    /// If the action reveals a card, the suit and value of the card is given, otherwise None
    fn update(&mut self, action: Action, res: Option<(core::Suit, core::Value)>);
}

/// A helper struct for the AI
/// It holds the known information about the game state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolitaireObserver {
    pub talon_size: usize,
    pub waste: Vec<(Suit, Value)>,
    pub foundation_tops: [Option<(Suit, Value)>; 4],
    pub depots: [Vec<CardView>; 7],
}

impl SolitaireObserver {
    pub fn is_won(&self) -> bool {
        self.foundation_tops
            .iter()
            .all(|f| matches!(f, Some((_, Value::KING))))
    }

    /// For some address, how many face card can we pick?
    pub fn n_takeable_cards(&self, addr: &Addr) -> usize {
        match addr {
            Addr::Waste => !self.waste.is_empty() as usize,
            Addr::Foundation1 | Addr::Foundation2 | Addr::Foundation3 | Addr::Foundation4 => {
                self.foundation_tops[addr.index()].is_some() as usize
            }
            Addr::Depot1
            | Addr::Depot2
            | Addr::Depot3
            | Addr::Depot4
            | Addr::Depot5
            | Addr::Depot6
            | Addr::Depot7 => {
                let pile = &self.depots[addr.index()];
                let mut n = 0;
                for c in pile.iter().rev() {
                    if matches!(c, CardView::FaceUp(..)) {
                        n += 1;
                    } else {
                        break;
                    }
                }
                n
            }
        }
    }

    /// Check what card is at some given address and depth
    ///
    pub fn card_at(&self, addr: &Addr, n: usize) -> Option<CardView> {
        if addr.is_waste() && n == 1 {
            self.waste.last().map(|&x| x.into())
        } else if addr.is_foundation() && n == 1 {
            return self.foundation_tops[addr.index()].map(|v| v.into());
        } else if addr.is_depot() {
            let pile = &self.depots[addr.index()];
            if n <= pile.len() {
                return Some(pile[pile.len() - n]);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    /// Update the view with the result of an action
    /// Assume that the result is valid for the action, e.g. that revealing a card do indeed reveal a card with a suit and a value
    pub fn update(&mut self, action: Action, res: Option<(Suit, Value)>) {
        match action {
            Action::Move(from, to, n) => {
                if from.is_depot() && to.is_depot() {
                    let n_skip = self.depots[from.index()].len().saturating_sub(n);
                    let mut cards_to_move = self.depots[from.index()].split_off(n_skip);
                    self.depots[to.index()].append(&mut cards_to_move);
                } else if from.is_depot() && to.is_foundation() {
                    assert!(n == 1);
                    if let Some(CardView::FaceUp(s, v)) = self.depots[from.index()].pop() {
                        self.foundation_tops[to.index()] = Some((s, v));
                    } else {
                        panic!("We should only move face up cards to the foundation")
                    }
                } else if from.is_foundation() && to.is_depot() {
                    let card = self.foundation_tops[from.index()].unwrap();
                    self.foundation_tops[from.index()].unwrap().1 =
                        Value::try_from(card.1.numeric_value() - 1)
                            .expect("We should never move an ace from foundation");
                    self.depots[to.index()].push(card.into());
                } else if from.is_waste() && to.is_depot() && n == 1 {
                    let card = self.waste.pop().unwrap();
                    self.depots[to.index()].push(card.into());
                } else if from.is_waste() && to.is_foundation() && n == 1 {
                    let card = self.waste.pop().unwrap();
                    self.foundation_tops[to.index()] = Some(card);
                } else {
                    dbg!(action, res);
                    panic!("Illegal move (?)");
                }
            }
            Action::Take => {
                let res = res.expect("We took a card, so there should be some card taken");
                self.waste.push(res);
                self.talon_size -= 1;
            }
            Action::Turnover => {
                self.talon_size = self.waste.len();
                self.waste.clear();
            }
            Action::Quit => {}
            Action::Reveal(addr) => {
                let res = res.expect("We revealed a card, so there should be some card revealed");
                if let Some(a) = self.depots[addr.index()].last_mut() {
                    *a = match a {
                        CardView::FaceDown => CardView::FaceUp(res.0, res.1),
                        _ => panic!("We should only reveal face down cards"),
                    }
                } else {
                    panic!("We should only reveal face down cards");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_move_counts() {
        let view = SolitaireObserver {
            talon_size: 0,
            waste: vec![],
            foundation_tops: [None; 4],
            depots: [
                vec![CardView::FaceUp(Suit::Hearts, Value::KING)],
                vec![CardView::FaceUp(Suit::Clubs, Value::QUEEN)],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
        };
        assert_eq!(view.n_takeable_cards(&Addr::Depot1), 1);
        assert_eq!(view.n_takeable_cards(&Addr::Depot2), 1);
        assert_eq!(view.n_takeable_cards(&Addr::Waste), 0);
    }
}
