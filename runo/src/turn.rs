use crate::card::{Card, CardColor};

pub enum PlayAction {
    ColoredCard(Card),
    Wild(CardColor),
    WildDraw(CardColor),
}

pub enum TurnAction {
    Play(PlayAction),
    Draw,
    Callout,
    Uno,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TurnActionResult {
    Neutral,
    CardNotInHand,
    Skip,
    Reverse,
    SelfDraw,
    Draw,
    Wild,
    WildDraw,
    CalloutFailed,
    CalledOut(Vec<u64>),
    UnoFailed,
    UnoSuccessful,
}
