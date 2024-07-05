use super::{IllegalCardPlayed, BOARD_SIZE};
use crate::{Card, CardToPlace, CardsSet, CompactField, Field, Rank};

#[derive(Clone, Debug)]
pub struct DenseBoard {
    // This covers not only the currently used fields,
    // but all fields that a card may be placed on, so it can be up to
    // 7 x 7.
    // It's in i-major order, i.e. indexed by [i * size_j][j].
    arr: Vec<CompactField>,
    size_i: usize,
    size_j: usize,
    offset_i: i8,
    offset_j: i8,
}

type IndexSet = u64;

/// The change that playing a card effects on the board.
///
/// Obviously, this struct only makes sense in connection
/// with the [`DenseBoard`] it was created from.
///
/// It's designed to be flat (not using vectors).
#[derive(Copy, Clone, Debug)]
struct Diff {
    // Card to play onto the card_dest field
    card: Card,
    i: i8,
    j: i8,
    // Indices set into arr
    turned_face_down: IndexSet,
    // Indices set into arr
    gathered: IndexSet,
}

#[derive(Copy, Clone, Debug)]
pub struct CalculatedCardPlay<'a> {
    board: &'a DenseBoard,
    /// Changes that would need to be made to the board.
    ///
    /// It's important that this field is private, so it can't be
    /// applied to a different board.
    diff: Diff,
    /// How many/which cards would be won by this move?
    pub cards_won: CardsSet,
    // Is this move a combo?
    pub combo: bool,
}

impl DenseBoard {
    /// Creates a new board from a list of fields.
    ///
    /// Panics if the fields are (obviously) invalid.
    pub fn new(fields: &[Field]) -> Self {
        assert!(!fields.is_empty());
        let (mut i_min, mut i_max, mut j_min, mut j_max) =
            (fields[0].i, fields[0].i, fields[0].j, fields[0].j);
        for field in fields {
            i_min = i_min.min(field.i);
            i_max = i_max.max(field.i);
            j_min = j_min.min(field.j);
            j_max = j_max.max(field.j);
        }
        let mut board = Self::new_aux(i_min, i_max, j_min, j_max);
        for field in fields {
            let idx = board.arr_idx(field.i, field.j).unwrap();
            board.arr[idx] = CompactField::from(field);
        }
        board
    }

    // Internal helper function to create a new, empty board based on the
    // minimum/maximum field coordinates given.
    fn new_aux(i_min: i8, i_max: i8, j_min: i8, j_max: i8) -> Self {
        let min_i_possible = i_max - BOARD_SIZE + 1;
        let min_j_possible = j_max - BOARD_SIZE + 1;
        let max_i_possible = i_min + BOARD_SIZE - 1;
        let max_j_possible = j_min + BOARD_SIZE - 1;

        let size_i = usize::try_from(max_i_possible - min_i_possible + 1).unwrap();
        let size_j = usize::try_from(max_j_possible - min_j_possible + 1).unwrap();
        assert!(size_i < 8);
        assert!(size_j < 8);

        Self {
            arr: vec![CompactField::new(); size_i * size_j],
            size_i,
            size_j,
            offset_i: min_i_possible,
            offset_j: min_j_possible,
        }
    }

    pub fn get(&self, i: i8, j: i8) -> Option<CompactField> {
        let idx = self.arr_idx(i, j)?;
        Some(self.arr[idx])
    }

    /// Simulate playing a card and return the effects that this would have.
    ///
    /// This function does not validate that the played card has not already been played
    /// and so on. It only checks that the coordinates are valid and the new card is
    /// compatible with any existing card at that coordinate.
    pub fn calculate(
        &self,
        card_to_place: CardToPlace,
    ) -> Result<CalculatedCardPlay<'_>, IllegalCardPlayed> {
        let CardToPlace {
            i,
            j,
            card,
            target_field_for_king_ability,
        } = card_to_place;
        let (i_local, j_local) = self
            .local_coords(i, j)
            .ok_or(IllegalCardPlayed::OutOfBounds)?;
        let card_dest = self.size_j * i_local + j_local;

        // If there is already a card on that spot, it must be compatible.
        // and placing the new card on top causes a combo
        if let Some(c) = self.arr[card_dest].top_card() {
            if !card.can_be_placed_on(c) {
                return Err(IllegalCardPlayed::IncompatibleCard { existing_card: c });
            }
        }
        let combo = !self.arr[card_dest].is_empty();

        // Activate the face card's abilities
        let turned_face_down = if combo {
            self.fields_to_flip(i, j, card, target_field_for_king_ability)?
        } else {
            0
        };

        // Next, detect lines.
        // First, build an index set of all cards of the same suit
        let mut cards_with_same_suit: IndexSet = 1u64 << card_dest;
        for (idx, field) in self.arr.iter().enumerate() {
            if let Some(c) = field.top_card() {
                if c.suit == card.suit {
                    cards_with_same_suit |= 1u64 << idx;
                }
            }
        }
        // Don't count the cards that were flipped
        cards_with_same_suit &= !turned_face_down;

        let gathered = {
            let cards_with_line = self.detect_line(i_local, j_local, cards_with_same_suit);
            // The new card itself stays on the board
            cards_with_line & !(1u64 << card_dest)
        };

        let cards_won = {
            let mut iter = gathered;
            let mut set = CardsSet::new();
            while iter != 0 {
                let idx: u32 = iter.trailing_zeros();
                set |= self.arr[idx as usize].hidden_cards();
                if let Some(card) = self.arr[idx as usize].top_card() {
                    set = set.insert(card);
                }
                iter ^= 1u64 << idx;
            }
            set
        };

        Ok(CalculatedCardPlay {
            board: self,
            diff: Diff {
                card,
                i,
                j,
                turned_face_down,
                gathered,
            },
            cards_won,
            combo,
        })
    }

    pub fn possible_locations_for_card(&self, card: Card) -> impl Iterator<Item = (i8, i8)> + '_ {
        self.arr.iter().enumerate().filter_map(move |(idx, field)| {
            // Casting is fine, the values are never larger than 7
            let i = (idx / self.size_j) as i8 + self.offset_i;
            let j = (idx % self.size_j) as i8 + self.offset_j;
            if let Some(c) = field.top_card() {
                if card.can_be_placed_on(c) {
                    Some((i, j))
                } else {
                    None
                }
            } else {
                Some((i, j))
            }
        })
    }

    pub fn fields(&self) -> impl Iterator<Item = (i8, i8, CompactField)> + '_ {
        self.arr.iter().enumerate().map(|(idx, field)| {
            // Casting is fine, the values are never larger than 7
            let i = (idx / self.size_j) as i8 + self.offset_i;
            let j = (idx % self.size_j) as i8 + self.offset_j;
            (i, j, *field)
        })
    }

    pub fn to_fields_vec(&self) -> Vec<Field> {
        self.fields()
            .filter_map(|(i, j, cf)| {
                if cf.is_empty() {
                    None
                } else {
                    Some(Field {
                        i,
                        j,
                        top_card: cf.top_card(),
                        hidden_cards: cf.hidden_cards().into_iter().collect(),
                    })
                }
            })
            .collect()
    }

    // Check that the 2D coordinates are valid and convert them into local coordinates
    fn local_coords(&self, i: i8, j: i8) -> Option<(usize, usize)> {
        let i_local = usize::try_from(i - self.offset_i).ok()?;
        let j_local = usize::try_from(j - self.offset_j).ok()?;
        if i_local < self.size_i && j_local < self.size_j {
            Some((i_local, j_local))
        } else {
            None
        }
    }

    // Convert the 2D index into a "flat" array index
    fn arr_idx(&self, i: i8, j: i8) -> Option<usize> {
        let (local_i, local_j) = self.local_coords(i, j)?;
        Some(self.size_j * local_i + local_j)
    }

    // Internal helper function to compute fields where the top cards are flipped face-down.
    //
    // Note: The resulting index set also contains fields without cards to flip.
    fn fields_to_flip(
        &self,
        i: i8,
        j: i8,
        card: Card,
        target_field_for_king_ability: Option<(i8, i8)>,
    ) -> Result<IndexSet, IllegalCardPlayed> {
        let mut index_set = 0;

        match card.rank {
            Rank::Jack => {
                for (i, j) in [(i - 1, j), (i + 1, j), (i, j - 1), (i, j + 1)] {
                    if let Some(idx) = self.arr_idx(i, j) {
                        index_set |= 1u64 << idx;
                    }
                }
            }
            Rank::Queen => {
                for (i, j) in [
                    (i - 1, j - 1),
                    (i - 1, j + 1),
                    (i + 1, j - 1),
                    (i + 1, j + 1),
                ] {
                    if let Some(idx) = self.arr_idx(i, j) {
                        index_set |= 1u64 << idx;
                    }
                }
            }
            Rank::King => {
                let (tgt_i, tgt_j) = target_field_for_king_ability
                    .ok_or(IllegalCardPlayed::NoTargetForKingAbility)?;
                let idx = self
                    .arr_idx(tgt_i, tgt_j)
                    .ok_or(IllegalCardPlayed::TargetForKingAbilityDoesNotExist { tgt_i, tgt_j })?;
                let field = self.arr[idx];
                if field.is_empty() {
                    return Err(IllegalCardPlayed::TargetForKingAbilityDoesNotExist {
                        tgt_i,
                        tgt_j,
                    });
                }
                if field.top_card().is_none() && (i, j) != (tgt_i, tgt_j) {
                    return Err(IllegalCardPlayed::TargetForKingAbilityIsFaceDown { tgt_i, tgt_j });
                }
                index_set |= 1u64 << idx;
            }
            _ => (), // If no combo or no face card, nothing is flipped
        }
        Ok(index_set)
    }

    // Internal helper function to check if a newly played card at (i, j) results in a line.
    //
    // The function makes use of the fact that such a line can only go through (i, j)
    // and have the same suit as the new card.
    fn detect_line(
        &self,
        i_local: usize,
        j_local: usize,
        cards_with_same_suit: IndexSet,
    ) -> IndexSet {
        // Create index sets corresponding to horizontal/vertical/diagonal lines going through (i, j)
        // Note, these often have more than 4 entries, but that's intentional. Afterwards, these index sets
        // are intersected with cards_with_the_same_suit, so it only counts fields with actual cards on them
        // that have the correct suit.
        let constant_i_indices: IndexSet = {
            let mut set = 0;
            for j in 0..self.size_j {
                let idx = i_local * self.size_j + j;
                set |= 1u64 << idx;
            }
            set
        };

        let constant_j_indices: IndexSet = {
            let mut set = 0;
            for i in 0..self.size_i {
                let idx = i * self.size_j + j_local;
                set |= 1u64 << idx;
            }
            set
        };

        let diag: IndexSet = {
            let mut set = 0;
            let (mut i, mut j) = if i_local >= j_local {
                (i_local - j_local, 0)
            } else {
                (0, j_local - i_local)
            };
            while i < self.size_i && j < self.size_j {
                let idx = i * self.size_j + j;
                set |= 1u64 << idx;
                i += 1;
                j += 1;
            }
            set
        };

        let antidiag: IndexSet = {
            let j_max = self.size_j - 1;
            // anti_j is j counted from the opposite side, i.e. j_max - j
            let (mut i, mut anti_j) = if i_local + j_local >= j_max {
                (i_local + j_local - j_max, 0)
            } else {
                (0, j_max - j_local - i_local)
            };
            let mut set = 0;

            while i < self.size_i && anti_j < self.size_j {
                let idx = i * self.size_j + (j_max - anti_j);
                set |= 1u64 << idx;
                i += 1;
                anti_j += 1;
            }
            set
        };

        let mut index_set = 0;

        for pattern in [constant_i_indices, constant_j_indices, diag, antidiag] {
            let pattern_intersect = pattern & cards_with_same_suit;
            debug_assert!(pattern_intersect.count_ones() <= 4);

            if pattern_intersect.count_ones() == 4 {
                index_set |= pattern_intersect;
            }
        }

        index_set
    }
}

impl<'a> CalculatedCardPlay<'a> {
    pub fn execute(self) -> DenseBoard {
        // Create new empty board with appropriate size
        let (mut i_min, mut j_min, mut i_max, mut j_max) =
            (self.diff.i, self.diff.j, self.diff.i, self.diff.j);
        for i_local in 0..self.board.size_i {
            for j_local in 0..self.board.size_j {
                let idx = i_local * self.board.size_j + j_local;
                if !(self.board.arr[idx].is_empty() || (self.diff.gathered & (1u64 << idx)) != 0) {
                    i_min = i_min.min(self.board.offset_i + i_local as i8);
                    j_min = j_min.min(self.board.offset_j + j_local as i8);
                    i_max = i_max.max(self.board.offset_i + i_local as i8);
                    j_max = j_max.max(self.board.offset_j + j_local as i8);
                }
            }
        }
        let mut new_board = DenseBoard::new_aux(i_min, i_max, j_min, j_max);

        // Copy over the fields. This is not that easy since the board can change
        // shape every move.
        for i_local in 0..self.board.size_i {
            let i = i_local as i8 + self.board.offset_i;
            // The old coordinate may not be representable in the new board,
            // in which case it's empty and can be skipped
            let i_local_new_board = match usize::try_from(i - new_board.offset_i) {
                Ok(i_new_board) if i_new_board < new_board.size_i => i_new_board,
                _ => continue,
            };
            for j_local in 0..self.board.size_j {
                let j = j_local as i8 + self.board.offset_j;
                let j_local_new_board = match usize::try_from(j - new_board.offset_j) {
                    Ok(j_new_board) if j_new_board < new_board.size_j => j_new_board,
                    _ => continue,
                };

                let idx = i_local * self.board.size_j + j_local;
                let idx_new_board = i_local_new_board * new_board.size_j + j_local_new_board;
                if self.diff.gathered & (1u64 << idx) != 0 {
                    continue;
                }
                new_board.arr[idx_new_board] = if self.diff.turned_face_down & (1u64 << idx) != 0 {
                    self.board.arr[idx].turn_face_down()
                } else {
                    self.board.arr[idx]
                };
            }
        }
        // Add the new card
        let card_dest_idx = new_board.arr_idx(self.diff.i, self.diff.j).unwrap();
        let dest_field = &mut new_board.arr[card_dest_idx];
        *dest_field = dest_field.place_card(self.diff.card);
        if (self.diff.turned_face_down
            & (1u64 << self.board.arr_idx(self.diff.i, self.diff.j).unwrap()))
            != 0
        {
            *dest_field = dest_field.turn_face_down();
        }

        // Done
        new_board
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use quickcheck::quickcheck;

    use super::*;
    use crate::arbitrary::PlayCardInput;
    use crate::{card, CardToPlace, Field, IllegalCardPlayed, SparseBoard};
    quickcheck! {
        fn fields_vec_roundtrip(input: PlayCardInput) -> bool {
            DenseBoard::new(&input.fields).to_fields_vec() == input.fields
        }
    }

    quickcheck! {
        fn possible_cards_are_indeed_possible(input: PlayCardInput) -> bool {
            let board = DenseBoard::new(&input.fields);
            let mut good = true;
            for (i, j) in board.possible_locations_for_card(input.card_to_place.card) {
                good &= match board.calculate(CardToPlace { i, j, card: input.card_to_place.card, target_field_for_king_ability: None}) {
                    Ok(_) => true,
                    Err(IllegalCardPlayed::NoTargetForKingAbility) => true,
                    Err(_) => false,
                };
            }
            good
        }
    }

    #[test]
    fn play_card_horizontal() {
        let board = DenseBoard::new(&[
            Field {
                i: -1,
                j: 0,
                top_card: Some(card!("4♦")),
                hidden_cards: BTreeSet::new(),
            },
            Field {
                i: -1,
                j: -1,
                top_card: Some(card!("5♦")),
                hidden_cards: BTreeSet::new(),
            },
            Field {
                i: -1,
                j: -2,
                top_card: Some(card!("6♦")),
                hidden_cards: BTreeSet::new(),
            },
            Field {
                i: -1,
                j: -3,
                top_card: Some(card!("A♠")),
                hidden_cards: BTreeSet::new(),
            },
        ]);
        let card = card!("A♦");
        let plan = board
            .calculate(CardToPlace {
                i: -1,
                j: -3,
                card,
                target_field_for_king_ability: None,
            })
            .unwrap();
        assert_eq!(plan.diff.turned_face_down, 0);
        assert_eq!(
            plan.cards_won,
            CardsSet::from_iter([card!("4♦"), card!("5♦"), card!("6♦")])
        );
    }

    #[test]
    fn play_card_antidiag() {
        let board = DenseBoard::new(&[
            Field {
                i: -1,
                j: 0,
                top_card: Some(card!("4♦")),
                hidden_cards: BTreeSet::new(),
            },
            Field {
                i: 0,
                j: -1,
                top_card: Some(card!("5♦")),
                hidden_cards: BTreeSet::new(),
            },
            Field {
                i: 1,
                j: -2,
                top_card: Some(card!("6♦")),
                hidden_cards: BTreeSet::new(),
            },
            Field {
                i: 2,
                j: -3,
                top_card: Some(card!("A♠")),
                hidden_cards: BTreeSet::new(),
            },
        ]);
        let card = card!("A♦");
        let plan = board
            .calculate(CardToPlace {
                i: 2,
                j: -3,
                card,
                target_field_for_king_ability: None,
            })
            .unwrap();
        assert_eq!(plan.diff.turned_face_down, 0);
        assert!(plan.diff.gathered != 0);
    }

    #[test]
    fn repro() {
        let fields = [
            Field {
                i: -2,
                j: 0,
                top_card: Some(card!("J♦")),
                hidden_cards: BTreeSet::new(),
            },
            Field {
                i: -2,
                j: 3,
                top_card: Some(card!("T♣")),
                hidden_cards: BTreeSet::from([card!("3♠"), card!("T♠"), card!("K♠"), card!("K♣")]),
            },
            Field {
                i: -1,
                j: 1,
                top_card: Some(card!("4♣")),
                hidden_cards: BTreeSet::from([card!("8♦"), card!("8♠"), card!("8♣")]),
            },
            Field {
                i: 0,
                j: 0,
                top_card: Some(card!("Q♠")),
                hidden_cards: BTreeSet::from([card!("8♥"), card!("7♠"), card!("7♣")]),
            },
            Field {
                i: 0,
                j: 2,
                top_card: Some(card!("Q♣")),
                hidden_cards: BTreeSet::from([card!("Q♦")]),
            },
            Field {
                i: 1,
                j: 3,
                top_card: Some(card!("9♣")),
                hidden_cards: BTreeSet::from([card!("4♠"), card!("A♣")]),
            },
        ];
        let mut board = DenseBoard::new(&fields);
        let ctp = CardToPlace {
            card: card!("J♣"),
            i: -2,
            j: 0,
            target_field_for_king_ability: None,
        };
        let calc = dbg!(board.calculate(ctp).unwrap());
        board = calc.execute();

        assert_eq!(board.get(-2, 0).unwrap().top_card(), Some(card!("J♣")));
    }
}
