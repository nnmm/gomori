use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub struct Recorder {
    num: usize,
    directory: PathBuf,
    requests: Vec<RequestToPlayer>,
}

impl Recorder {
    pub fn new(directory: PathBuf) -> anyhow::Result<Self> {
        if !directory.is_dir() {
            anyhow::bail!("Directory '{}' does not exist", directory.display());
        }
        Ok(Self {
            num: 1,
            directory,
            requests: Vec::new(),
        })
    }

    pub fn store_request(&mut self, player: &str, request: String, response: String) {
        self.requests.push(RequestToPlayer {
            player: String::from(player),
            request,
            response,
        });
    }

    // TODO: Refactor - this is super ugly
    // I don't use serde here but write JSON manually because the request/response
    // are already JSON strings and serde escapes them.
    pub fn write_game_recording(&mut self) -> anyhow::Result<()> {
        let filepath = self.directory.join(format!("game_{:0>6}.json", self.num));
        let mut writer = BufWriter::new(File::create(filepath)?);
        write!(writer, "[")?;
        let mut first = true;
        for req in std::mem::take(&mut self.requests).into_iter() {
            if !first {
                write!(writer, ",")?;
            } else {
                first = false;
            }
            write!(
                writer,
                "\n  {{\n    \"player\": \"{}\",\n    \"request\": {},\n    \"response\": {}\n  }}",
                req.player, req.request, req.response
            )?;
        }
        write!(writer, "\n]")?;
        self.num += 1;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameRecording {
    requests: Vec<RequestToPlayer>,
}

#[derive(Serialize, Deserialize)]
pub struct RequestToPlayer {
    player: String,
    request: String,
    response: String,
}

// #[derive(Serialize, Deserialize)]
// pub enum Response {
//     Okay,
//     Card(Card),
//     PlayTurnResponse(PlayTurnResponse),
// }

// impl From<Okay> for Response {
//     fn from(_: Okay) -> Response {
//         Response::Okay
//     }
// }

// impl From<Card> for Response {
//     fn from(card: Card) -> Response {
//         Response::Card(card)
//     }
// }

// impl From<PlayTurnResponse> for Response {
//     fn from(action: PlayTurnResponse) -> Response {
//         Response::PlayTurnResponse(action)
//     }
// }
