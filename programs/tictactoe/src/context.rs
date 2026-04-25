use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(uuid: [u8; 16])]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = owner,
        seeds = [b"ttt", owner.key().as_ref(), uuid.as_ref()],
        bump,
        space = 8 + Game::SIZE
    )]
    pub game: Account<'info, Game>,

    #[account(mut)]
    pub owner: Signer<'info>,

    system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(uuid: [u8; 16])]
pub struct PlayGame<'info> {
    #[account(mut, seeds = [b"ttt", owner.key().as_ref(), uuid.as_ref()], bump)]
    pub game: Account<'info, Game>,

    pub owner: SystemAccount<'info>,

    pub player: Signer<'info>,
}
