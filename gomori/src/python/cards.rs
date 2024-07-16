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
}

#[pymethods]
impl Rank {
    pub(in crate::python) fn __repr__(&self) -> String {
        format!("Rank.{:?}", self)
    }
}
