#![allow(dead_code)]
#![allow(unused_variables)]

mod bit_operations;
mod board;
mod fen;
mod bitboard_gen; // generating bitboards
mod movegen;
mod action;
use action::actions;
use action::Action;
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "2");
    let test: Action = actions::new(36, 45, actions::PASSANT, 0);
    let mut the_board = fen::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    println!("{}", the_board.to_boardstring());
    println!("zbkey: {}", the_board.zobrist_key);

    
    the_board.make_move(test);
    the_board.set_mailbox();
    println!("{}", the_board.to_boardstring());
    println!("zbkey: {}", the_board.zobrist_key);

    the_board.undo_move(test);

    the_board.set_mailbox();
    the_board.generate_zobristkey();
    println!("{}", the_board.to_boardstring());
    println!("zbkey: {}", the_board.zobrist_key)
    
}
