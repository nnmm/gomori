use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A playing card in a standard 52-card game.
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

/// The suit of a [card](Card).
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Suit {
    #[serde(rename = "♦")]
    Diamond,
    #[serde(rename = "♥")]
    Heart,
    #[serde(rename = "♠")]
    Spade,
    #[serde(rename = "♣")]
    Club,
}

/// The rank of a [card](Card).
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Rank {
    #[serde(rename = "2")]
    Two,
    #[serde(rename = "3")]
    Three,
    #[serde(rename = "4")]
    Four,
    #[serde(rename = "5")]
    Five,
    #[serde(rename = "6")]
    Six,
    #[serde(rename = "7")]
    Seven,
    #[serde(rename = "8")]
    Eight,
    #[serde(rename = "9")]
    Nine,
    #[serde(rename = "10")]
    Ten,
    #[serde(rename = "J")]
    Jack,
    #[serde(rename = "Q")]
    Queen,
    #[serde(rename = "K")]
    King,
    #[serde(rename = "A")]
    Ace,
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unicode_char())
    }
}

// !!!!!! NOTE: Keep in sync with pymethods impl block !!!!!!
impl Card {
    pub fn can_be_placed_on(&self, other: Card) -> bool {
        self.rank == other.rank
            || match self.rank {
                Rank::Ace => true,
                Rank::Jack | Rank::Queen | Rank::King => self.suit == other.suit,
                _ => false,
            }
    }

    /// Render this card as a Unicode playing cards character
    pub fn unicode_char(&self) -> char {
        // https://en.wikipedia.org/wiki/Playing_Cards_(Unicode_block)
        let row = match self.suit {
            Suit::Spade => 0,
            Suit::Heart => 1,
            Suit::Diamond => 2,
            Suit::Club => 3,
        };
        let col = match self.rank {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 13,
            Rank::King => 14,
            Rank::Ace => 1,
        };
        let codepoint = 0x1F0A0 + 16 * row + col;
        char::from_u32(codepoint).unwrap()
    }
}

/// The error type for the [`FromStr`] instance of [`Card`].
#[derive(Clone, Copy, Debug)]
pub enum CardFromStrErr {
    LessThanTwoChars,
    MoreThanTwoChars,
    InvalidRank,
    InvalidSuit,
}

impl FromStr for Card {
    type Err = CardFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let rank_char = chars.next().ok_or(CardFromStrErr::LessThanTwoChars)?;
        let suit_char = chars.next().ok_or(CardFromStrErr::LessThanTwoChars)?;
        if chars.next().is_some() {
            return Err(CardFromStrErr::MoreThanTwoChars);
        }
        let rank = match rank_char {
            '2' => Rank::Two,
            '3' => Rank::Three,
            '4' => Rank::Four,
            '5' => Rank::Five,
            '6' => Rank::Six,
            '7' => Rank::Seven,
            '8' => Rank::Eight,
            '9' => Rank::Nine,
            'T' => Rank::Ten,
            'J' => Rank::Jack,
            'Q' => Rank::Queen,
            'K' => Rank::King,
            'A' => Rank::Ace,
            _ => return Err(CardFromStrErr::InvalidRank),
        };
        let suit = match suit_char {
            '♦' => Suit::Diamond,
            '♥' => Suit::Heart,
            '♠' => Suit::Spade,
            '♣' => Suit::Club,
            _ => return Err(CardFromStrErr::InvalidSuit),
        };
        Ok(Card { rank, suit })
    }
}

/// Shorthand for creating cards from a two-character string.
///
/// The first character is the [rank](Rank) (note: 10 is `T`), the second is
/// the [suit](Suit) as a unicode character (♦, ♥, ♠, or ♣).
///
/// This macro is just calling the [`FromStr`] instance of [`Card`].
/// ```
/// # use gomori::{card, Card, Rank, Suit};
/// assert_eq!(
///     card!("T♥"),
///     Card { rank: Rank::Ten, suit: Suit::Heart }
/// );
/// ```
#[macro_export]
macro_rules! card {
    ($rs:literal) => {
        <$crate::Card as std::str::FromStr>::from_str($rs)
            .expect("Invalid card code given to card! macro")
    };
}
// The import is for using the macro in other modules, see https://stackoverflow.com/a/31749071/1726797
#[allow(unused_imports)]
pub(crate) use card;

pub static RED_CARDS: [Card; 26] = [
    Card {
        suit: Suit::Diamond,
        rank: Rank::Two,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Three,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Four,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Five,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Six,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Seven,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Eight,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Nine,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Ten,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Jack,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Queen,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::King,
    },
    Card {
        suit: Suit::Diamond,
        rank: Rank::Ace,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Two,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Three,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Four,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Five,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Six,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Seven,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Eight,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Nine,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Ten,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Jack,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Queen,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::King,
    },
    Card {
        suit: Suit::Heart,
        rank: Rank::Ace,
    },
];

pub static BLACK_CARDS: [Card; 26] = [
    Card {
        suit: Suit::Spade,
        rank: Rank::Two,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Three,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Four,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Five,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Six,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Seven,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Eight,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Nine,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Ten,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Jack,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Queen,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::King,
    },
    Card {
        suit: Suit::Spade,
        rank: Rank::Ace,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Two,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Three,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Four,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Five,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Six,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Seven,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Eight,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Nine,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Ten,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Jack,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Queen,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::King,
    },
    Card {
        suit: Suit::Club,
        rank: Rank::Ace,
    },
];

#[cfg(feature = "python")]
mod python {
    use pyo3::pymethods;

    use super::*;

    #[pymethods]
    impl Card {
        #[new]
        fn py_new(rank: Rank, suit: Suit) -> Self {
            Self { rank, suit }
        }

        // Needed for other __repr__ functions
        pub(crate) fn __repr__(&self) -> String {
            format!("Card({}, {})", self.rank.__repr__(), self.suit.__repr__())
        }

        fn __str__(&self) -> String {
            self.to_string()
        }

        fn py_can_be_placed_on(&self, other: Card) -> bool {
            self.can_be_placed_on(other)
        }
    }

    #[pymethods]
    impl Suit {
        fn __repr__(&self) -> String {
            format!("Suit.{:?}", self)
        }

        fn __str__(&self) -> &'static str {
            match self {
                Suit::Diamond => "♦",
                Suit::Heart => "♥",
                Suit::Spade => "♠",
                Suit::Club => "♣",
            }
        }
    }

    #[pymethods]
    impl Rank {
        fn __repr__(&self) -> String {
            format!("Rank.{:?}", self)
        }

        fn __str__(&self) -> &'static str {
            match self {
                Rank::Two => "2",
                Rank::Three => "3",
                Rank::Four => "4",
                Rank::Five => "5",
                Rank::Six => "6",
                Rank::Seven => "7",
                Rank::Eight => "8",
                Rank::Nine => "9",
                Rank::Ten => "10",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ace => "A",
            }
        }
    }
}
