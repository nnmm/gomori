use std::collections::BTreeSet;

use clap::Parser;
use gomori::{Board, Card, CardToPlay, CardsSet, Color, Field, PlayTurnResponse, Rank};
use gomori_bot_utils::Bot;
use rand::rngs::StdRng;
use rand::{seq::SliceRandom, SeedableRng};

#[derive(Parser)]
struct Args {
    /// RNG seed
    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let seed = args.seed.unwrap_or_else(rand::random);
    let rng = StdRng::seed_from_u64(seed);

    GreedyBot { rng }.run()
}

struct GreedyBot {
    rng: StdRng,
}

impl GreedyBot {
    fn fix_up_target_field_for_king_ability(
        &mut self,
        board: &Board,
        card_to_play: &mut CardToPlay,
    ) {
        let CardToPlay { card, i, j, .. } = card_to_play;
        card_to_play.target_field_for_king_ability = (card.rank == Rank::King).then(|| {
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

    fn best_card_placement(&mut self, board: &Board, cards: &BTreeSet<Card>) -> Option<CardToPlay> {
        let mut top_choices: Vec<CardToPlay> = Vec::new();
        let mut top_score = 0;
        for &card in cards.iter() {
            for (i, j) in board.locations_for_card(card) {
                let mut card_to_play = CardToPlay {
                    card,
                    i,
                    j,
                    target_field_for_king_ability: None,
                };
                self.fix_up_target_field_for_king_ability(board, &mut card_to_play);
                let card_calculation = board
                    .calculate(card_to_play)
                    .expect("Calculate error despite card being a possible location");
                // Add a bonus for combo moves, because they have the potential to
                // give further points
                let score = card_calculation.cards_won.len() * 2
                    + if card_calculation.combo { 1 } else { 0 };
                match score.cmp(&top_score) {
                    std::cmp::Ordering::Less => {}
                    std::cmp::Ordering::Equal => {
                        top_choices.push(card_to_play);
                    }
                    std::cmp::Ordering::Greater => {
                        top_choices = vec![card_to_play];
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

    fn play_turn(
        &mut self,
        cards: [Card; 5],
        fields: Vec<Field>,
        _cards_won_by_opponent: CardsSet,
    ) -> PlayTurnResponse {
        let mut cards_to_play = vec![];

        let mut board = Board::new(&fields);
        let mut remaining_cards: BTreeSet<Card> = BTreeSet::from(cards);

        while let Some(card_to_play) = self.best_card_placement(&board, &remaining_cards) {
            cards_to_play.push(card_to_play);
            remaining_cards.remove(&card_to_play.card);
            let plan = board.calculate(card_to_play).unwrap();
            if !plan.combo {
                break;
            }
            board = plan.execute();
        }
        PlayTurnResponse(cards_to_play)
    }
}
