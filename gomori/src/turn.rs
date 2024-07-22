use std::collections::BTreeSet;

use crate::{
    Board, Card, CardsSet, Field, IllegalMove, PlayCardCalculation, PlayTurnResponse, PlayerState,
};

/// Summarizes the outcome of playing a move.
pub enum TurnOutcome {
    Skipped,
    Normal { cards_won_this_turn: CardsSet },
    GameEnded,
}

pub fn execute_first_turn(
    state: &mut PlayerState,
    card_to_place: Card,
) -> Result<Board, IllegalMove> {
    // Draw a new card, and validate that the card was in the hand of the player
    let mut card_found = false;
    for card in state.hand.iter_mut() {
        if *card == card_to_place {
            let next_card: Card = state.draw_pile.pop().unwrap(); // Can't fail, since it's the first turn
            let _ = std::mem::replace(card, next_card);
            card_found = true;
        }
    }
    if !card_found {
        Err(IllegalMove::PlayedCardNotInHand)
    } else {
        Ok(Board::new(&[Field {
            i: 0,
            j: 0,
            top_card: Some(card_to_place),
            hidden_cards: BTreeSet::new(),
        }]))
    }
}

pub fn execute_turn(
    state: &mut PlayerState,
    board: &mut Board,
    action: PlayTurnResponse,
) -> Result<TurnOutcome, IllegalMove> {
    let mut cards_to_place = action.0;
    if cards_to_place.is_empty() {
        // The player wants to skip their turn. This is only allowed if there is no possible move.
        for &hand_card in &state.hand {
            if board.possible_to_place_card(hand_card) {
                return Err(IllegalMove::PlayedZeroCards);
            }
        }
        return Ok(TurnOutcome::Skipped);
    }
    if cards_to_place.len() > 5 {
        return Err(IllegalMove::PlayedMoreThanFiveCards);
    }

    let mut hand = BTreeSet::from(state.hand);

    cards_to_place.reverse(); // So that pop() goes through them in order

    let mut cards_won_this_turn = CardsSet::new();

    let mut card_idx = 0;
    while let Some(ctp) = cards_to_place.pop() {
        let card_is_valid = hand.remove(&ctp.card);
        if !card_is_valid {
            return Err(IllegalMove::PlayedCardNotInHand);
        }
        let calculation @ PlayCardCalculation {
            cards_won, combo, ..
        } = board
            .calculate(ctp)
            .map_err(|err| IllegalMove::IllegalCardPlayed {
                card_idx,
                card: ctp.card,
                err,
            })?;
        if !combo && !cards_to_place.is_empty() {
            return Err(IllegalMove::PlayedCardAfterEndOfCombo { card_idx });
        }
        *board = calculation.execute();
        if combo && cards_to_place.is_empty() {
            // Is there a possible move?
            for &hand_card in hand.iter() {
                if board.possible_to_place_card(hand_card) {
                    return Err(IllegalMove::PrematurelyEndedCombo { card_idx });
                }
            }
        }
        cards_won_this_turn |= cards_won;

        card_idx += 1;
    }

    // Draw cards until hand is full again
    let mut hand: Vec<Card> = hand.into_iter().collect();
    while hand.len() < 5 {
        match state.draw_pile.pop() {
            Some(card) => {
                hand.push(card);
            }
            None => {
                return Ok(TurnOutcome::GameEnded);
            }
        };
    }
    state.hand = hand.try_into().unwrap();
    state.cards_won |= cards_won_this_turn;
    Ok(TurnOutcome::Normal {
        cards_won_this_turn,
    })
}
