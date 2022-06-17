#![allow(dead_code)]
#![allow(unused_variables)]

mod bit_operations;
mod board;
mod fen;
mod bitboard_gen; // generating bitboards
mod movegen;
mod action;

fn main() {

    let the_board = fen::from_fen("rnbqkbnr/ppppp1pp/8/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 1 2");
    println!("{}", the_board.to_fen())
}
