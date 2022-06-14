#![allow(dead_code)]
#![allow(unused_variables)]




mod bit_operations;
mod board;
mod fen;
mod move_generation; // generating moves

fn main() {

    let the_board = fen::from_fen("rnbqkbnr/ppppp1pp/8/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 2");
    println!("{}", the_board.to_boardstring())
}
