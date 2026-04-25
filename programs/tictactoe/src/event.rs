use anchor_lang::prelude::*;

#[event]
pub struct PlayerWins {
    pub player: Pubkey,
}
