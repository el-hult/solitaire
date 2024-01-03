//! The interface module that specifies how the game is displayed to the AI
//!
//!
use std::hash::Hash;


#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}
impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Suit::Hearts => write!(f, "H"),
            Suit::Clubs => write!(f, "C"),
            Suit::Diamonds => write!(f, "D"),
            Suit::Spades => write!(f, "S"),
        }
    }
}
impl Suit {
    pub fn color(&self) -> Color {
        match self {
            Suit::Hearts => Color::Red,
            Suit::Diamonds => Color::Red,
            Suit::Clubs => Color::Black,
            Suit::Spades => Color::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub enum CardView {
    FaceUp(Suit, Value),
    FaceDown,
}

/// The observable state of the game, as a struct in itself
/// This is what a human would see when playing the game
/// so it is the same information that we would pass to an AI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolitaireView {
    pub talon_size: usize,
    pub waste_top: Option<(Suit, Value)>,
    pub foundation_tops: [Option<CardView>; 4],
    pub depots: [Vec<CardView>; 7],
}

impl SolitaireView {
    pub fn is_won(&self) -> bool {
        self.foundation_tops
            .iter()
            .all(|f| matches!(f, Some(CardView::FaceUp(_, Value::KING))))
    }

    /// For some address, how many face card can we pick?
    pub fn n_takeable_cards(&self, addr: &Addr) -> usize {
        match addr {
            Addr::Waste => self.waste_top.is_some() as usize,
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
            self.waste_top.map(|(s, v)| CardView::FaceUp(s, v))
        } else if addr.is_foundation() && n == 1 {
            return self.foundation_tops[addr.index()];
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
}
impl Hash for SolitaireView {

    /// A hash function that does not care about the order of the cards in the depots or foundations
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.talon_size.hash(state);
        self.waste_top.hash(state);
        let mut sorted_f = self.foundation_tops.to_vec();
        sorted_f.sort();
        sorted_f.hash(state);
        let mut sorted_d = self.depots.to_vec();
        sorted_d.sort();
        sorted_d.hash(state);
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum Addr {
    Waste,
    Foundation1,
    Foundation2,
    Foundation3,
    Foundation4,
    Depot1,
    Depot2,
    Depot3,
    Depot4,
    Depot5,
    Depot6,
    Depot7,
}

impl Addr {
    pub fn is_depot(&self) -> bool {
        match self {
            Addr::Depot1
            | Addr::Depot2
            | Addr::Depot3
            | Addr::Depot4
            | Addr::Depot5
            | Addr::Depot6
            | Addr::Depot7 => true,
            Addr::Foundation1
            | Addr::Foundation2
            | Addr::Foundation3
            | Addr::Foundation4
            | Addr::Waste => false,
        }
    }

    pub fn is_foundation(&self) -> bool {
        match self {
            Addr::Foundation1 | Addr::Foundation2 | Addr::Foundation3 | Addr::Foundation4 => true,
            Addr::Depot1
            | Addr::Depot2
            | Addr::Depot3
            | Addr::Depot4
            | Addr::Depot5
            | Addr::Depot6
            | Addr::Depot7
            | Addr::Waste => false,
        }
    }
    pub fn is_waste(&self) -> bool {
        match self {
            Addr::Waste => true,
            Addr::Foundation1
            | Addr::Foundation2
            | Addr::Foundation3
            | Addr::Foundation4
            | Addr::Depot1
            | Addr::Depot2
            | Addr::Depot3
            | Addr::Depot4
            | Addr::Depot5
            | Addr::Depot6
            | Addr::Depot7 => false,
        }
    }

    /// Return the index of the address into its own array
    pub fn index(&self) -> usize {
        match self {
            Addr::Waste => 1,
            Addr::Foundation1 => 0,
            Addr::Foundation2 => 1,
            Addr::Foundation3 => 2,
            Addr::Foundation4 => 3,
            Addr::Depot1 => 0,
            Addr::Depot2 => 1,
            Addr::Depot3 => 2,
            Addr::Depot4 => 3,
            Addr::Depot5 => 4,
            Addr::Depot6 => 5,
            Addr::Depot7 => 6,
        }
    }

    pub const FOUNDATIONS: [Addr; 4] = [
        Addr::Foundation1,
        Addr::Foundation2,
        Addr::Foundation3,
        Addr::Foundation4,
    ];
    pub const DEPOTS: [Addr; 7] = [
        Addr::Depot1,
        Addr::Depot2,
        Addr::Depot3,
        Addr::Depot4,
        Addr::Depot5,
        Addr::Depot6,
        Addr::Depot7,
    ];
}

pub const DEPOTS_AND_WASTE: [Addr; 8] = [
    Addr::Depot1,
    Addr::Depot2,
    Addr::Depot3,
    Addr::Depot4,
    Addr::Depot5,
    Addr::Depot6,
    Addr::Depot7,
    Addr::Waste,
];

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Value(u8);
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:02}", self.0)
    }
}
impl Value {
    pub fn is_king(&self) -> bool {
        self.0 == 13
    }

    pub fn numeric_value(&self) -> u8 {
        self.0
    }

    pub fn is_ace(&self) -> bool {
        self.0 == 1
    }

    pub const TWO: Value = Value(2);
    #[cfg(test)]
    pub const QUEEN: Value = Value(12);
    pub const KING: Value = Value(13);
}
impl std::convert::From<u8> for Value {
    fn from(v: u8) -> Self {
        Value(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_move_counts() {
        let view = SolitaireView {
            talon_size: 0,
            waste_top: None,
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
