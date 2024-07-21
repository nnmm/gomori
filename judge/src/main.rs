use std::path::PathBuf;

use clap::Parser;
use judge::{play_game, GameResult, Player, Recorder};
use rand::rngs::StdRng;
use rand::SeedableRng;
use tracing::{debug, info};
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
struct Args {
    /// Path to the config JSON file for player 1
    player_1_config: PathBuf,

    /// Path to the config JSON file for player 2
    player_2_config: PathBuf,

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

    /// A log level among "off", "error", "warn", "info", "debug", "trace"
    #[arg(short, long, default_value = "info")]
    log_level: LevelFilter,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    initialize_logging(args.log_level);

    let mut player_1 = Player::new(&args.player_1_config)?;
    let mut player_2 = Player::new(&args.player_2_config)?;

    let player_names = [player_1.name.clone(), player_2.name.clone()];

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
        wins[0], &player_1.name, paren_1, wins[1], player_2.name, paren_2, ties
    );
    Ok(())
}

fn initialize_logging(level: LevelFilter) {
    let format = tracing_subscriber::fmt::format()
        .with_target(false)
        .compact();

    let filter = Targets::new().with_default(level);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().event_format(format))
        .with(filter)
        .init();
}
