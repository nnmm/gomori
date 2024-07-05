use crate::bitset::bitset_traits;
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CompactField {
    /// The low 52 bits are a bitset of the hidden cards.
    /// The next highest 6 bits are the index of the face-up card, if any.
    /// The next highest bit indicates whether there is a face-up card.
    /// The highest 5 bits are empty.
    bits: u64,
}

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardsSet {
    bits: u64,
}
impl CardsSet {
    /// Creates a new, empty set.
    pub fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn len(self) -> u32 {
        self.bits.count_ones()
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
