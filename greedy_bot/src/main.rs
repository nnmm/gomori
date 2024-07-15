use std::collections::BTreeSet;

use gomori::{Board, Card, CardToPlace, Color, Field, PlayTurnResponse, Rank};
use gomori_bot_utils::Bot;
use rand::{rngs::ThreadRng, seq::SliceRandom};

fn main() -> anyhow::Result<()> {
    GreedyBot {
        rng: rand::thread_rng(),
    }
    .run()
}

struct GreedyBot {
    rng: ThreadRng,
}

impl GreedyBot {
    fn fix_up_target_field_for_king_ability(
        &mut self,
        board: &Board,
        card_to_place: &mut CardToPlace,
    ) {
        let CardToPlace { card, i, j, .. } = card_to_place;
        card_to_place.target_field_for_king_ability = (card.rank == Rank::King).then(|| {
            let flippable_cards: Vec<_> = board
                .iter()
                .filter(|(_i, _j, field)| field.top_card().is_some())
                .collect();
            flippable_cards
                .choose(&mut self.rng)
                .map(|(i, j, _)| (*i, *j))
                .unwrap_or((*i, *j))
        });
    }

    fn best_card_placement(
        &mut self,
        board: &Board,
        cards: &BTreeSet<Card>,
    ) -> Option<CardToPlace> {
        let mut top_choices: Vec<CardToPlace> = Vec::new();
        let mut top_score = 0;
        for &card in cards.iter() {
            for (i, j) in board.locations_for_card(card) {
                let mut card_to_place = CardToPlace {
                    card,
                    i,
                    j,
                    target_field_for_king_ability: None,
                };
                self.fix_up_target_field_for_king_ability(board, &mut card_to_place);
                let card_calculation = board
                    .calculate(card_to_place)
                    .expect("Calculate error despite card being a possible location");
                // Add a bonus for combo moves, because they have the potential to
                // give further points
                let score = card_calculation.cards_won.len() * 2
                    + if card_calculation.combo { 1 } else { 0 };
                match score.cmp(&top_score) {
                    std::cmp::Ordering::Less => {}
                    std::cmp::Ordering::Equal => {
                        top_choices.push(card_to_place);
                    }
                    std::cmp::Ordering::Greater => {
                        top_choices = vec![card_to_place];
                        top_score = score;
                    }
                }
            }
        }
        top_choices.choose(&mut self.rng).copied()
    }
}

impl Bot for GreedyBot {
    fn new_game(&mut self, _color: Color) {}

    fn play_first_turn(&mut self, cards: [Card; 5]) -> Card {
        *cards.choose(&mut self.rng).unwrap()
    }

    fn play_turn(&mut self, cards: [Card; 5], fields: Vec<Field>) -> PlayTurnResponse {
        let mut cards_to_place = vec![];

        let mut board = Board::new(&fields);
        let mut remaining_cards: BTreeSet<Card> = BTreeSet::from(cards);

        while let Some(card_to_place) = self.best_card_placement(&board, &remaining_cards) {
            cards_to_place.push(card_to_place);
            remaining_cards.remove(&card_to_place.card);
            let plan = board.calculate(card_to_place).unwrap();
            if !plan.combo {
                break;
            }
            board = plan.execute();
        }
        PlayTurnResponse(cards_to_place)
    }
}
