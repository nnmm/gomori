mod card_counting;
pub use card_counting::*;

use gomori::{Card, CardsSet, Color, Field, Okay, PlayTurnResponse, Request};

/// A trait to simplify writing bots.
pub trait Bot {
    fn new_game(&mut self, color: Color);
    fn play_first_turn(&mut self, cards: [Card; 5]) -> Card;
    fn play_turn(
        &mut self,
        cards: [Card; 5],
        fields: Vec<Field>,
        cards_won_by_opponent: CardsSet,
    ) -> PlayTurnResponse;

    fn run(&mut self) -> anyhow::Result<()> {
        // Communication happens through stdin/stdout.
        // Stderr can be used for logging.
        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();
        let mut buf = String::new();

        loop {
            // Read the next line into buf
            buf.clear(); // because stdin.read_line() appends to the buffer
            use std::io::BufRead;
            let num_bytes_read = stdin.read_line(&mut buf)?;
            if num_bytes_read == 0 {
                // 0 bytes read means EOF - the judge has exited.
                break Ok(());
            }

            let req = serde_json::from_str::<Request>(buf.trim_end())?;

            match req {
                Request::NewGame { color } => {
                    self.new_game(color);
                    serde_json::to_writer(&mut stdout, &Okay())?;
                }
                Request::PlayFirstTurn { cards } => {
                    serde_json::to_writer(&mut stdout, &self.play_first_turn(cards))?
                }
                Request::PlayTurn {
                    cards,
                    fields,
                    cards_won_by_opponent,
                } => serde_json::to_writer(
                    &mut stdout,
                    &self.play_turn(cards, fields, CardsSet::from_iter(cards_won_by_opponent)),
                )?,
                Request::Bye => break Ok(()),
            }
            use std::io::Write;
            writeln!(stdout)?;
            stdout.flush()?;
        }
    }
}
