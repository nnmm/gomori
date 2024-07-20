#[cfg(feature = "python")]
use pyo3::pyclass;

use crate::helpers::bitset_traits;
use crate::{Card, Field, Rank, Suit};

const TOP_CARD_INDICATOR_BIT: u64 = 0x400000000000000;
const TOP_CARD_MASK: u64 = 0x3f0000000000000;
const HIDDEN_CARDS_MASK: u64 = 0xfffffffffffff;
const CLEAR_TOP_CARD_MASK: u64 = !(TOP_CARD_INDICATOR_BIT | TOP_CARD_MASK);

/// An efficient representation of a single field on the board.
///
/// Contains an optional top card (i.e. face up), plus a set of hidden [`Card`]s.
/// It doesn't store the order of hidden cards, or which of the hidden cards are
/// facing up and down, because that doesn't matter for the game.
///
/// Note that its "mutating" methods return a new object instead of really mutating.
#[cfg_attr(feature = "python", pyclass)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CompactField {
    /// The low 52 bits are a bitset of the hidden cards.
    /// The next highest 6 bits are the index of the face-up card, if any.
    /// The next highest bit indicates whether there is a face-up card.
    /// The highest 5 bits are empty.
    bits: u64,
}

// !!!!!! NOTE: Keep in sync with pymethods impl block !!!!!!
impl CompactField {
    /// Creates an empty field.
    pub fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn is_empty(self) -> bool {
        self.bits == 0
    }

    /// The uppermost card, if it faces up, else `None`.
    pub fn top_card(self) -> Option<Card> {
        if self.bits & TOP_CARD_INDICATOR_BIT == 0 {
            None
        } else {
            let card_idx = ((self.bits & TOP_CARD_MASK) >> 52) as u8;
            Some(card_from_index(card_idx))
        }
    }

    pub fn can_place_card(self, card: Card) -> bool {
        if let Some(c) = self.top_card() {
            card.can_be_placed_on(c)
        } else {
            true
        }
    }

    /// Place a new card on this field, which will be the new face-up card
    #[must_use] // Because users might expect this to be a mutating method
    pub(crate) fn place_card(self, card: Card) -> Self {
        let card_idx = index_from_card(card);
        let Self { bits } = self.turn_face_down();
        Self {
            bits: bits | TOP_CARD_INDICATOR_BIT | (u64::from(card_idx) << 52),
        }
    }

    /// Returns the number of hidden cards on this field.
    pub fn num_hidden_cards(self) -> u32 {
        (self.bits & HIDDEN_CARDS_MASK).count_ones()
    }

    /// Removes the top card and puts it into the hidden card set.
    #[must_use] // Because users might expect this to be a mutating method
    pub fn turn_face_down(self) -> Self {
        if self.bits & TOP_CARD_INDICATOR_BIT == 0 {
            self
        } else {
            let card_idx = (self.bits & TOP_CARD_MASK) >> 52;
            let bits = self.bits & CLEAR_TOP_CARD_MASK | (1u64 << card_idx);
            Self { bits }
        }
    }

    /// Returns a struct that allows iterating over the hidden cards on this field.
    pub fn hidden_cards(self) -> CardsSet {
        CardsSet {
            bits: self.bits & HIDDEN_CARDS_MASK,
        }
    }

    pub fn into_field(self, i: i8, j: i8) -> Field {
        Field {
            i,
            j,
            top_card: self.top_card(),
            hidden_cards: self.hidden_cards().into_iter().collect(),
        }
    }
}

impl Default for CompactField {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&Field> for CompactField {
    fn from(field: &Field) -> Self {
        let mut bits = 0;
        for card in &field.hidden_cards {
            bits |= 1 << index_from_card(*card);
        }
        if let Some(card) = field.top_card {
            let card_idx = index_from_card(card);
            Self {
                bits: bits | TOP_CARD_INDICATOR_BIT | (u64::from(card_idx) << 52),
            }
        } else {
            Self { bits }
        }
    }
}

/// A compact set of [`Card`]s.
///
/// Allows intersection/union/xor with other such sets via bitwise ops.
/// Also implements [`IntoIterator`].
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardsSet {
    bits: u64,
}

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
        let card_idx = index_from_card(card);
        (self.bits & (1u64 << card_idx)) != 0
    }

    pub fn is_empty(self) -> bool {
        self.bits == 0
    }

    #[must_use] // Because users might expect this to be a mutating method
    pub fn insert(self, card: Card) -> Self {
        let card_idx = index_from_card(card);
        Self {
            bits: self.bits | (1u64 << card_idx),
        }
    }
}

bitset_traits!(CardsSet);

impl FromIterator<Card> for CardsSet {
    fn from_iter<T: IntoIterator<Item = Card>>(iter: T) -> Self {
        let mut bits = 0;
        for card in iter {
            bits |= 1u64 << index_from_card(card);
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
#[cfg_attr(feature = "python", pyclass)]
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

            Some(card_from_index(card_idx))
        }
    }
}

// INTERNAL - maps a card onto its "index", a number less than 52
#[inline]
fn index_from_card(card: Card) -> u8 {
    (card.rank as u8) << 2 | card.suit as u8
}

// INTERNAL - mnverse of index_from_cards()
#[inline]
fn card_from_index(bits: u8) -> Card {
    // Fuck it, we transmute
    // SAFETY: This function is internal to this module and only used on
    // bit patterns created by index_from_cards(). In effect, both rank and
    // suit are just cast to their underlying repr and back, which is fine.
    unsafe {
        Card {
            rank: std::mem::transmute::<u8, Rank>(bits >> 2),
            suit: std::mem::transmute::<u8, Suit>(bits & 3),
        }
    }
}

#[cfg(feature = "python")]
mod python {
    use pyo3::pymethods;

    use super::*;
    use crate::{Card, Field};

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

        fn __repr__(&self) -> String {
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

    #[pymethods]
    impl CompactField {
        fn __bool__(&self) -> bool {
            !self.is_empty()
        }

        #[pyo3(name = "is_empty")]
        fn py_is_empty(&self) -> bool {
            self.is_empty()
        }

        #[getter]
        #[pyo3(name = "top_card")]
        fn py_top_card(&self) -> Option<Card> {
            self.top_card()
        }

        #[pyo3(name = "can_place_card")]
        fn py_can_place_card(&self, card: Card) -> bool {
            self.can_place_card(card)
        }

        #[getter]
        #[pyo3(name = "num_hidden_cards")]
        fn py_num_hidden_cards(&self) -> u32 {
            self.num_hidden_cards()
        }

        #[pyo3(name = "turn_face_down")]
        fn py_turn_face_down(&self) -> Self {
            self.turn_face_down()
        }

        #[getter]
        #[pyo3(name = "hidden_cards")]
        fn py_hidden_cards(&self) -> CardsSet {
            self.hidden_cards()
        }

        #[pyo3(name = "into_field")]
        fn py_into_field(&self, i: i8, j: i8) -> Field {
            self.into_field(i, j)
        }
    }

}
#[cfg(test)]
mod test {
    use super::*;
    use crate::{Card, Rank, Suit};

    // Card with the lowest index
    const CARD_1: Card = Card {
        rank: Rank::Two,
        suit: Suit::Diamond,
    };
    // Card with the highest index
    const CARD_2: Card = Card {
        rank: Rank::Ace,
        suit: Suit::Club,
    };
    // Some other card
    const CARD_3: Card = Card {
        rank: Rank::Queen,
        suit: Suit::Heart,
    };

    #[test]
    fn index_is_isomorphic_to_card() {
        assert_eq!(card_from_index(index_from_card(CARD_1)), CARD_1);
        assert_eq!(card_from_index(index_from_card(CARD_2)), CARD_2);
        assert_eq!(card_from_index(index_from_card(CARD_3)), CARD_3);
    }

    #[test]
    fn place_cards() {
        let mut field = CompactField::new();
        field = field.place_card(CARD_1);
        assert_eq!(field.hidden_cards().into_iter().collect::<Vec<_>>(), vec![]);
        assert_eq!(field.top_card(), Some(CARD_1));
        field = field.place_card(CARD_2);
        assert_eq!(
            field.hidden_cards().into_iter().collect::<Vec<_>>(),
            vec![CARD_1]
        );
        assert_eq!(field.top_card(), Some(CARD_2));
        field = field.place_card(CARD_3);
        assert_eq!(
            field.hidden_cards().into_iter().collect::<Vec<_>>(),
            vec![CARD_1, CARD_2]
        );
        assert_eq!(field.top_card(), Some(CARD_3));
    }
}
