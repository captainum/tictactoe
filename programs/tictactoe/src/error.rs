use anchor_lang::prelude::*;

#[error_code]
pub enum PlayGameError {
    #[msg("Game is already finished")]
    GameFinished,

    #[msg("It's not your turn!")]
    InvalidPlayer,

    #[msg("Invalid move, off the grid")]
    OffTheGrid,

    #[msg("Position is already occupied")]
    PositionOccupied,
}
