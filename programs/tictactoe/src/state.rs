use anchor_lang::prelude::*;

#[account]
pub struct Game {
    pub uuid: [u8; 16],
    pub grid: [u8; 9],
    pub players: [Pubkey; 2],
    pub current_player: u8,
    pub state: u8,
}

impl Game {
    pub const SIZE: usize = 16 + 9 + 2 * 32 + 1 + 1;
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum State {
    InProgress = 0,
    Player1Wins = 1,
    Player2Wins = 2,
    Draw = 3,
}
