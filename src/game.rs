//! The game engine/logic.
//! It is mostly private, but creating a new game and sending actions to the game engine is public.

use crate::{core::{Addr,CardView, Suit, Value}, ai::SolitaireObserver};
use itertools::Itertools;
use rand::prelude::*;
use thiserror::Error;

/// The different actions that can be taken in the game
///
/// Implemented as a kind of command pattern, decoupling from the actual methods on the game engine.
/// Designed to be used with the [`GameEngine::act`] method.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Action {
    /// Take the first card of the talon and place it on the waste pile face up
    Take,
    /// Move from one pile to another. If moving between depots, several cards may be moved at once
    Move(
        /// Where do we move from?
        Addr,
        /// Where do we move to?
        Addr,
        /// How many cards?
        usize,
    ),
    /// Turn the waste over to form a new talon
    Turnover,
    /// Reveal a face down cards in some pile
    Reveal(Addr),
    /// Stop playing the game
    Quit,
}

/// A simple flag to know if the game is running, and if not, was it a win or a loss?
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
enum State {
    Running,
    Fail,
    Win,
}

/// The game state. It has methods to observe the state (create a solitaire view) and to act.
///
/// Invariant: the game is always valid, meaning
///  - all 52 cards are in there
///  - the talon have cards face down
///  - face up cards in the columns are alternating colors and decreasing values
///  - the foundations are increasing values of the same suit
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct GameEngine {
    /// the last element = the face up card. pop from last element when picking one.
    talon: Vec<Card>,
    /// the last element = the visible card
    ///
    /// Waste is also sometimes called the "hand" in solitaire, since that is the pile we can play from
    waste: Vec<Card>,
    /// The columns of cards on the table The major part of the tableaux
    ///
    /// last element = the available card.
    columns: [Vec<Card>; 7],
    /// The foundations are where we build the ace piles
    foundations: [Vec<Card>; 4],
    state: State,
    /// The current score
    current_score: u32,
}

/// Errors that can occur when trying to make a move
/// This is bit haphazard, and got extended as needed in my debuggning.
#[derive(Error, Debug)]
pub enum MoveError {
    /// An error with some textual explanation
    #[error("Got explanation {0}")]
    WithDescription(String),
    /// Tried to move a card from a position, but there is no movable cards at that place
    #[error("Found no card to move")]
    NoCardToMove,
    /// The catch-all error type
    #[error("Unspecified move error")]
    Unspecified,
}

impl GameEngine {
    pub fn score(&self) -> u32 {
        self.current_score
    }

    /// Update the score, according to the rules at <https://australiancardgames.com.au/solitaire/>
    fn score_action(&mut self, action: &Action) {
        match action {
            Action::Take => {}
            Action::Move(from, to, _) => {
                if from.is_waste() && to.is_foundation() {
                    self.current_score += 10;
                } else if from.is_waste() && to.is_depot() {
                    self.current_score += 5;
                } else if from.is_depot() && to.is_foundation() {
                    self.current_score += 10;
                } else if from.is_foundation() && to.is_depot() {
                    self.current_score=self.current_score.saturating_sub(15);
                }
            }
            Action::Reveal(_) => {
                self.current_score += 5;
            }
            Action::Turnover => {self.current_score=self.current_score.saturating_sub(100)},
            Action::Quit => {}
        }
    }

    pub fn observe(&self) -> SolitaireObserver {
        SolitaireObserver {
            talon_size: self.talon.len(),
            waste: self.waste.iter().map(|c| (c.suit, c.value)).collect_vec()
            ,
            foundation_tops: [
                self.foundations[0].last().map(|c| c.clone().into()),
                self.foundations[1].last().map(|c| c.clone().into()),
                self.foundations[2].last().map(|c| c.clone().into()),
                self.foundations[3].last().map(|c| c.clone().into()),
            ],
            depots: [
                self.columns[0].iter().map(|c| c.clone().into()).collect(),
                self.columns[1].iter().map(|c| c.clone().into()).collect(),
                self.columns[2].iter().map(|c| c.clone().into()).collect(),
                self.columns[3].iter().map(|c| c.clone().into()).collect(),
                self.columns[4].iter().map(|c| c.clone().into()).collect(),
                self.columns[5].iter().map(|c| c.clone().into()).collect(),
                self.columns[6].iter().map(|c| c.clone().into()).collect(),
            ],
        }
    }

    /// Are we still playing?
    pub fn is_running(&self) -> bool {
        self.state == State::Running
    }

    /// Have we won?
    pub fn is_won(&self) -> bool {
        self.state == State::Win
    }

    /// Deal a new game
    pub fn deal(seed: u64) -> Self {
        /// Inner function that is just a helper to build the depots
        fn build_depot(iter: &mut dyn Iterator<Item = Card>, n: usize) -> Vec<Card> {
            let mut v = vec![];
            for c in iter.take(n - 1) {
                v.push(c);
            }
            v.push(iter.next().expect("Preconditon"));
            v.last_mut().unwrap().reveal();
            v
        }

        let mut pack = shuffled_deck(seed).into_iter();
        let depots = [
            build_depot(&mut pack, 1),
            build_depot(&mut pack, 2),
            build_depot(&mut pack, 3),
            build_depot(&mut pack, 4),
            build_depot(&mut pack, 5),
            build_depot(&mut pack, 6),
            build_depot(&mut pack, 7),
        ];
        let talon: Vec<_> = pack.collect();
        let foundations = [vec![], vec![], vec![], vec![]];
        GameEngine {
            talon,
            waste: vec![],
            columns: depots,
            foundations,
            state: State::Running,
            current_score: 0,
        }
    }

    /// Take the topmost card from the talon and place it on the waste pile
    fn take(&mut self) -> Result<(Suit,Value), MoveError> {
        if let Some(c) = self.talon.pop() {
            self.waste.push(c.clone());
            self.waste.last_mut().unwrap().reveal();
            Ok((c.suit, c.value))
        } else {
            Err(MoveError::Unspecified)
        }
    }

    /// If the talon is empty, we may turn over the waste pile
    fn turnover(&mut self) -> Result<(), MoveError> {
        if self.talon.is_empty() {
            if self.waste.is_empty() {
                Err(MoveError::Unspecified)
            } else {
                self.talon = self
                    .waste
                    .drain(..)
                    .map(|c| Card { faceup: false, ..c })
                    .rev()
                    .collect();
                Ok(())
            }
        } else {
            Err(MoveError::Unspecified)
        }
    }

    /// Reveal the topmost card in a depot, if there is one
    fn reveal(&mut self, addr: &Addr) -> Result<(Suit,Value), MoveError> {
        let depot = match addr {
            Addr::Waste
            | Addr::Foundation1
            | Addr::Foundation2
            | Addr::Foundation3
            | Addr::Foundation4 => Err(MoveError::WithDescription(
                "Cannot reveal cards in this pile".to_string(),
            )),
            Addr::Depot1 => Ok(0),
            Addr::Depot2 => Ok(1),
            Addr::Depot3 => Ok(2),
            Addr::Depot4 => Ok(3),
            Addr::Depot5 => Ok(4),
            Addr::Depot6 => Ok(5),
            Addr::Depot7 => Ok(6),
        }?;
        if let Some(c) = self.columns[depot].last_mut() {
            if c.faceup {
                Err(MoveError::Unspecified)
            } else {
                c.reveal();
                Ok((c.suit, c.value))
            }
        } else {
            Err(MoveError::Unspecified)
        }
    }

    /// Return the pile at the given address
    fn pile_mut(&mut self, addr: &Addr) -> &mut Vec<Card> {
        match addr {
            Addr::Waste => &mut self.waste,
            Addr::Depot1 => &mut self.columns[0],
            Addr::Depot2 => &mut self.columns[1],
            Addr::Depot3 => &mut self.columns[2],
            Addr::Depot4 => &mut self.columns[3],
            Addr::Depot5 => &mut self.columns[4],
            Addr::Depot6 => &mut self.columns[5],
            Addr::Depot7 => &mut self.columns[6],
            Addr::Foundation1 => &mut self.foundations[0],
            Addr::Foundation2 => &mut self.foundations[1],
            Addr::Foundation3 => &mut self.foundations[2],
            Addr::Foundation4 => &mut self.foundations[3],
        }
    }

    /// Return the pile at the given address
    fn pile(&self, addr: &Addr) -> &Vec<Card> {
        match addr {
            Addr::Waste => &self.waste,
            Addr::Depot1 => &self.columns[0],
            Addr::Depot2 => &self.columns[1],
            Addr::Depot3 => &self.columns[2],
            Addr::Depot4 => &self.columns[3],
            Addr::Depot5 => &self.columns[4],
            Addr::Depot6 => &self.columns[5],
            Addr::Depot7 => &self.columns[6],
            Addr::Foundation1 => &self.foundations[0],
            Addr::Foundation2 => &self.foundations[1],
            Addr::Foundation3 => &self.foundations[2],
            Addr::Foundation4 => &self.foundations[3],
        }
    }

    fn move_to_foundation(&mut self, from: &Addr, to: &Addr) -> Result<(), MoveError> {
        let card_to_move = self.pile(from).last().ok_or(MoveError::NoCardToMove)?;

        // Place ace on empty slot
        if card_to_move.numeric_value() == 1 && self.pile(to).is_empty() {
            let card = self.pile_mut(from).pop().unwrap();
            self.pile_mut(to).push(card);
            return Ok(());
        } else if card_to_move.numeric_value() == 1 {
            return Err(MoveError::WithDescription(
                "Cannot place ace on non-empty slot".into(),
            ));
        }

        // Place card on top of same suit and one higher, possibly ending the game
        if let Some(c) = self.pile(to).last() {
            if c.suit == card_to_move.suit && card_to_move.numeric_value() == c.numeric_value() + 1
            {
                let card = self.pile_mut(from).pop().unwrap();
                self.pile_mut(to).push(card);
                if self.foundations.iter().all(|f| f.len() == 13) {
                    self.state = State::Win;
                }
                Ok(())
            } else {
                Err(MoveError::WithDescription(
                    "Cannot place card on top of non-matching suit or non-one-lower value".into(),
                ))
            }
        } else {
            Err(MoveError::WithDescription(
                "Cannot place non-ace on empty slot".into(),
            ))
        }
    }

    fn move_to_depot(&mut self, from: &Addr, to: &Addr, n: usize) -> Result<(), MoveError> {
        // are there enough cards to move?
        if self.pile(from).len() < n {
            return Err(MoveError::Unspecified);
        }

        // all face up?
        let n_skip = self.pile(from).len().saturating_sub(n);
        if self.pile(from).iter().skip(n_skip).any(|c| !c.faceup) {
            return Err(MoveError::Unspecified);
        }

        let base_card = &self.pile(from)[n_skip];

        // move king-starting sequence to empty slot
        if base_card.value.is_king() && self.pile(to).last().is_none() {
            let mut cards_to_move = self.pile_mut(from).split_off(n_skip);
            self.pile_mut(to).append(&mut cards_to_move);
            return Ok(());
        }

        // move red on a black or vice versa, decrease value by one, and destination is face up
        if let Some(c) = self.pile(to).last() {
            if base_card.suit.color() != c.suit.color()
                && base_card.numeric_value() == c.numeric_value() - 1
                && c.faceup
            {
                let mut cards_to_move = self.pile_mut(from).split_off(n_skip);
                self.pile_mut(to).append(&mut cards_to_move);
                return Ok(());
            }
        }

        Err(MoveError::Unspecified)
    }

    fn move_cards(&mut self, from: &Addr, to: &Addr, n: usize) -> Result<(), MoveError> {
        if (from.is_waste() || from.is_foundation()) && n != 1 {
            return Err(MoveError::Unspecified);
        }
        match to {
            Addr::Waste => Err(MoveError::Unspecified),
            Addr::Foundation1 | Addr::Foundation2 | Addr::Foundation3 | Addr::Foundation4 => {
                if n != 1 {
                    return Err(MoveError::Unspecified);
                }
                self.move_to_foundation(from, to)
            }
            Addr::Depot1
            | Addr::Depot2
            | Addr::Depot3
            | Addr::Depot4
            | Addr::Depot5
            | Addr::Depot6
            | Addr::Depot7 => self.move_to_depot(from, to, n),
        }
    }

    pub fn act(&mut self, action: &Action) -> Result<Option<(Suit,Value)>, MoveError> {
        let moveres = match action {
            Action::Take => self.take().map(Some),
            Action::Move(a1, a2, k) => self.move_cards(a1, a2, *k).map(|_| Option::None),
            Action::Reveal(a) => self.reveal(a).map(Some),
            Action::Quit => self.quit().map(|_|Option::None),
            Action::Turnover => self.turnover().map(|_|Option::None),
        };
        if moveres.is_ok() {
            self.score_action(action);
        }
        moveres
    }

    fn quit(&mut self) -> Result<(), MoveError> {
        self.state = State::Fail;
        Ok(())
    }

    pub fn talon_len(&self) -> usize {
        self.talon.len()
    }
}

impl std::fmt::Display for GameEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Talon size {}", self.talon_len())?;
        writeln!(f)?;

        write!(f, "Waste ({} cards)", self.waste.len())?;
        if let Some(c) = self.waste.last() {
            write!(f, " Top card: {c},")?;
        }
        writeln!(f)?;

        // Foundations
        for i in 0..4 {
            if let Some(c) = self.foundations[i].last() {
                write!(f, "{c},")?;
            } else {
                write!(f, "□ ")?
            }
        }
        writeln!(f)?;

        // The tableaux
        writeln!(f)?;
        for i in 0..7 {
            for c in self.columns[i].iter() {
                write!(f, "{c},")?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

/// A card in play. Information about suit, value and whether it is face up/down
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
struct Card {
    suit: Suit,
    value: Value,
    faceup: bool,
}
impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if self.faceup {
            write!(f, "{}{:02}", self.suit, self.value)
        } else {
            write!(f, "▨")
        }
    }
}
impl Card {
    fn reveal(&mut self) {
        self.faceup = true;
    }

    fn numeric_value(&self) -> u8 {
        self.value.numeric_value()
    }
}
impl From<Card> for (Suit, Value) {
    fn from(val: Card) -> Self {
        (val.suit, val.value)
    }
}
impl From<Card> for CardView {
    fn from(val: Card) -> Self {
        match val {
            Card {
                faceup: false,
                ..
            } => CardView::FaceDown,
            Card {
                suit,
                value,
                faceup: true,
            } => CardView::FaceUp(suit, value),
        }
    }
}

/// A deck of cards in random shuffled order. 52 cards of 4 suits and 13 values each.
fn shuffled_deck(seed: u64) -> Vec<Card> {
    let mut d = vec![];
    for c in [Suit::Hearts, Suit::Clubs, Suit::Diamonds, Suit::Spades] {
        for v in 1..=13 {
            d.push(Card {
                suit: c,
                value: Value::try_from(v).expect("Known to be in range"),
                faceup: false,
            })
        }
    }
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
    d.shuffle(&mut rng);
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_only_move_one_from_waste() {
        let mut gs = GameEngine::deal(0);
        let action = Action::Move(Addr::Waste, Addr::Depot3, 2);
        assert!(gs.act(&action).is_err());
    }

    /// When taking some simplified game state and
    /// 1) move card from waste to foundation
    /// 2) reveal a card in the tableaux
    /// 3) move card from tableaux to foundation
    /// make sure the score increase by 10 + 5 + 10 = 25
    #[test]
    fn score_when_moving_cards() {
        let mut gs = GameEngine {
            talon: vec![],
            waste: vec![Card {
                suit: Suit::Hearts,
                value: Value::ACE,
                faceup: true,
            }],
            columns: [
                vec![Card {
                    suit: Suit::Spades,
                    value: Value::TWO,
                    faceup: false,
                }],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            foundations: [vec![], vec![
                Card {
                    suit: Suit::Spades,
                    value: Value::ACE,
                    faceup: true,
                }
            ], vec![], vec![]],
            state: State::Running,
            current_score: 0,
        };
        gs.act(&Action::Move(Addr::Waste, Addr::Foundation1, 1))
            .map_err(|e| eprintln!("{}", e))
            .unwrap();
        gs.act(&Action::Reveal(Addr::Depot1))
            .map_err(|e| eprintln!("{}", e))
            .unwrap();
        gs.act(&Action::Move(Addr::Depot1, Addr::Foundation2, 1))
            .map_err(|e| eprintln!("{}", e))
            .unwrap();
        assert_eq!(gs.score(), 25);
    }

    /// Test there wont be underflow in scoring when turning the deck over
    #[test]
    fn score_when_turning_over() {
        let mut gs = GameEngine {
            talon: vec![],
            waste: vec![Card {
                suit: Suit::Spades,
                value: Value::TWO,
                faceup: true,
            }],
            columns: [
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            foundations: [vec![], vec![], vec![], vec![]],
            state: State::Running,
            current_score: 0,
        };
        gs.act(&Action::Turnover)
            .map_err(|e| eprintln!("{}", e))
            .expect("This should be fin. No underflows. No funny business.");
        assert_eq!(gs.score(), 0);
    }
}
