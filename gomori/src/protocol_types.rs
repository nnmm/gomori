use std::collections::BTreeSet;

use pyo3::pyclass;
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

#[pyclass]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    /// The clubs and spades.
    Black,
    /// The diamonds and hearts.
    Red,
}

/// A single field on the board, including coordinates.
#[pyclass]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    /// The first coordinate.
    #[pyo3(get, set)]
    pub i: i8,
    /// The second coordinate.
    #[pyo3(get, set)]
    pub j: i8,
    /// This may be `None` if the top card has been flipped face-down.
    #[pyo3(get, set)]
    pub top_card: Option<Card>,
    /// Any cards below the top card, in no particular order.
    ///
    /// A card on the top that has been flipped face-down also counts as "below the top card".
    #[pyo3(get, set)]
    pub hidden_cards: BTreeSet<Card>,
}

/// Specifies which card to play, and where.
#[pyclass]
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
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayTurnResponse(pub Vec<CardToPlace>);