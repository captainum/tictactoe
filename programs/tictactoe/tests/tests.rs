use anchor_lang::prelude::{system_program, Pubkey};
use anchor_lang::{
    AccountDeserialize, AnchorDeserialize, Discriminator, InstructionData, ToAccountMetas,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use litesvm::types::TransactionMetadata;
use litesvm::LiteSVM;
use solana_sdk::account::ReadableAccount;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::Message;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use tictactoe::event::PlayerWins;
use tictactoe::state::Game;
use tictactoe::{accounts, instruction, ID as PROGRAM_ID};
use uuid::Uuid;

static PROGRAM_PATH: &str = "../../target/deploy/tictactoe.so";

fn read_game(svm: &LiteSVM, pda: &Pubkey) -> Game {
    let acc = svm.get_account(pda).unwrap();
    Game::try_deserialize(&mut acc.data().iter().as_slice()).unwrap()
}

fn setup() -> (LiteSVM, Keypair, Keypair, Keypair, [u8; 16], Pubkey) {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(PROGRAM_ID, PROGRAM_PATH).unwrap();

    let owner = Keypair::new();
    let player1 = Keypair::new();
    let player2 = Keypair::new();

    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&player1.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&player2.pubkey(), 10_000_000_000).unwrap();

    let uuid = Uuid::new_v4().into_bytes();
    let (pda, _) = Pubkey::find_program_address(
        &[b"ttt", owner.pubkey().as_ref(), uuid.as_ref()],
        &PROGRAM_ID,
    );

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts::Initialize {
            game: pda,
            owner: owner.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::Initialize {
            uuid,
            player1: player1.pubkey(),
            player2: player2.pubkey(),
        }
        .data(),
    };
    let tx = Transaction::new(
        &[&owner],
        Message::new(&[ix], Some(&owner.pubkey())),
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    (svm, owner, player1, player2, uuid, pda)
}

fn play(
    svm: &mut LiteSVM,
    owner: &Pubkey,
    pda: &Pubkey,
    uuid: [u8; 16],
    player: &Keypair,
    position: u8,
) -> TransactionMetadata {
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts::PlayGame {
            game: *pda,
            owner: *owner,
            player: player.pubkey(),
        }
        .to_account_metas(None),
        data: instruction::PlayGame {
            uuid,
            move_position: position,
        }
        .data(),
    };
    let tx = Transaction::new(
        &[player],
        Message::new(&[ix], Some(&player.pubkey())),
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap()
}

fn find_player_wins(logs: &[String]) -> Option<PlayerWins> {
    for log in logs {
        let Some(payload) = log.strip_prefix("Program data: ") else {
            continue;
        };
        let Ok(bytes) = BASE64.decode(payload.trim()) else {
            continue;
        };
        if bytes.len() < 8 || &bytes[..8] != PlayerWins::DISCRIMINATOR {
            continue;
        }
        if let Ok(ev) = PlayerWins::try_from_slice(&bytes[8..]) {
            return Some(ev);
        }
    }
    None
}

#[test]
fn test_game_initialization() {
    let (svm, _owner, _p1, _p2, _uuid, pda) = setup();

    let game = read_game(&svm, &pda);

    assert_eq!(game.current_player, 1);
    assert_eq!(game.grid, [0; 9]);
    assert_eq!(game.state, tictactoe::state::State::InProgress as u8);
}

#[test]
fn player1_wins_top_row() {
    let (mut svm, owner, p1, p2, uuid, pda) = setup();

    // P1 | P1 | P1   <- победа верхней строкой
    // P2 | P2 |  .
    //  . |  . |  .
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 1);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 4);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 2);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 5);
    let meta = play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 3);

    let game = read_game(&svm, &pda);
    assert_eq!(game.state, tictactoe::state::State::Player1Wins as u8);

    let event = find_player_wins(&meta.logs).expect("PlayerWins event was not emitted");
    assert_eq!(event.player, p1.pubkey());
}

#[test]
fn player2_wins_middle_row() {
    let (mut svm, owner, p1, p2, uuid, pda) = setup();

    // P1 | P1 |  .
    // P2 | P2 | P2   <- победа средней строкой
    //  . |  . | P1
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 1);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 4);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 2);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 5);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 9);
    let meta = play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 6);

    let game = read_game(&svm, &pda);
    assert_eq!(game.state, tictactoe::state::State::Player2Wins as u8);

    let event = find_player_wins(&meta.logs).expect("PlayerWins event was not emitted");
    assert_eq!(event.player, p2.pubkey());
}

#[test]
fn draw_full_board() {
    let (mut svm, owner, p1, p2, uuid, pda) = setup();

    // Полностью заполненное поле без выигрышных линий:
    // P1 | P2 | P1
    // P1 | P2 | P2
    // P2 | P1 | P1
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 1);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 2);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 3);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 5);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 4);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 6);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 8);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p2, 7);
    play(&mut svm, &owner.pubkey(), &pda, uuid, &p1, 9);

    let game = read_game(&svm, &pda);
    assert_eq!(game.state, tictactoe::state::State::Draw as u8);
}
