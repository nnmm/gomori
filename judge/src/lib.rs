mod error;
mod game;
mod player;
mod recording;
mod turn;
pub use error::*;
pub use game::*;
pub use player::*;
pub use recording::*;
pub use turn::*;

pub struct Config {
    pub rng: rand::rngs::StdRng,
    pub recorder: Option<recording::Recorder>,
}
