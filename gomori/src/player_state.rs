use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::{Card, CardsSet, Color, BLACK_CARDS, RED_CARDS};

/// The state for a single player during one game.
#[derive(Clone, Debug)]
pub struct PlayerState {
    pub draw_pile: Vec<Card>,
    pub hand: [Card; 5],
    pub won_cards: CardsSet,
}

impl PlayerState {
    pub fn new(color: Color, rng: &mut StdRng) -> Self {
        let mut draw_pile = Vec::from(match color {
            Color::Black => &BLACK_CARDS,
            Color::Red => &RED_CARDS,
        });
        draw_pile.shuffle(rng);
        let hand = draw_pile.split_off(26 - 5).try_into().unwrap();

        Self {
            draw_pile,
            hand,
            won_cards: CardsSet::new(),
        }
    }
}
