use std::cmp::Ordering;

use gomori::{Card, Color, Okay, PlayTurnResponse, Request};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::error::IllegalMove;
use crate::player::{Player, PlayerWithGameState};
use crate::recording::Recorder;
use crate::turn::{execute_first_turn, execute_turn, TurnOutcome};

pub enum GameResult {
    WonByPlayer { player_idx: usize },
    Tie,
    IllegalMoveByPlayer { player_idx: usize, err: IllegalMove },
}

/// Returns an error only on communication failure, not when an
/// illegal move is played.
pub fn play_game(
    rng: &mut StdRng,
    player_1: &mut Player,
    player_2: &mut Player,
    recorder: &mut Option<Recorder>,
) -> anyhow::Result<GameResult> {
    // Assign one bot the red cards and the other the black cards randomly
    let [player_1_color, player_2_color] = {
        let mut arr = [Color::Red, Color::Black];
        arr.shuffle(rng);
        arr
    };

    // Bundle everything up in a PlayerWithGameState struct, which tracks the player's state during this game
    let mut players = [
        PlayerWithGameState::new(player_1, player_1_color, rng),
        PlayerWithGameState::new(player_2, player_2_color, rng),
    ];

    // Inform the players about the new game, so that they can reset their state
    let _: Okay = players[0].perform_request(
        recorder,
        &Request::NewGame {
            color: player_1_color,
        },
    )?;
    let _: Okay = players[1].perform_request(
        recorder,
        &Request::NewGame {
            color: player_2_color,
        },
    )?;

    // Randomly pick a starting player
    let mut current_player_idx = if rng.gen::<bool>() { 1 } else { 0 };

    // Play the first turn. This one is special.
    let req = Request::PlayFirstTurn {
        cards: players[current_player_idx].state.hand,
    };
    let card: Card = players[current_player_idx].perform_request(recorder, &req)?;
    let mut board = match execute_first_turn(&mut players[current_player_idx].state, card) {
        Ok(board) => board,
        Err(err) => {
            return Ok(GameResult::IllegalMoveByPlayer {
                player_idx: current_player_idx,
                err,
            })
        }
    };

    let mut turn_skipped = false;
    loop {
        // eprintln!("{}", board);
        current_player_idx = 1 - current_player_idx;
        let current_player = &mut players[current_player_idx];
        let req = Request::PlayTurn {
            cards: current_player.state.hand,
            fields: board.to_fields_vec(),
        };
        let action: PlayTurnResponse = current_player.perform_request(recorder, &req)?;
        match execute_turn(&mut current_player.state, &mut board, action) {
            Ok(TurnOutcome::Normal) => {
                turn_skipped = false;
            }
            Ok(TurnOutcome::GameEnded) => {
                break;
            }
            Ok(TurnOutcome::Skipped) => {
                if turn_skipped {
                    break; // When both players couldn't play a card, the game ends
                } else {
                    turn_skipped = true;
                }
            }
            Err(err) => {
                return Ok(GameResult::IllegalMoveByPlayer {
                    player_idx: current_player_idx,
                    err,
                })
            }
        };
    }

    if let Some(rec) = recorder {
        rec.write_game_recording()?;
    }

    // Report who won
    let num_cards_0 = players[0].state.won_cards.len();
    let num_cards_1 = players[1].state.won_cards.len();
    let game_result = match num_cards_0.cmp(&num_cards_1) {
        Ordering::Less => GameResult::WonByPlayer { player_idx: 1 },
        Ordering::Equal => GameResult::Tie,
        Ordering::Greater => GameResult::WonByPlayer { player_idx: 0 },
    };
    Ok(game_result)
}
