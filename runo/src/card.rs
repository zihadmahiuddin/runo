use core::fmt;
use std::fmt::Display;

use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, EnumString};

#[derive(Clone, Debug, Display, EnumString, EnumCountMacro, EnumIter, PartialEq, Eq)]
pub enum CardColor {
    Red,
    Green,
    Blue,
    Yellow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColoredCard {
    Number(u8),
    Skip,
    Reverse,
    Draw,
}

impl ColoredCard {
    pub fn into_played_card(self, color: CardColor) -> PlayedCard {
        match self {
            ColoredCard::Number(number) => PlayedCard::Colored(color, ColoredCard::Number(number)),
            ColoredCard::Skip => PlayedCard::Colored(color, ColoredCard::Skip),
            ColoredCard::Reverse => PlayedCard::Colored(color, ColoredCard::Reverse),
            ColoredCard::Draw => PlayedCard::Colored(color, ColoredCard::Draw),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Card {
    Colored(CardColor, ColoredCard),
    Wild,
    WildDraw,
}

#[derive(Clone, Debug)]
pub enum PlayedCard {
    Colored(CardColor, ColoredCard),
    Wild(CardColor),
    WildDraw(CardColor),
}

impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Card::Colored(color, card) => {
                write!(f, "{} {}", color, {
                    match card {
                        ColoredCard::Number(number) => number.to_string(),
                        ColoredCard::Skip => "Skip".to_string(),
                        ColoredCard::Reverse => "Reverse".to_string(),
                        ColoredCard::Draw => "Draw".to_string(),
                    }
                })
            }
            Card::Wild => write!(f, "Wild"),
            Card::WildDraw => write!(f, "Wild Draw"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_correct_string_for_number_card() {
        let red_3 = Card::Colored(CardColor::Red, ColoredCard::Number(3));
        assert_eq!(red_3.to_string(), "Red 3");

        let yellow_5 = Card::Colored(CardColor::Yellow, ColoredCard::Number(5));
        assert_eq!(yellow_5.to_string(), "Yellow 5");

        let blue_9 = Card::Colored(CardColor::Blue, ColoredCard::Number(9));
        assert_eq!(blue_9.to_string(), "Blue 9");
    }

    #[test]
    fn return_correct_string_for_skip_card() {
        let red_skip = Card::Colored(CardColor::Red, ColoredCard::Skip);
        assert_eq!(red_skip.to_string(), "Red Skip");

        let yellow_skip = Card::Colored(CardColor::Yellow, ColoredCard::Skip);
        assert_eq!(yellow_skip.to_string(), "Yellow Skip");

        let blue_skip = Card::Colored(CardColor::Blue, ColoredCard::Skip);
        assert_eq!(blue_skip.to_string(), "Blue Skip");
    }

    #[test]
    fn return_correct_string_for_reverse_card() {
        let red_reverse = Card::Colored(CardColor::Red, ColoredCard::Reverse);
        assert_eq!(red_reverse.to_string(), "Red Reverse");

        let yellow_reverse = Card::Colored(CardColor::Yellow, ColoredCard::Reverse);
        assert_eq!(yellow_reverse.to_string(), "Yellow Reverse");

        let blue_reverse = Card::Colored(CardColor::Blue, ColoredCard::Reverse);
        assert_eq!(blue_reverse.to_string(), "Blue Reverse");
    }

    #[test]
    fn return_correct_string_for_draw_card() {
        let red_draw = Card::Colored(CardColor::Red, ColoredCard::Draw);
        assert_eq!(red_draw.to_string(), "Red Draw");

        let yellow_draw = Card::Colored(CardColor::Yellow, ColoredCard::Draw);
        assert_eq!(yellow_draw.to_string(), "Yellow Draw");

        let blue_draw = Card::Colored(CardColor::Blue, ColoredCard::Draw);
        assert_eq!(blue_draw.to_string(), "Blue Draw");
    }

    #[test]
    fn return_correct_string_for_wild_card() {
        let wild = Card::Wild;
        assert_eq!(wild.to_string(), "Wild");
    }

    #[test]
    fn return_correct_string_for_wild_draw_card() {
        let wild_draw = Card::WildDraw;
        assert_eq!(wild_draw.to_string(), "Wild Draw");
    }
}
