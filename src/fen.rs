use crate::bit_operations;
use crate::board::BoardData;

impl BoardData {
    fn to_fen(&self) -> String {
        unimplemented!()
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
