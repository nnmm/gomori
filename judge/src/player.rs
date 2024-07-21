use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use anyhow::Context;
use gomori::{Color, PlayerState, Request};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use tracing::{info, trace};

use crate::recording::Recorder;

pub struct Player {
    pub name: String,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    // A re-usable buffer for IO.
    // Should always be empty before and after perform_request().
    buf: String,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerConfig {
    pub nick: String,
    pub cmd: Vec<String>,
}

impl PlayerConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let inner = || -> anyhow::Result<PlayerConfig> {
            let f = File::open(path)?;
            let config = serde_json::from_reader::<_, PlayerConfig>(BufReader::new(f))
                .context("Could not parse file as PlayerConfig JSON")?;
            if config.cmd.is_empty() {
                anyhow::bail!("'cmd' field cannot be empty.");
            }
            Ok(config)
        };
        inner().with_context(|| format!("Could not read config file '{}'", path.display()))
    }
}

pub struct PlayerWithGameState<'a> {
    pub player: &'a mut Player,
    pub state: PlayerState,
}

impl Player {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let config = PlayerConfig::load(path)?;
        let child_proc = Command::new(&config.cmd[0])
            .args(&config.cmd[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn child process {:?}", &config.cmd))?;
        info!(cmd = ?config.cmd, "Spawned child process");

        Ok(Self {
            name: config.nick,
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

    pub fn perform_request<T: serde::de::DeserializeOwned>(
        &mut self,
        recorder: &mut Option<Recorder>,
        req: &Request,
    ) -> anyhow::Result<T> {
        let mut inner = || -> anyhow::Result<T> {
            let mut req_json = serde_json::to_string(req)?;
            trace!(name: "Sending request", player = &self.player.name, request = %req_json);
            req_json.push('\n');
            self.player
                .stdin
                .write_all(req_json.as_bytes())
                .context("Could not send request")?;
            self.player.stdin.flush()?;
            self.player.buf.clear();
            self.player.stdout.read_line(&mut self.player.buf)?;
            let serialized_response = self.player.buf.trim_end();
            let response = serde_json::from_str::<T>(serialized_response).with_context(|| {
                format!("Could not parse response '{}' as JSON", serialized_response)
            })?;
            trace!(name: "Recieved response", player = &self.player.name, response = %serialized_response);
            if let Some(recorder) = recorder {
                recorder.store_request(
                    &self.player.name,
                    req_json,
                    String::from(serialized_response),
                );
            }
            Ok(response)
        };
        inner().with_context(|| format!("Failed to make a request to '{}'", self.player.name))
    }
}
