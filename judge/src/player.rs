use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use gomori::{Color, PlayerState, Request};
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
    pub state: PlayerState,
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
        Self {
            player,
            state: PlayerState::new(color, rng),
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
