use gomori::{Card, CardToPlay, CardsSet, Color, Field, PlayTurnResponse, Rank};
use gomori_bot_utils::Bot;

use clap::Parser;
use max_bot::GameState;
use tracing::debug;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
struct Args {
    /// A log level among "off", "error", "warn", "info", "debug", "trace"
    #[arg(short, long, default_value = "info")]
    log_level: LevelFilter,
}

struct DFSBot {}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    initialize_logging(args.log_level);
    DFSBot {}.run()
}

fn initialize_logging(level: LevelFilter) {
    let format = tracing_subscriber::fmt::format()
        .with_target(false)
        .compact();

    let filter = Targets::new().with_default(level);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(format)
                .with_writer(std::io::stderr),
        )
        .with(filter)
        .init();
}

impl Bot for DFSBot {
    fn new_game(&mut self, _color: Color) {}

    fn play_first_turn(&mut self, cards: [Card; 5]) -> Card {
        // Don't waste a "special" card on the first move
        for card in cards {
            match card.rank {
                Rank::Jack | Rank::Queen | Rank::King | Rank::Ace => {}
                _ => {
                    return card;
                }
            }
        }
        cards[0]
    }

    fn play_turn(&mut self, cards: [Card; 5], fields: Vec<Field>, _: CardsSet) -> PlayTurnResponse {
        let root = GameState::initial(cards, fields);
        let cards_to_play = search_unroll(&root);
        PlayTurnResponse(cards_to_play)
    }
}

fn search_unroll(state0: &GameState) -> Vec<CardToPlay> {
    let mut best_score: i8 = i8::MIN;
    let mut best_actions = [None, None, None, None, None];
    for action0 in state0.possible_actions() {
        let state1 = state0.apply_action(action0);
        if state1.is_terminal() {
            if state1.score_delta > best_score {
                best_score = state1.score_delta;
                best_actions = [Some(action0), None, None, None, None];
                debug!("New best score with action0 {:?}", action0);
            }
            continue;
        }
        for action1 in state1.possible_actions() {
            let state2 = state1.apply_action(action1);
            if state2.is_terminal() {
                if state2.score_delta > best_score {
                    best_score = state2.score_delta;
                    best_actions = [Some(action0), Some(action1), None, None, None];
                    debug!("New best score with action1 {:?}", action1);
                }
                continue;
            }
            for action2 in state2.possible_actions() {
                let state3 = state2.apply_action(action2);
                if state3.is_terminal() {
                    if state3.score_delta > best_score {
                        best_score = state3.score_delta;
                        best_actions = [Some(action0), Some(action1), Some(action2), None, None];
                        debug!("New best score with action2 {:?}", action2);
                    }
                    continue;
                }
                for action3 in state3.possible_actions() {
                    let state4 = state3.apply_action(action3);
                    if state4.is_terminal() {
                        if state4.score_delta > best_score {
                            best_score = state4.score_delta;
                            best_actions = [
                                Some(action0),
                                Some(action1),
                                Some(action2),
                                Some(action3),
                                None,
                            ];
                            debug!("New best score with action3 {:?}", action3);
                        }
                        continue;
                    }
                    for action4 in state4.possible_actions() {
                        let state5 = state4.apply_action(action4);
                        if state5.is_terminal() {
                            if state5.score_delta >= best_score {
                                best_score = state5.score_delta;
                                best_actions = [
                                    Some(action0),
                                    Some(action1),
                                    Some(action2),
                                    Some(action3),
                                    Some(action4),
                                ];
                                debug!("New best score with action4 {:?}", best_actions);
                            }
                            continue;
                        }
                    }
                }
            }
        }
    }
    best_actions.into_iter().filter_map(|x| x).collect()
}
