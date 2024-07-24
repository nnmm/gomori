use crate::Card;

/// The error type for [`Board::calculate()`](crate::Board::calculate), i.e. for playing a single card.
#[derive(Debug, PartialEq, Eq)]
pub enum IllegalCardPlayed {
    OutOfBounds,
    IncompatibleCard { existing_card: Card },
    NoTargetForKingAbility,
    TargetForKingAbilityDoesNotExist { tgt_i: i8, tgt_j: i8 },
    TargetForKingAbilityIsFaceDown { tgt_i: i8, tgt_j: i8 },
}

impl std::error::Error for IllegalCardPlayed {}

impl std::fmt::Display for IllegalCardPlayed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IllegalCardPlayed::OutOfBounds =>
                write!(f, "Card was played out of the bounds of the playing field"),
            IllegalCardPlayed::IncompatibleCard { existing_card } =>
                write!(f, "Card was played on top of an incompatible card, {}", existing_card.unicode_char()),
            IllegalCardPlayed::NoTargetForKingAbility =>
                write!(f, "A king was played on top of another card, but no target for its ability was specified"),
            IllegalCardPlayed::TargetForKingAbilityDoesNotExist { tgt_i, tgt_j } =>
                write!(f, "A king was played on top of another card, but the specified target card for its ability ({}, {}) does not exist", tgt_i, tgt_j),
            IllegalCardPlayed::TargetForKingAbilityIsFaceDown { tgt_i, tgt_j } =>
                write!(f, "A king was played on top of another card, but the specified target card for its ability ({}, {}) is already face-down", tgt_i, tgt_j),
        }
    }
}

#[derive(Debug)]
/// The error type for one turn.
pub enum IllegalMove {
    PlayedCardNotInHand,
    PlayedZeroCards,
    PlayedMoreThanFiveCards,
    IllegalCardPlayed {
        card_idx: usize,
        card: Card,
        err: IllegalCardPlayed,
    },
    PlayedCardAfterEndOfCombo {
        card_idx: usize,
    },
    PrematurelyEndedCombo {
        card_idx: usize,
    },
}

impl std::error::Error for IllegalMove {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IllegalMove::IllegalCardPlayed { err, .. } => Some(err),
            _ => None,
        }
    }
}

fn ordinal_number(num: usize) -> &'static str {
    match num {
        0 => "first",
        1 => "second",
        2 => "third",
        3 => "fourth",
        4 => "fifth",
        _ => panic!("ordinal_number called with {}", num),
    }
}

impl std::fmt::Display for IllegalMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IllegalMove::PlayedCardNotInHand => {
                write!(f, "Tried to play a card that was not in the player's hand")
            }
            IllegalMove::PlayedZeroCards => write!(f, "Tried to play zero cards"),
            IllegalMove::PlayedMoreThanFiveCards => write!(f, "Tried to play more than five cards"),
            IllegalMove::IllegalCardPlayed {
                card_idx,
                card,
                err: _,
            } => write!(
                f,
                "Error playing the {} card, which was {}",
                ordinal_number(*card_idx),
                card
            ),
            IllegalMove::PlayedCardAfterEndOfCombo { card_idx } => write!(
                f,
                "The {} card did not start a combo, but another card was played",
                ordinal_number(*card_idx)
            ),
            IllegalMove::PrematurelyEndedCombo { card_idx } => write!(
                f,
                "The {} card should be followed up by another card, but wasn't",
                ordinal_number(*card_idx)
            ),
        }
    }
}

#[cfg(feature = "python")]
mod python {
    use pyo3::create_exception;
    use pyo3::PyErr;

    use super::*;

    create_exception!(
        gomori,
        IllegalCardPlayedException,
        pyo3::exceptions::PyException,
        "Describes why the card cannot be played."
    );

    impl From<IllegalCardPlayed> for PyErr {
        fn from(err: IllegalCardPlayed) -> PyErr {
            IllegalCardPlayedException::new_err(err.to_string())
        }
    }

    create_exception!(
        gomori,
        IllegalMoveException,
        pyo3::exceptions::PyException,
        "Describes why a move is illegal."
    );

    impl From<IllegalMove> for PyErr {
        fn from(err: IllegalMove) -> PyErr {
            IllegalMoveException::new_err(err.to_string())
        }
    }
}
#[cfg(feature = "python")]
pub use python::*;
