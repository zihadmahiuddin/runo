use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnoError {
    #[error("Not enough players")]
    NotEnoughPlayers,
    #[error("Too many players")]
    TooManyPlayers,
}

pub type Result<T, E = UnoError> = std::result::Result<T, E>;
