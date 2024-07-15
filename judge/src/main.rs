use std::path::PathBuf;

use clap::Parser;
use judge::{play_game, GameResult, Player, Recorder};
use rand::rngs::StdRng;
use rand::SeedableRng;
use tracing::{debug, info};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

#[derive(Parser)]
struct Args {
    /// Path to the executable for player 1
    player_1: String,

    /// Path to the executable for player 2
    player_2: String,

    /// Nickname for player 1
    #[arg(long, default_value_t = String::from("Player 1"))]
    player_1_nick: String,

    /// Nickname for player 2
    #[arg(long, default_value_t = String::from("Player 2"))]
    player_2_nick: String,

    /// How many games to play
    #[arg(short, long, default_value_t = 100)]
    num_games: usize,

    /// RNG seed
    #[arg(long)]
    seed: Option<u64>,

    /// Stop the tournament as soon as one player makes an illegal move
    #[arg(short, long, default_value_t = false)]
    stop_on_illegal_move: bool,

    /// Record the game's interactions as JSON files into this directory
    #[arg(short, long)]
    record_games_to_directory: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let format = tracing_subscriber::fmt::format()
        .with_target(false)
        .compact();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .event_format(format)
        .init();

    let mut player_1 = Player::new(&args.player_1_nick, &args.player_1)?;
    let mut player_2 = Player::new(&args.player_2_nick, &args.player_2)?;

    let player_names = [&args.player_1_nick, &args.player_2_nick];

    let mut wins = [0, 0];
    let mut illegal_moves = [0, 0];
    let mut ties = 0;

    let mut recorder = if let Some(dir_path) = args.record_games_to_directory {
        Some(Recorder::new(dir_path)?)
    } else {
        None
    };

    // Get a random seed
    let seed = args.seed.unwrap_or_else(|| rand::random());
    info!(seed);
    let mut rng = StdRng::seed_from_u64(seed);

    for game_idx in 0..args.num_games {
        match play_game(&mut rng, &mut player_1, &mut player_2, &mut recorder)? {
            GameResult::WonByPlayer { player_idx } => {
                debug!(winner = player_names[player_idx], game_idx);
                wins[player_idx] += 1;
            }
            GameResult::Tie => {
                debug!(game_idx, "Tie");
                ties += 1;
            }
            GameResult::IllegalMoveByPlayer { player_idx, err } => {
                info!(
                    player = player_names[player_idx],
                    game_idx, "Illegal move by player"
                );
                let mut err_dyn = &err as &dyn std::error::Error;
                while let Some(src_err) = err_dyn.source() {
                    info!("{}", err_dyn);
                    err_dyn = src_err;
                }
                info!("{}", err_dyn);
                if args.stop_on_illegal_move {
                    break;
                } else {
                    wins[1 - player_idx] += 1;
                    illegal_moves[player_idx] += 1;
                }
            }
        }
    }

    let paren_1 = if illegal_moves[1] > 0 {
        format!(" ({} through illegal moves by player 2)", illegal_moves[1])
    } else {
        String::new()
    };
    let paren_2 = if illegal_moves[0] > 0 {
        format!(" ({} through illegal moves by player 1)", illegal_moves[0])
    } else {
        String::new()
    };
    eprintln!(
        "End result:\n- {} wins by {}{}\n- {} wins by {}{}\n- {} ties",
        wins[0], args.player_1_nick, paren_1, wins[1], args.player_2_nick, paren_2, ties
    );
    Ok(())
}
