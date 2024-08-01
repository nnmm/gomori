use gomori::{
    BitBoard, BitBoardIter, Board, CalculatedEffects, Card, CardToPlay, CardsSet, Field, Rank,
};

#[derive(Clone, Debug)]
pub struct GameState {
    pub cards: CardsSet,
    pub board: Board,
    pub score_delta: i8,
}

impl GameState {
    pub fn initial(cards: [Card; 5], fields: Vec<Field>) -> Self {
        Self {
            cards: CardsSet::from_iter(cards),
            board: Board::new(&fields),
            score_delta: 0,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn apply_action(&self, ctp: CardToPlay) -> Self {
        let calc @ CalculatedEffects {
            combo,
            cards_won: cards_won_this_turn,
            ..
        } = self.board.calculate(ctp).unwrap();
        let board = calc.execute();

        let cards = if combo {
            self.cards.remove(ctp.card)
        } else {
            CardsSet::new()
        };

        GameState {
            board,
            cards,
            score_delta: self.score_delta + i8::try_from(cards_won_this_turn.len()).unwrap(),
        }
    }

    /// Returns all possible `CardToPlay`
    pub fn possible_actions(&self) -> impl Iterator<Item = CardToPlay> {
        let king_tgts =
            self.board.diamonds() | self.board.hearts() | self.board.spades() | self.board.clubs();

        let board = self.board.clone(); // To make the return type fully owned
        self.cards
            .into_iter()
            .flat_map(move |card| {
                board
                    .locations_for_card(card)
                    .into_iter()
                    .map(move |loc| (card, loc))
            })
            .flat_map(move |(card, loc)| {
                // Internal helper for use in flat_map(), which requires a single iterator type to be
                // returned.
                //
                // Either produces a list of `Some(<coordinate>)`, or yields a single `None`.
                enum KingTgtsIter {
                    King { iter: BitBoardIter },
                    Regular { done: bool },
                }

                impl KingTgtsIter {
                    fn new(card: Card, possible_tgts: BitBoard) -> Self {
                        match card.rank {
                            Rank::King => Self::King {
                                iter: possible_tgts.into_iter(),
                            },
                            _ => Self::Regular { done: false },
                        }
                    }
                }

                impl Iterator for KingTgtsIter {
                    type Item = Option<(i8, i8)>;

                    fn next(&mut self) -> Option<Self::Item> {
                        match self {
                            KingTgtsIter::King { iter } => iter.next().map(Some),
                            KingTgtsIter::Regular { done: true } => None,
                            KingTgtsIter::Regular { done } => {
                                *done = true;
                                Some(None)
                            }
                        }
                    }
                }

                KingTgtsIter::new(card, king_tgts).map(move |tgt| CardToPlay {
                    card,
                    i: loc.0,
                    j: loc.1,
                    target_field_for_king_ability: tgt,
                })
            })
    }
}
