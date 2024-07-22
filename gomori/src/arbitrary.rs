use std::collections::{BTreeMap, BTreeSet};

use crate::{Card, CardToPlay, Field, Rank, Suit};

#[derive(Clone, Debug)]
pub struct PlayCardInput {
    // Nonempty
    pub fields: Vec<Field>,
    pub card_to_play: CardToPlay,
}

impl quickcheck::Arbitrary for PlayCardInput {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut already_played_cards = BTreeSet::<Card>::arbitrary(g);

        // The card to be played
        let card = Card::arbitrary(g);
        // Ensure the card does not exist twice
        already_played_cards.remove(&card);
        // Ensure that the list of already played cards is not empty
        // For this, we need a card that is distinct from the card to be played
        let other_card = loop {
            let c = Card::arbitrary(g);
            if c != card {
                break c;
            }
        };
        already_played_cards.insert(other_card);

        let mut cards_on_field = BTreeMap::new();
        for played_card in already_played_cards {
            let i = (u8::arbitrary(g) % 4) as i8 - 2;
            let j = (u8::arbitrary(g) % 4) as i8 - 2;
            cards_on_field
                .entry((i, j))
                .or_insert(BTreeSet::new())
                .insert(played_card);
        }

        let mut fields = Vec::with_capacity(cards_on_field.len());
        for ((i, j), mut cards) in cards_on_field {
            let top_card = if bool::arbitrary(g) {
                cards.pop_last()
            } else {
                None
            };
            fields.push(Field {
                i,
                j,
                top_card,
                hidden_cards: cards,
            });
        }
        fields.sort_by_key(|field| (field.i, field.j));

        let i = (u8::arbitrary(g) % 4) as i8 - 2;
        let j = (u8::arbitrary(g) % 4) as i8 - 2;
        let i_tgt = (u8::arbitrary(g) % 4) as i8 - 2;
        let j_tgt = (u8::arbitrary(g) % 4) as i8 - 2;
        let target_field_for_king_ability = Some((i_tgt, j_tgt));
        let card_to_play = CardToPlay {
            card,
            i,
            j,
            target_field_for_king_ability,
        };

        PlayCardInput {
            fields,
            card_to_play,
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Suit {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        *g.choose(&[Suit::Diamond, Suit::Heart, Suit::Spade, Suit::Club])
            .unwrap()
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Rank {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        *g.choose(&[
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ])
        .unwrap()
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Card {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            rank: Rank::arbitrary(g),
            suit: Suit::arbitrary(g),
        }
    }
}
