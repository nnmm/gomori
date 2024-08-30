use std::collections::HashMap;
use std::path::PathBuf;

use clap::Parser;
use itertools::Itertools;
use judge::{play_game, GameResult, Player, PlayerConfig, Recorder};
use rand::rngs::StdRng;
use rand::SeedableRng;
use tracing::{debug, info};
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
struct Args {
    /// Path to the config JSON files of players
    #[clap(num_args(2..), value_delimiter = ' ')]
    player_configs: Vec<PathBuf>,

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

#[derive(Default)]
struct MatchScore {
    wins: [usize; 2],
    illegal_moves: [usize; 2],
    ties: usize,
}

fn play_matchup(
    player_1: &mut Player,
    player_2: &mut Player,
    num_games: usize,
    rng: &mut StdRng,
    stop_on_illegal_move: bool,
    recorder: &mut Option<Recorder>,
) -> anyhow::Result<MatchScore> {
    let player_names = [player_1.name.clone(), player_2.name.clone()];
    let mut match_score = MatchScore::default();

    for game_idx in 0..num_games {
        match play_game(rng, player_1, player_2, recorder)? {
            GameResult::WonByPlayer { player_idx } => {
                debug!(winner = player_names[player_idx], game_idx);
                match_score.wins[player_idx] += 1;
            }
            GameResult::Tie => {
                debug!(game_idx, "Tie");
                match_score.ties += 1;
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
                if stop_on_illegal_move {
                    break;
                } else {
                    match_score.wins[1 - player_idx] += 1;
                    match_score.illegal_moves[player_idx] += 1;
                }
            }
        }
    }

    let paren_1 = if match_score.illegal_moves[1] > 0 {
        format!(
            " ({} through illegal moves by player 2)",
            match_score.illegal_moves[1]
        )
    } else {
        String::new()
    };
    let paren_2 = if match_score.illegal_moves[0] > 0 {
        format!(
            " ({} through illegal moves by player 1)",
            match_score.illegal_moves[0]
        )
    } else {
        String::new()
    };
    eprintln!(
        "End result:\n- {} wins by {}{}\n- {} wins by {}{}\n- {} ties",
        match_score.wins[0],
        &player_1.name,
        paren_1,
        match_score.wins[1],
        player_2.name,
        paren_2,
        match_score.ties
    );

    Ok(match_score)
}

// prints an upper triangular matrix of the results of the tournament
fn print_tournament_results(
    player_configs: &[PlayerConfig],
    match_results: &HashMap<(usize, usize), Option<MatchScore>>,
) {
    println!("\nTournament results (p1 win %, p2 win %, tie %):\n");
    print!(" {:19} |", "p1 ↓           p2 →");
    for j in (0..player_configs.len()).rev() {
        print!(" {:19} |", player_configs[j].nick);
    }
    println!();
    for i in 0..player_configs.len() {
        for _ in 0..player_configs.len() - i + 1 {
            print!("---------------------|");
        }
        println!();
        print!(" {:19} |", player_configs[i].nick);
        for j in (0..player_configs.len()).rev() {
            if i >= j {
                print!("    ");
            } else if let Some(Some(score)) = match_results.get(&(i, j)) {
                let num_games = score.wins[0] + score.wins[1] + score.ties;
                let win_1_percentage = score.wins[0] as f32 / num_games as f32 * 100.0;
                let win_2_percentage = score.wins[1] as f32 / num_games as f32 * 100.0;
                let tie_percentage = score.ties as f32 / num_games as f32 * 100.0;
                print!(
                    "{:5.1}% {:5.1}% {:5.1}% |",
                    win_1_percentage, win_2_percentage, tie_percentage
                );
            } else {
                print!(" {:19} |", "N/A");
            }
        }
        println!();
    }
    println!("---------------------|");
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    initialize_logging(args.log_level);

    // Get a random seed
    let seed = args.seed.unwrap_or_else(rand::random);
    info!(seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let mut recorder = if let Some(dir_path) = args.record_games_to_directory {
        Some(Recorder::new(dir_path)?)
    } else {
        None
    };

    let player_configs = args
        .player_configs
        .iter()
        .map(|path| PlayerConfig::load(path))
        .collect::<Result<Vec<PlayerConfig>, anyhow::Error>>()?;

    let matchups: Vec<(usize, usize)> = (0..player_configs.len()).tuple_combinations().collect();

    let mut match_results: HashMap<(usize, usize), Option<MatchScore>> = HashMap::new();
    for (i1, i2) in matchups {
        let mut player_1 = Player::from_config(&player_configs[i1])?;
        let mut player_2 = Player::from_config(&player_configs[i2])?;

        let match_score = play_matchup(
            &mut player_1,
            &mut player_2,
            args.num_games,
            &mut rng,
            args.stop_on_illegal_move,
            &mut recorder,
        )?;

        match_results.insert((i1, i2), Some(match_score));
    }

    if player_configs.len() > 2 {
        print_tournament_results(&player_configs, &match_results);
    }

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
