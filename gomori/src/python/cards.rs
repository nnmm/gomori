use pyo3::pymethods;

use crate::{Card, Rank, Suit};

#[pymethods]
impl Card {
    #[new]
    pub(in crate::python) fn py_new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    pub(in crate::python) fn __repr__(&self) -> String {
        format!("Card({}, {})", self.rank.__repr__(), self.suit.__repr__())
    }

    pub(in crate::python) fn __str__(&self) -> String {
        self.to_string()
    }

    pub(in crate::python) fn py_can_be_placed_on(&self, other: Card) -> bool {
        self.can_be_placed_on(other)
    }
}

#[pymethods]
impl Suit {
    pub(in crate::python) fn __repr__(&self) -> String {
        format!("Suit.{:?}", self)
    }

    pub(in crate::python) fn __str__(&self) -> &'static str {
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
    pub(in crate::python) fn __repr__(&self) -> String {
        format!("Rank.{:?}", self)
    }

    pub(in crate::python) fn __str__(&self) -> &'static str {
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
