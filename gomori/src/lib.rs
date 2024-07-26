//! # Overview
//! A bot will receive a [`Board`] and a set of five [`Card`]s, and outputs a list of [`CardToPlay`].
//! Therefore, to get familiar with the API, it is recommended to start looking at these types.
//!
//! # Coordinates
//! Gomori uses 2D coordinates, usually referred to as `i` and `j`, to identify locations on the board.
//! `i` is always the first coordinate in a coordinate pair, and `j` the second.
//!
//! #### Interpretation
//!
//! This library doesn't define which of `i` and `j` is the horizontal coordinate and which is the vertical,
//! or if larger coordinates mean "left" or "right", "down" or "up".
//! The game rules can be implemented without assigning an interpretation to these coordinates, and so it is
//! up to the user to choose how to interpret and visualize these coordinates.
//! For instance, one could treat them as matrix coordinates, i.e. `i` being the row index
//! (top to bottom) and `j` the column index (left to right).
//!
//! #### Absolute vs relative
//!
//! The game rules allow the board location to drift over the course of a game.
//! For example, imagine that players play new cards (darker shading) to the right of the board,
//! and winning (taking away) cards on the left like so:
//! ```text
//! ▒ ▒ ▒ ▓          ▒          ▒          ▒
//!   ▒ ▒ ▒  =>  ▒ ▒ ▒  =>  ▒ ▒ ▒ ▓  =>      ▒
//!     ▒ ▒        ▒ ▒        ▒ ▒        ▒ ▒
//! ```
//! In each turn, the board fits within a 4x4 grid, but the 4x4 grid shifts to the right.
//!
//! This library uses _absolute_ coordinates, meaning that the same "spot on the table",
//! called a "field" by this library, has consistent coordinates in every turn,
//! no matter where the current 4x4 boundary is.
//! As a result, these coordinates may be negative, or larger than 4. They are represented
//! as an `i8`.

pub use board::*;
pub use cards::*;
pub use cards_set::*;
pub use errors::*;
pub use player_state::*;
pub use protocol_types::*;
pub use turn::*;
pub use visualization::*;

#[cfg(test)]
mod arbitrary;
mod board;
mod cards;
mod cards_set;
mod errors;
mod player_state;
mod protocol_types;
mod turn;
mod visualization;
