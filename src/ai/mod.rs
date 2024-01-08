//! AI module
//!
//! Defines the interface for the AI players and reexports them from their respective submodules.
//!
pub mod greedy;
pub mod simple;

use crate::{
    game::{self, Action},
    view::{self, Addr, Suit, Value},
};
pub use greedy::GreedyAi;
pub use simple::SimpleAi;
use std::hash::Hash;

pub enum AiType {
    Simple,
    Greedy,
}

pub trait Ai {
    fn make_move(&mut self) -> game::Action;
    fn name(&self) -> &'static str;
    fn update(&mut self, action: game::Action, res: Option<(view::Suit, view::Value)>);
}

impl Ai for SimpleAi {
    fn make_move(&mut self) -> game::Action {
        self.calc_action()
    }
    fn name(&self) -> &'static str {
        "SimpleAi"
    }
    fn update(&mut self, action: game::Action, res: Option<(view::Suit, view::Value)>) {
        self.update_view(action, res);
    }
}

impl Ai for GreedyAi {
    fn make_move(&mut self) -> game::Action {
        self.calc_action()
    }
    fn name(&self) -> &'static str {
        "GreedyAi"
    }
    fn update(&mut self, action: game::Action, res: Option<(view::Suit, view::Value)>) {
        self.update_view(action, res);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub enum CardView {
    FaceUp(Suit, Value),
    FaceDown,
}

impl From<(Suit, Value)> for CardView {
    fn from((s, v): (Suit, Value)) -> Self {
        CardView::FaceUp(s, v)
    }
}

/// The observable state of the game, as a struct in itself
/// This is what a human could see when playing the game
/// so it is the same information that we would pass to an AI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolitaireObserver {
    pub talon_size: usize,
    pub waste: Vec<(Suit, Value)>,
    pub foundation_tops: [Option<(Suit,Value)>; 4],
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
                }
                else if from.is_depot() && to.is_foundation() {
                    assert!(n==1);
                    if let Some(CardView::FaceUp(s,v )) = self.depots[from.index()].pop() {
                        self.foundation_tops[to.index()] = Some((s,v));
                    } else{
                        panic!("We should only move face up cards to the foundation")
                    }
                }
                else if from.is_foundation() && to.is_depot() {
                    let card = self.foundation_tops[from.index()].unwrap();
                    self.foundation_tops[from.index()].unwrap().1 = Value::from(card.1.numeric_value()-1);
                    self.depots[to.index()].push(card.into());
                }
                else if from.is_waste() && to.is_depot() && n == 1 {
                    let card = self.waste.pop().unwrap();
                    self.depots[to.index()].push(card.into());
                }
                else if from.is_waste() && to.is_foundation() && n == 1 {
                    let card = self.waste.pop().unwrap();
                    self.foundation_tops[to.index()] = Some(card);
                }
                else {
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
impl Hash for SolitaireObserver {
    /// A hash function that does not care about the order of the cards in the depots or foundations
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.talon_size.hash(state);
        self.waste.hash(state);
        let mut sorted_f = self.foundation_tops.to_vec();
        sorted_f.sort();
        sorted_f.hash(state);
        let mut sorted_d = self.depots.to_vec();
        sorted_d.sort();
        sorted_d.hash(state);
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
