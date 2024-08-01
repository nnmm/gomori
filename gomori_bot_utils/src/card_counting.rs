use crate::Bot;
use gomori::{
    Board, Card, CardsSet, Color, CompactField, Field, PlayTurnResponse, BLACK_CARDS_SET,
    RED_CARDS_SET,
};

/// Information about the cards in the game, derived from
/// observing all played cards.
///
/// This can be automatically updated by implementing [`HasCardCounter`] for your bot
/// and wrapping it in a `CardCountingWrapper`.
#[derive(Clone, Copy, Debug)]
pub struct CardCounter {
    /// Cards in our draw pile.
    pub draw_pile: CardsSet,
    /// Cards in the opponent's draw pile + hand.
    /// We don't have any information to distinguish the two.
    pub available_cards_opponent: CardsSet,
    /// Cards won by us.
    pub cards_won_self: CardsSet,
    /// Cards won by our opponent.
    pub cards_won_opponent: CardsSet,
}

impl CardCounter {
    fn new(color: Color) -> Self {
        let (draw_pile, available_cards_opponent) = match color {
            Color::Black => (BLACK_CARDS_SET, RED_CARDS_SET),
            Color::Red => (RED_CARDS_SET, BLACK_CARDS_SET),
        };
        Self {
            draw_pile,
            available_cards_opponent,
            cards_won_self: CardsSet::new(),
            cards_won_opponent: CardsSet::new(),
        }
    }
}

impl Default for CardCounter {
    fn default() -> Self {
        CardCounter {
            draw_pile: CardsSet::new(),
            available_cards_opponent: CardsSet::new(),
            cards_won_self: CardsSet::new(),
            cards_won_opponent: CardsSet::new(),
        }
    }
}

/// Implement this trait on your bot to allow it to be used with a [`CardCountingWrapper`].
///
/// Basically the same as `DerefMut<Target=CardCounter>`
pub trait HasCardCounter {
    fn get_counter(&mut self) -> &mut CardCounter;
}

/// Automatically counts cards for your bot.
pub struct CardCountingWrapper<T>
where
    T: HasCardCounter,
{
    bot: T,
}

impl<T> CardCountingWrapper<T>
where
    T: HasCardCounter,
{
    pub fn new(bot: T) -> Self {
        Self { bot }
    }
}

impl<T: HasCardCounter + Bot> Bot for CardCountingWrapper<T> {
    fn new_game(&mut self, color: Color) {
        *self.bot.get_counter() = CardCounter::new(color);
        self.bot.new_game(color);
    }

    fn play_first_turn(&mut self, cards: [Card; 5]) -> Card {
        self.bot.get_counter().draw_pile &= !CardsSet::from_iter(cards);
        self.bot.play_first_turn(cards)
    }

    fn play_turn(
        &mut self,
        cards: [Card; 5],
        fields: Vec<Field>,
        cards_won_by_opponent: CardsSet,
    ) -> PlayTurnResponse {
        self.bot.get_counter().draw_pile &= !CardsSet::from_iter(cards);
        self.bot.get_counter().cards_won_opponent |= cards_won_by_opponent;
        self.bot.get_counter().available_cards_opponent &= !cards_won_by_opponent;
        for field in &fields {
            self.bot.get_counter().available_cards_opponent &=
                !CompactField::from(field).all_cards();
        }
        let mut board = Board::new(&fields);
        let response = self.bot.play_turn(cards, fields, cards_won_by_opponent);
        for &card_to_play in &response.0 {
            if let Ok(effects) = board.calculate(card_to_play) {
                self.bot.get_counter().cards_won_self |= effects.cards_won;
                board = effects.execute();
            } else {
                // Let the judge handle the illegal card play.
                return response;
            }
        }
        response
    }
}
