//! The interface module that specifies how the game is displayed to the AI
//!
//!


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