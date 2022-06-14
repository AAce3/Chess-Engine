use core::panic;

use crate::bit_operations;
use crate::board::{BoardData};

impl BoardData {
    pub fn to_fen(&self) -> String {
        // main fen string
        let mut fen_list = Vec::new();
        // a row
        let mut so_far_string = String::new();
        let mut counter = 0;
        for i in 0..64 {
            if i % 8 == 0 {
                if counter != 0 {
                    so_far_string += &counter.to_string();
                }
                fen_list.push(so_far_string.clone());
                so_far_string.clear();
                counter = 0;
            }
            let pval = match self.mailbox[i] {
                0 => "K",
                1 => "Q",
                2 => "B",
                3 => "N",
                4 => "R",
                5 => "P",

                6 => "k",
                7 => "q",
                8 => "b",
                9 => "n",
                10 => "r",
                11 => "p",
                _ => {
                    counter += 1;
                    ""
                }
            };

            // there were some number of gaps, which culminated in a piece
            if counter != 0 && !pval.is_empty() {
                so_far_string += &counter.to_string();
                counter = 0
            }
            so_far_string += pval;
            
        }
        // since it never gets to 64 it will never get pushed
        fen_list.push(so_far_string);
       
        let mut val = fen_list.iter().rev().fold(fen_list[0].clone(), |a, b| a + "/" + b);
        // remove slashes at beginning and end
        val.pop();
        val.remove(0);

        let tomove = if self.is_white_move {"w"} else {"b"};
        let passant = match self.passant_square {
            Some(sqr) => idx_to_coordsquare(sqr),
            None => String::from("-"),
        };

        let castlemask = match self.castle_rights_mask {
            0b1111 => "KQkq",
            0b0000 => "-",
            0b1110 => "KQk",
            0b1101 => "KQq",
            0b1011 => "Kkq",
            0b0111 => "Qkq",
            0b1100 => "KQ",
            0b0011 => "kq",
            0b1001 => "Kq",
            0b0110 => "Qk",
            0b1010 => "Kk",
            0b0101 => "Qq",
            0b1000 => "K",
            0b0100 => "Q",
            0b0010 => "k",
            0b0001 => "q",

            _ => panic!("Invalid castle!"),
        };

        let move_counter = self.half_move_counter / 2;

        format!("{} {} {} {} {} {}", val, tomove, castlemask, passant, move_counter, self.half_move_counter)
    }

    //prints out a board for debugging
    pub fn to_boardstring(&self) -> String {
        let mailbox = self.mailbox;
        let mut to_return_str = String::new();
        let mut counter: u8 = 72;
        let mut file: u8 = 8;
        loop {
            if counter % 8 == 0 {
                if counter > 8 {
                    counter -= 16
                } else {
                    break;
                }
                to_return_str.push('\n');
                to_return_str.push(char::from_digit(file as u32, 10).unwrap());
                to_return_str.push(' ');
                file -= 1;
            }
            to_return_str.push(match_pval_to_char(mailbox[counter as usize]));
            to_return_str.push(' ');
            counter += 1;
        }

        let passantval: i32 = match self.passant_square {
            None => -1,
            Some(sqr) => sqr as i32,
        };
        let castle = self.castle_rights_mask;
        to_return_str
            + "\n  a b c d e f g h \nCastle Rights: "
            + &format!("{castle:4b}")
            + "\nHalf Move Counter: "
            + &self.half_move_counter.to_string()
            + "\nPassant Square: "
            + &passantval.to_string()
    }
}

pub fn from_fen(fen_string: &str) -> BoardData {
    let mut bit_boards: [u64; 14] = [0; 14];
    let fen_board: Vec<&str> = fen_string.split(' ').collect();
    let board = fen_board[0];
    let mut file = 0;
    let mut rank = 7;
    for symbol in board.chars() {
        if symbol == '/' {
            file = 0;
            rank -= 1;
        } else if symbol.is_numeric() {
            let val = char::to_digit(symbol, 10).unwrap();
            file += val;
        } else {
            let index: u8 = (rank as u8 * 8) + file as u8;
            let val = bit_operations::generate_from_index(index);
            bit_boards[piece_val_from_symbol(symbol) as usize] |=
                bit_operations::generate_from_index(index);
            file += 1;
        }
    }

    let mut base_white: u64 = 0;
    for i in bit_boards.iter().take(6) {
        base_white |= i;
    }

    let mut base_black: u64 = 0;
    for i in bit_boards.iter().take(12).skip(6) {
        base_black |= i;
    }

    bit_boards[12] = base_white;
    bit_boards[13] = base_black;

    let to_move = fen_board[1] == "w";

    let mut base_mask: u8 = 0;
    if fen_board[2].contains('K') {
        base_mask |= crate::board::WK;
    }
    if fen_board[2].contains('Q') {
        base_mask |= crate::board::WQ;
    }
    if fen_board[2].contains('k') {
        base_mask |= crate::board::BK;
    }
    if fen_board[2].contains('q') {
        base_mask |= crate::board::BQ;
    }

    let half_move_ctr = fen_board[5].parse().unwrap();

    let mut new_board = BoardData {
        is_white_move: to_move,
        bitboards: bit_boards,
        mailbox: [0; 64],
        passant_square: sqr_to_index(fen_board[3]),
        castle_rights_mask: base_mask,
        half_move_counter: half_move_ctr,
        prev_move_data: Vec::new(),
    };

    new_board.set_mailbox();
    new_board
}

fn piece_val_from_symbol(val: char) -> u8 {
    match val {
        'K' => 0,
        'Q' => 1,
        'B' => 2,
        'N' => 3,
        'R' => 4,
        'P' => 5,

        'k' => 6,
        'q' => 7,
        'b' => 8,
        'n' => 9,
        'r' => 10,
        'p' => 11,
        _ => panic!("Invalid FEN!"),
    }
}

fn match_pval_to_char(pval: u8) -> char {
    match pval {
        0 => '♚',
        1 => '♛',
        2 => '♝',
        3 => '♞',
        4 => '♜',
        5 => '♟',

        6 => '♔',
        7 => '♕',
        8 => '♗',
        9 => '♘',
        10 => '♖',
        11 => '♙',
        _ => '.',
    }
}

fn sqr_to_index(square: &str) -> Option<u8> {
    let coords: Vec<char> = square.chars().collect();
    let file = match coords[0] {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => return None,
    };

    let rank = char::to_digit(coords[1], 10).unwrap() - 1;
    Some((rank * 8 + file) as u8)
}

fn idx_to_coordsquare(idx: u8) -> String {
    let rank = (idx >> 3) + 1;
    let file = match idx & 7{
        0 => "a",
        1 => "b",
        2 => "c",
        3 => "d",
        4 => "e",
        5 => "f",
        6 => "g",
        7 => "h",
        _ => panic!("Not a valid idx!")
    };

   format!("{}{}", file, rank)
}