use std::collections::BTreeSet;

use super::{BoundingBox, IllegalCardPlayed, BOARD_SIZE};
use crate::{Card, CardToPlace, Field, Rank};
/// Represents a board with at least one card on it.
///
/// Because after the first move, there is at least one card on it,
/// the minimum and maximum coordinates always exist.
#[derive(Clone, Debug)]
pub struct SparseBoard {
    pub fields: Vec<Field>,
    /// The smallest i coordinate with a card on it.
    pub i_min: i8,
    /// The largest i coordinate with a card on it.
    pub i_max: i8,
    /// The smallest j coordinate with a card on it.
    pub j_min: i8,
    /// The largest j coordinate with a card on it.
    pub j_max: i8,
}

impl SparseBoard {
    /// Creates a new board from a list of fields.
    ///
    /// Panics if the fields are (obviously) invalid.
    pub fn new(fields: Vec<Field>) -> Self {
        assert!(!fields.is_empty());
        let (mut i_min, mut i_max, mut j_min, mut j_max) =
            (fields[0].i, fields[0].i, fields[0].j, fields[0].j);
        for field in &fields {
            i_min = i_min.min(field.i);
            i_max = i_max.max(field.i);
            j_min = j_min.min(field.j);
            j_max = j_max.max(field.j);
        }
        Self {
            fields,
            i_min,
            i_max,
            j_min,
            j_max,
        }
    }

    pub fn get(&self, i: i8, j: i8) -> Option<&Field> {
        self.fields
            .iter()
            .find(|field| field.i == i && field.j == j)
    }

    pub fn get_mut(&mut self, i: i8, j: i8) -> Option<&mut Field> {
        self.fields
            .iter_mut()
            .find(|field| field.i == i && field.j == j)
    }

    pub fn is_in_bounds(&self, i: i8, j: i8) -> bool {
        (i.checked_sub(self.i_min).unwrap() < BOARD_SIZE)
            && (self.i_max.checked_sub(i).unwrap() < BOARD_SIZE)
            && (j.checked_sub(self.j_min).unwrap() < BOARD_SIZE)
            && (self.j_max.checked_sub(j).unwrap() < BOARD_SIZE)
    }

    fn flip(&mut self, i: i8, j: i8) {
        if let Some(field) = self.get_mut(i, j) {
            if let Some(c) = field.top_card.take() {
                field.hidden_cards.insert(c);
            }
        }
    }

    /// Play a card and return the cards that have been won as a result of this play.
    ///
    /// This function does not validate that the played card has not already been played
    /// and so on. It only checks that the coordinates are valid and the new card is
    /// compatible with any existing card at that coordinate.
    ///
    /// If an error is returned, the board is unmodified.
    pub fn play_card(
        &mut self,
        card_to_place: CardToPlace,
    ) -> Result<PlayCardOutcome, IllegalCardPlayed> {
        let CardToPlace {
            card,
            i,
            j,
            target_field_for_king_ability,
        } = card_to_place;
        if !self.is_in_bounds(i, j) {
            return Err(IllegalCardPlayed::OutOfBounds);
        }

        // If there is already a card (face up) on that spot, it must be compatible.
        // and placing the new card on top causes a combo
        #[rustfmt::skip]
        let combo = match self.get(i, j) {
            Some(Field { top_card: Some(c), .. }) => {
                if !card.can_be_placed_on(*c) {
                    return Err(IllegalCardPlayed::IncompatibleCard { existing_card: *c });
                }
                true
            }
            Some(Field { top_card: None, .. }) => true,
            None => false,
        };

        // Add the card to an existing field, or add a new field with the card in it
        if let Some(field) = self.get_mut(i, j) {
            if let Some(old_card) = field.top_card.replace(card) {
                field.hidden_cards.insert(old_card);
            }
        } else {
            let new_field = Field {
                i,
                j,
                top_card: Some(card),
                hidden_cards: BTreeSet::new(),
            };
            self.fields.push(new_field);
        }
        self.fields.sort_by_key(|field| (field.i, field.j));

        // Activate the face card's abilities
        if combo {
            match card.rank {
                Rank::Jack => {
                    self.flip(i - 1, j);
                    self.flip(i + 1, j);
                    self.flip(i, j - 1);
                    self.flip(i, j + 1);
                }
                Rank::Queen => {
                    self.flip(i - 1, j - 1);
                    self.flip(i - 1, j + 1);
                    self.flip(i + 1, j - 1);
                    self.flip(i + 1, j + 1);
                }
                Rank::King => {
                    let (tgt_i, tgt_j) =
                        if let Some(tgt_coordinates) = target_field_for_king_ability {
                            tgt_coordinates
                        } else {
                            return Err(IllegalCardPlayed::NoTargetForKingAbility);
                        };
                    let f = if let Some(field) = self.get_mut(tgt_i, tgt_j) {
                        field
                    } else {
                        return Err(IllegalCardPlayed::TargetForKingAbilityDoesNotExist {
                            tgt_i,
                            tgt_j,
                        });
                    };
                    if let Some(c) = f.top_card.take() {
                        f.hidden_cards.insert(c);
                    } else {
                        return Err(IllegalCardPlayed::TargetForKingAbilityIsFaceDown {
                            tgt_i,
                            tgt_j,
                        });
                    }
                }
                _ => {}
            }
        }

        // When checking for a line of 4, we know that the last played card must be part of it.
        // Therefore, we can just check how many other cards of the same suit as the played card
        // are in the same row/column/diagonals as the played card.
        let mut matches_with_same_i = 0;
        let mut matches_with_same_j = 0;
        let mut matches_diag_add = 0;
        let mut matches_diag_sub = 0;
        for field in &self.fields {
            // Skip if field has no card or card has the wrong suit
            match field.top_card {
                Some(c) if c.suit == card.suit => {}
                _ => continue,
            }
            if i == field.i {
                matches_with_same_i += 1;
            }
            if j == field.j {
                matches_with_same_j += 1;
            }
            if i.checked_add(j).unwrap() == field.i.checked_add(field.j).unwrap() {
                matches_diag_add += 1;
            }
            if i.checked_sub(j).unwrap() == field.i.checked_sub(field.j).unwrap() {
                matches_diag_sub += 1;
            }
        }

        // Now we know whether there was a connect-4. What remains to be done is to
        // remove the cards/fields that were won and return them.
        let mut won_fields = Vec::new();
        for field in std::mem::take(&mut self.fields).into_iter() {
            // The newly added card and cards underneath it stay on the board
            if field.i == i && field.j == j {
                self.fields.push(field);
                continue;
            }
            if (matches_with_same_i == 4 && field.i == i)
                || (matches_with_same_j == 4 && field.j == j)
                || (matches_diag_add == 4 && i + j == field.i + field.j)  // no need for checked arithmetic here anymore
                || (matches_diag_sub == 4 && i - j == field.i - field.j)
            {
                won_fields.push(field);
            } else {
                self.fields.push(field);
            }
        }

        // Finally, update the min/max bounds
        if won_fields.is_empty() {
            self.i_min = self.i_min.min(i);
            self.j_min = self.j_min.min(j);
            self.i_max = self.i_max.max(i);
            self.j_max = self.j_max.max(j);
        } else {
            // Cards were removed, the bounds have to be recomputed
            self.i_min = i;
            self.j_min = j;
            self.i_max = i;
            self.j_max = j;
            for field in &self.fields {
                self.i_min = self.i_min.min(field.i);
                self.j_min = self.j_min.min(field.j);
                self.i_max = self.i_max.max(field.i);
                self.j_max = self.j_max.max(field.j);
            }
        }

        Ok(PlayCardOutcome { won_fields, combo })
    }

    /// The coordinates where a card may be placed.
    ///
    /// Note that this area can be bigger than BOARD_SIZE x BOARD_SIZE,
    /// e.g. if there's only a single card on the board so far, the area
    /// will be the 7 x 7 area centered on that card.
    pub fn valid_locations(&self) -> BoundingBox {
        BoundingBox {
            i_min: self.i_max.checked_sub(BOARD_SIZE - 1).unwrap(),
            j_min: self.j_max.checked_sub(BOARD_SIZE - 1).unwrap(),
            i_max: self.i_min.checked_add(BOARD_SIZE - 1).unwrap(),
            j_max: self.j_min.checked_add(BOARD_SIZE - 1).unwrap(),
        }
    }

    pub fn num_possible_locations(&self, card: Card) -> usize {
        let mut result = 0;
        let mut num_face_up_cards = 0;
        for top_card in self.fields.iter().filter_map(|f| f.top_card) {
            num_face_up_cards += 1;
            if card.can_be_placed_on(top_card) {
                result += 1;
            }
        }
        let BoundingBox {
            i_max,
            i_min,
            j_max,
            j_min,
        } = self.valid_locations();
        let all_valid_locations = (i_max - i_min + 1) * (j_max - j_min + 1);
        result + usize::try_from(all_valid_locations).unwrap() - num_face_up_cards
    }
}

#[derive(Debug)]
pub struct PlayCardOutcome {
    pub won_fields: Vec<Field>,
    pub combo: bool,
}
