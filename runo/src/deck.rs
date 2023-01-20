use rand::{seq::SliceRandom, thread_rng};
use strum::IntoEnumIterator;

use crate::{
    card::{Card, CardColor, ColoredCard},
    constants::*,
};

#[derive(Debug)]
pub struct Deck(pub(crate) Vec<Card>);

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(TOTAL_CARDS_IN_DECK.into());

        // Colored Cards
        for color in CardColor::iter() {
            // Skip Cards
            for _ in 0..SKIP_CARDS_PER_COLOR {
                cards.push(Card::Colored(color.clone(), ColoredCard::Skip));
            }

            // Reverse Cards
            for _ in 0..REVERSE_CARDS_PER_COLOR {
                cards.push(Card::Colored(color.clone(), ColoredCard::Reverse));
            }

            // Draw Cards
            for _ in 0..DRAW_CARDS_PER_COLOR {
                cards.push(Card::Colored(color.clone(), ColoredCard::Draw));
            }

            // Number Cards
            for number in NUMBER_CARDS_PER_COLOR {
                cards.push(Card::Colored(color.clone(), ColoredCard::Number(*number)));
            }
        }

        for _ in 0..WILD_CARDS_IN_DECK {
            cards.push(Card::Wild);
        }

        for _ in 0..WILD_DRAW_CARDS_IN_DECK {
            cards.push(Card::WildDraw);
        }

        Self(cards)
    }

    pub(crate) fn shuffle(&mut self) -> () {
        let mut rng = thread_rng();
        self.0.shuffle(&mut rng);
    }

    pub(crate) fn draw_cards(&mut self, count: usize) -> Vec<Card> {
        self.0.drain(0..count).collect::<Vec<_>>()
    }

    pub(crate) fn draw_colored_card(&mut self) -> Option<Card> {
        self.0
            .iter()
            .position(|x| matches!(x, Card::Colored(_, _)))
            .map(|pos| self.0.remove(pos))
    }

    pub(crate) fn cards_count(&self) -> usize {
        return self.0.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_card_count_new_deck() {
        assert_eq!(Deck::new().cards_count(), TOTAL_CARDS_IN_DECK as usize);
    }
}
