use std::collections::BTreeSet;

use clap::Parser;
use gomori::{Board, Card, CardToPlay, CardsSet, Color, Field, PlayTurnResponse, Rank};
use gomori_bot_utils::Bot;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

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

    RandomBot { rng }.run()
}

struct RandomBot {
    rng: StdRng,
}

fn possible_card_placements(board: &Board, cards: &BTreeSet<Card>) -> Vec<(i8, i8, Card)> {
    let mut moves = Vec::new();
    for &card in cards.iter() {
        moves.extend(
            board
                .locations_for_card(card)
                .into_iter()
                .map(|(i, j)| (i, j, card)),
        );
    }
    moves
}

impl Bot for RandomBot {
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
        while let Some((i, j, card)) =
            possible_card_placements(&board, &remaining_cards).choose(&mut self.rng)
        {
            let target_field_for_king_ability = (card.rank == Rank::King).then(|| {
                let flippable_cards: Vec<(i8, i8)> = board
                    .iter()
                    .filter_map(|&(i, j, field)| field.top_card().map(|_| (i, j)))
                    .collect();
                flippable_cards
                    .choose(&mut self.rng)
                    .copied()
                    .unwrap_or((*i, *j))
            });
            let ctp = CardToPlay {
                i: *i,
                j: *j,
                card: *card,
                target_field_for_king_ability,
            };
            cards_to_play.push(ctp);
            remaining_cards.remove(card);
            let calculation_result = board.calculate(ctp).unwrap();
            if !calculation_result.combo {
                break;
            } else {
                board = calculation_result.execute();
            }
        }
        PlayTurnResponse(cards_to_play)
    }
}
