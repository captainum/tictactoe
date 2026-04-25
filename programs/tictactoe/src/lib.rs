#![allow(clippy::diverging_sub_expression)]

pub mod context;
mod error;
pub mod event;
pub mod state;

use context::*;
use error::*;
use event::*;
use state::*;

use anchor_lang::prelude::*;

declare_id!("7e9Nbo8sbBqacV5AULLQapKYuzCtX37QUShPovB3qbwi");

#[program]
pub mod tictactoe {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        uuid: [u8; 16],
        player1: Pubkey,
        player2: Pubkey,
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;

        game.uuid = uuid;
        game.players = [player1, player2];
        game.current_player = 1;
        game.state = State::InProgress as u8;

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn play_game(ctx: Context<PlayGame>, uuid: [u8; 16], move_position: u8) -> Result<()> {
        let mut move_position = move_position;

        let game = &mut ctx.accounts.game;
        let current_player = game.players[game.current_player as usize - 1];

        require_eq!(
            game.state,
            State::InProgress as u8,
            PlayGameError::GameFinished
        );
        require_eq!(
            current_player,
            ctx.accounts.player.key(),
            PlayGameError::InvalidPlayer
        );
        require!(
            move_position > 0 && move_position < 10,
            PlayGameError::OffTheGrid
        );
        require_eq!(
            game.grid[move_position as usize - 1],
            0,
            PlayGameError::PositionOccupied
        );

        move_position -= 1;

        game.grid[move_position as usize] = game.current_player;

        let state = check_win(&game.grid, game.current_player);

        game.current_player = game.current_player % 2 + 1;

        match state {
            State::Player1Wins => {
                emit!(PlayerWins {
                    player: current_player
                });

                msg!("Player 1 wins!");
            }
            State::Player2Wins => {
                emit!(PlayerWins {
                    player: current_player
                });

                msg!("Player 2 wins!");
            }
            State::Draw => {
                msg!("Draw!");
            }
            _ => {}
        }

        game.state = state as u8;

        Ok(())
    }
}

pub fn check_win(grid: &[u8; 9], player: u8) -> State {
    if (grid[0] == player && grid[1] == player && grid[2] == player)
        || (grid[0] == player && grid[3] == player && grid[6] == player)
        || (grid[6] == player && grid[7] == player && grid[8] == player)
        || (grid[2] == player && grid[5] == player && grid[8] == player)
        || (grid[0] == player && grid[4] == player && grid[8] == player)
        || (grid[2] == player && grid[4] == player && grid[6] == player)
        || (grid[1] == player && grid[4] == player && grid[7] == player)
        || (grid[3] == player && grid[4] == player && grid[5] == player)
    {
        return if player == 1 {
            State::Player1Wins
        } else {
            State::Player2Wins
        };
    }

    if grid.contains(&0) {
        State::InProgress
    } else {
        State::Draw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_state(actual: State, expected: State) {
        assert_eq!(actual, expected);
    }

    #[test]
    fn player1_wins_each_row() {
        assert_state(
            check_win(&[1, 1, 1, 2, 2, 0, 0, 0, 0], 1),
            State::Player1Wins,
        );
        assert_state(
            check_win(&[2, 2, 0, 1, 1, 1, 0, 0, 0], 1),
            State::Player1Wins,
        );
        assert_state(
            check_win(&[2, 0, 0, 0, 2, 0, 1, 1, 1], 1),
            State::Player1Wins,
        );
    }

    #[test]
    fn player1_wins_each_column() {
        assert_state(
            check_win(&[1, 2, 2, 1, 0, 0, 1, 0, 0], 1),
            State::Player1Wins,
        );
        assert_state(
            check_win(&[2, 1, 0, 0, 1, 2, 0, 1, 0], 1),
            State::Player1Wins,
        );
        assert_state(
            check_win(&[2, 0, 1, 0, 2, 1, 0, 0, 1], 1),
            State::Player1Wins,
        );
    }

    #[test]
    fn player1_wins_both_diagonals() {
        assert_state(
            check_win(&[1, 2, 0, 2, 1, 0, 0, 0, 1], 1),
            State::Player1Wins,
        );
        assert_state(
            check_win(&[0, 2, 1, 0, 1, 2, 1, 0, 0], 1),
            State::Player1Wins,
        );
    }

    #[test]
    fn player2_wins_returns_player2_variant() {
        assert_state(
            check_win(&[2, 2, 2, 1, 1, 0, 0, 0, 0], 2),
            State::Player2Wins,
        );
    }

    #[test]
    fn full_grid_without_winner_is_draw() {
        let grid = [1, 2, 1, 1, 2, 1, 2, 1, 2];
        assert_state(check_win(&grid, 1), State::Draw);
        assert_state(check_win(&grid, 2), State::Draw);
    }

    #[test]
    fn partial_grid_without_winner_is_in_progress() {
        let grid = [1, 2, 0, 0, 1, 0, 0, 0, 2];
        assert_state(check_win(&grid, 1), State::InProgress);
        assert_state(check_win(&grid, 2), State::InProgress);
    }
}
