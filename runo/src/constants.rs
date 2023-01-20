use strum::EnumCount;

use crate::card::CardColor;

pub(crate) const NUMBER_CARDS_PER_COLOR: &[u8] =
    &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9];
pub(crate) const SKIP_CARDS_PER_COLOR: u8 = 2;
pub(crate) const REVERSE_CARDS_PER_COLOR: u8 = 2;
pub(crate) const DRAW_CARDS_PER_COLOR: u8 = 2;

pub(crate) const NUMBER_CARDS_IN_DECK: u8 = (NUMBER_CARDS_PER_COLOR.len() * CardColor::COUNT) as u8;
pub(crate) const SKIP_CARDS_IN_DECK: u8 = SKIP_CARDS_PER_COLOR * CardColor::COUNT as u8;
pub(crate) const REVERSE_CARDS_IN_DECK: u8 = REVERSE_CARDS_PER_COLOR * CardColor::COUNT as u8;
pub(crate) const DRAW_CARDS_IN_DECK: u8 = DRAW_CARDS_PER_COLOR * CardColor::COUNT as u8;

pub(crate) const WILD_CARDS_IN_DECK: u8 = 4;
pub(crate) const WILD_DRAW_CARDS_IN_DECK: u8 = 4;

pub(crate) const TOTAL_CARDS_IN_DECK: u8 = NUMBER_CARDS_IN_DECK
    + SKIP_CARDS_IN_DECK
    + REVERSE_CARDS_IN_DECK
    + DRAW_CARDS_IN_DECK
    + WILD_CARDS_IN_DECK
    + WILD_DRAW_CARDS_IN_DECK;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_card_count_constants() {
        assert_eq!(NUMBER_CARDS_PER_COLOR.len(), 19);
        assert_eq!(NUMBER_CARDS_IN_DECK, 76);

        assert_eq!(SKIP_CARDS_IN_DECK, 8);

        assert_eq!(REVERSE_CARDS_IN_DECK, 8);

        assert_eq!(DRAW_CARDS_IN_DECK, 8);

        assert_eq!(TOTAL_CARDS_IN_DECK, 108);
    }
}
