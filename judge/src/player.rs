use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use gomori::{Card, CardsSet, Color, Request, BLACK_CARDS, RED_CARDS};
use rand::rngs::StdRng;
use tracing::trace;

use crate::recording::Recorder;

pub struct Player {
    pub name: String,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    // A re-usable buffer for IO.
    // Should always be empty before and after perform_request().
    buf: String,
}

pub struct PlayerWithGameState<'a> {
    pub player: &'a mut Player,
    pub state: PlayerGameState,
}

/// The state for a single player.
pub struct PlayerGameState {
    pub draw_pile: Vec<Card>,
    pub hand: [Card; 5],
    pub won_cards: CardsSet,
}

impl Player {
    pub fn new(name: &str, executable_path: &str) -> anyhow::Result<Self> {
        let child_proc = Command::new(executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self {
            name: String::from(name),
            stdin: child_proc.stdin.expect("Could not access stdin"),
            stdout: BufReader::new(child_proc.stdout.expect("Could not access stdout")),
            buf: String::new(),
        })
    }
}

impl<'a> PlayerWithGameState<'a> {
    pub fn new(player: &'a mut Player, color: Color, rng: &mut StdRng) -> Self {
        let deck = match color {
            Color::Black => &BLACK_CARDS,
            Color::Red => &RED_CARDS,
        };

        Self {
            player,
            state: PlayerGameState::new(deck, rng),
        }
    }

    pub fn perform_request<T: serde::de::DeserializeOwned + std::fmt::Debug>(
        &mut self,
        recorder: &mut Option<Recorder>,
        req: &Request,
    ) -> anyhow::Result<T> {
        let mut req_json = serde_json::to_string(req)?;
        trace!(name: "Sending request", player = &self.player.name, request = %req_json);
        req_json.push('\n');
        self.player.stdin.write_all(req_json.as_bytes())?;
        self.player.stdin.flush()?;
        self.player.buf.clear();
        self.player.stdout.read_line(&mut self.player.buf)?;
        let serialized_response = self.player.buf.trim_end();
        let response = serde_json::from_str::<T>(serialized_response)?;
        trace!(name: "Recieved response", player = &self.player.name, response = %serialized_response);

        if let Some(recorder) = recorder {
            recorder.store_request(
                &self.player.name,
                req_json,
                String::from(serialized_response),
            );
        }
        Ok(response)
    }
}

impl PlayerGameState {
    pub fn new(all_cards: &[Card; 26], rng: &mut StdRng) -> Self {
        let mut draw_pile = Vec::from(all_cards);

        use rand::seq::SliceRandom;
        draw_pile.shuffle(rng);
        let hand = draw_pile.split_off(26 - 5).try_into().unwrap();

        Self {
            draw_pile,
            hand,
            won_cards: CardsSet::new(),
        }
    }
}
