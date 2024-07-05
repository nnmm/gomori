use gomori::{Card, IllegalCardPlayed};

#[derive(Debug)]
/// Error type for one turn.
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
