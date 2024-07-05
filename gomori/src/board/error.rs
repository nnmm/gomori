use crate::Card;

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
