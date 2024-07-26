use crate::{Card, CardsSet, Field};

const TOP_CARD_INDICATOR_BIT: u64 = 0x400000000000000;
const TOP_CARD_MASK: u64 = 0x3f0000000000000;
const HIDDEN_CARDS_MASK: u64 = 0xfffffffffffff;
const CLEAR_TOP_CARD_MASK: u64 = !(TOP_CARD_INDICATOR_BIT | TOP_CARD_MASK);

/// A compact representation of a single field on the board.
///
/// Contains an optional top card (i.e. face up), plus a set of hidden [`Card`]s.
/// It doesn't store the order of hidden cards, or which of the hidden cards are
/// facing up and down, because that doesn't matter for the game.
///
/// # Note on immutability
///
/// This is an immutable type, so its "mutating" methods return a
/// new value instead of really mutating in-place. It is also [`Copy`],
/// so a value is not consumed by methods with `self` receiver.
#[cfg_attr(feature = "python", pyo3::pyclass)]
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
            Some(Card::from_index(card_idx))
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
        let card_idx = card.to_index();
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

    /// All cards on the field.
    ///
    /// Equal to [`hidden_cards()`](Self::hidden_cards) + [`top_card()`](Self::top_card), if any.
    pub fn all_cards(self) -> CardsSet {
        if let Some(c) = self.top_card() {
            self.hidden_cards().insert(c)
        } else {
            self.hidden_cards()
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
            bits |= 1 << card.to_index();
        }
        if let Some(card) = field.top_card {
            let card_idx = card.to_index();
            Self {
                bits: bits | TOP_CARD_INDICATOR_BIT | (u64::from(card_idx) << 52),
            }
        } else {
            Self { bits }
        }
    }
}

#[cfg(feature = "python")]
mod python {
    use pyo3::pymethods;

    use super::*;
    use crate::{Card, Field};

    #[pymethods]
    impl CompactField {
        #[new]
        #[pyo3(signature = (*, top_card, hidden_cards = CardsSet::new()))]
        fn py_new(top_card: Option<Card>, hidden_cards: CardsSet) -> Self {
            let field = Self {
                bits: hidden_cards.bits,
            };
            if let Some(c) = top_card {
                field.place_card(c)
            } else {
                field
            }
        }

        fn __repr__(&self) -> String {
            let top_card_repr = if let Some(c) = self.top_card() {
                c.__repr__()
            } else {
                String::from("None")
            };
            let hidden_cards_repr = self.hidden_cards().__repr__();
            format!(
                "CompactField(top_card={}, hidden_cards={})",
                top_card_repr, hidden_cards_repr
            )
        }

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

        #[pyo3(name = "all_cards")]
        fn py_all_cards(&self) -> CardsSet {
            self.all_cards()
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
