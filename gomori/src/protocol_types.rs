use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::Card;

/// Request for a bot to do something.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    /// Request to reset the bot's state for a new game.
    ///
    /// The response should be an [`Okay`].
    NewGame { color: Color },
    /// Request to play the first turn.
    ///
    /// The response should be a single [`Card`], as it is impossible to have a
    /// combo in the first turn. The card will be placed at the coordinates `(0, 0)`.
    PlayFirstTurn {
        /// The hand of the player.
        cards: [Card; 5],
    },
    /// Request to play the next turn.
    ///
    /// The response should be an [`PlayTurnResponse`].
    PlayTurn {
        /// The hand of the player.
        cards: [Card; 5],
        /// The board, represented as a list of the fields that are in use,
        /// i.e. have at least one card on them.
        ///
        /// They are sorted by i first, then j (row-major order, if you think
        /// of i and j as matrix indices).
        fields: Vec<Field>,
        // TODO: opponents action, or some other way of ensuring complete information
    },
    /// The bot should shut down.
    Bye,
}

/// Dummy struct for use in bot communication.
///
/// Used to signal an acknowledgement without data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Okay();

#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    /// The clubs and spades.
    Black,
    /// The diamonds and hearts.
    Red,
}

/// A single field on the board, including coordinates.
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    /// The first coordinate.
    pub i: i8,
    /// The second coordinate.
    pub j: i8,
    /// This may be `None` if the top card has been flipped face-down.
    pub top_card: Option<Card>,
    /// Any cards below the top card, in no particular order.
    ///
    /// A card on the top that has been flipped face-down also counts as "below the top card".
    pub hidden_cards: BTreeSet<Card>,
}

/// Specifies which card to play, and where.
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CardToPlace {
    pub card: Card,
    pub i: i8,
    pub j: i8,
    /// If a king was played on top of another card, this coordinate pair
    /// indicates which card to flip face-down. When not needed, this can be
    /// omitted from the JSON serialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub target_field_for_king_ability: Option<(i8, i8)>,
}

/// The cards to play in this turn, in order.
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayTurnResponse(pub Vec<CardToPlace>);

#[cfg(feature = "python")]
mod python {
    use pyo3::pymethods;

    use super::*;

    #[pymethods]
    impl CardToPlace {
        #[new]
        #[pyo3(signature = (*, card, i, j, target_field_for_king_ability=None))]
        pub(crate) fn py_new(
            card: Card,
            i: i8,
            j: i8,
            target_field_for_king_ability: Option<(i8, i8)>,
        ) -> Self {
            Self {
                card,
                i,
                j,
                target_field_for_king_ability,
            }
        }
    }
}
