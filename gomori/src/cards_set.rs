use std::iter::FusedIterator;

use crate::Card;

/// A compact set of [`Card`]s.
///
/// Allows intersection/union/xor with other such sets via bitwise ops.
/// Also implements [`IntoIterator`], so it can be converted into e.g.
/// a vector with `Vec::from_iter(cards_set)`.
///
/// ```
/// use gomori::{card, CardsSet};
/// let mut set = CardsSet::new();
/// // This is an immutable data type, so functions like `insert` return a new `CardsSet`.
/// set = set.insert(card!("7♥"));
/// set = set.insert(card!("7♥"));  // Inserting a second time has no effect
/// set = set.insert(card!("2♥"));
/// assert_eq!(Vec::from_iter(set), vec![card!("2♥"), card!("7♥")]);
/// ```
///
/// # Note on immutability
///
/// This is an immutable type, so its "mutating" methods return a
/// new value instead of really mutating in-place (except for `std::ops::BitXxxAssign` trait methods).
/// It is also [`Copy`], so a value is not consumed by methods with `self` receiver.
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardsSet {
    // Only the low 52 bits are used.
    pub(crate) bits: u64,
}

const VALID_BITS: u64 = 0b1111111111111111111111111111111111111111111111111111u64;

/// Equal to `CardsSet::from_iter(RED_CARDS)`.
pub const RED_CARDS_SET: CardsSet = CardsSet {
    bits: 0x3333333333333,
};

/// Equal to `CardsSet::from_iter(BLACK_CARDS)`.
pub const BLACK_CARDS_SET: CardsSet = CardsSet {
    bits: 0xccccccccccccc,
};

// !!!!!! NOTE: Keep in sync with pymethods impl block !!!!!!
impl CardsSet {
    /// Creates a new, empty set.
    pub fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn len(self) -> u32 {
        self.bits.count_ones()
    }

    pub fn contains(self, card: Card) -> bool {
        (self.bits & (1u64 << card.to_index())) != 0
    }

    pub fn is_empty(self) -> bool {
        self.bits == 0
    }

    #[must_use] // Because users might expect this to be a mutating method
    pub fn insert(self, card: Card) -> Self {
        Self {
            bits: self.bits | (1u64 << card.to_index()),
        }
    }

    #[must_use] // Because users might expect this to be a mutating method
    pub fn remove(self, card: Card) -> Self {
        Self {
            bits: self.bits & !(1u64 << card.to_index()),
        }
    }
}

impl std::ops::BitAnd for CardsSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits & rhs.bits,
        }
    }
}

impl std::ops::BitOr for CardsSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl std::ops::BitXor for CardsSet {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits ^ rhs.bits,
        }
    }
}

impl std::ops::BitAndAssign for CardsSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits &= rhs.bits;
    }
}

impl std::ops::BitOrAssign for CardsSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl std::ops::BitXorAssign for CardsSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bits ^= rhs.bits;
    }
}

impl std::ops::Not for CardsSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            bits: !self.bits & VALID_BITS,
        }
    }
}

impl Default for CardsSet {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

impl FromIterator<Card> for CardsSet {
    fn from_iter<T: IntoIterator<Item = Card>>(iter: T) -> Self {
        let mut bits = 0;
        for card in iter {
            bits |= 1u64 << card.to_index();
        }
        Self { bits }
    }
}

impl IntoIterator for CardsSet {
    type Item = Card;

    type IntoIter = CardsSetIter;

    fn into_iter(self) -> Self::IntoIter {
        CardsSetIter { bits: self.bits }
    }
}

/// Iterator for a [`CardsSet`] that returns cards by ascending rank.
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Clone, Copy, Debug)]
pub struct CardsSetIter {
    bits: u64,
}

impl Iterator for CardsSetIter {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            // The number of trailing bits is the card_idx
            let card_idx: u8 = self.bits.trailing_zeros().try_into().unwrap();
            // Clear the flag corresponding to this card index
            self.bits ^= 1u64 << card_idx;

            Some(Card::from_index(card_idx))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.bits.count_ones() as usize;
        (size, Some(size))
    }
}

impl ExactSizeIterator for CardsSetIter {
    fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
}

impl FusedIterator for CardsSetIter {}

#[cfg(feature = "python")]
mod python {
    use pyo3::pymethods;

    use super::*;
    use crate::Card;

    #[pymethods]
    impl CardsSet {
        #[new]
        #[pyo3(signature = (cards=vec![]))]
        fn py_new(cards: Vec<Card>) -> Self {
            Self::from_iter(cards)
        }

        fn __bool__(&self) -> bool {
            !self.is_empty()
        }

        fn __len__(&self) -> usize {
            self.len() as usize
        }

        fn __contains__(&self, card: Card) -> bool {
            self.contains(card)
        }

        fn __iter__(&self) -> CardsSetIter {
            self.into_iter()
        }

        // Needed in other __repr__ functions
        pub(crate) fn __repr__(&self) -> String {
            let card_reprs: Vec<_> = self.into_iter().map(|c| c.__repr__()).collect();
            format!("CardsSet([{}])", card_reprs.join(", "))
        }

        fn __and__(&self, other: CardsSet) -> CardsSet {
            *self & other
        }

        fn __or__(&self, other: CardsSet) -> CardsSet {
            *self | other
        }

        fn __xor__(&self, other: CardsSet) -> CardsSet {
            *self ^ other
        }

        fn __invert__(&self) -> CardsSet {
            !*self
        }

        fn __iand__(&mut self, other: CardsSet) {
            *self &= other
        }

        fn __ior__(&mut self, other: CardsSet) {
            *self |= other
        }

        fn __ixor__(&mut self, other: CardsSet) {
            *self ^= other
        }

        #[getter]
        #[pyo3(name = "is_empty")]
        fn py_is_empty(&self) -> bool {
            self.is_empty()
        }

        #[pyo3(name = "insert")]
        fn py_insert(&mut self, card: Card) {
            *self = self.insert(card);
        }

        #[pyo3(name = "remove")]
        fn py_remove(&mut self, card: Card) {
            *self = self.remove(card);
        }
    }

    #[pymethods]
    impl CardsSetIter {
        fn __iter__(&self) -> Self {
            *self
        }

        fn __next__(&mut self) -> Option<Card> {
            self.next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BLACK_CARDS, RED_CARDS};

    #[test]
    fn set_constants() {
        assert_eq!(CardsSet::from_iter(RED_CARDS), RED_CARDS_SET);
        assert_eq!(CardsSet::from_iter(BLACK_CARDS), BLACK_CARDS_SET);
    }
}
