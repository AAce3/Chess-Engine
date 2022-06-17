use crate::action::StateData;
use crate::bit_operations::generate_from_index;
use std::u8;
extern crate lazy_static;
use lazy_static::lazy_static;

pub const WK: u8 = 0b1000;
pub const WQ: u8 = 0b0100;
pub const BK: u8 = 0b0010;
pub const BQ: u8 = 0b0001;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum Squares {
    a1,
    b1,
    c1,
    d1,
    e1,
    f1,
    g1,
    h1, //  0 ..  7
    a2,
    b2,
    c2,
    d2,
    e2,
    f2,
    g2,
    h2, //  8 .. 15
    a3,
    b3,
    c3,
    d3,
    e3,
    f3,
    g3,
    h3, // 16 .. 23
    a4,
    b4,
    c4,
    d4,
    e4,
    f4,
    g4,
    h4, // 24 .. 31
    a5,
    b5,
    c5,
    d5,
    e5,
    f5,
    g5,
    h5, // 32 .. 39
    a6,
    b6,
    c6,
    d6,
    e6,
    f6,
    g6,
    h6, // 40 .. 47
    a7,
    b7,
    c7,
    d7,
    e7,
    f7,
    g7,
    h7, // 48 .. 55
    a8,
    b8,
    c8,
    d8,
    e8,
    f8,
    g8,
    h8, // 56 .. 63
}

pub mod pieces {
    //represents index of bitboard
    //DO NOT TRY TO ACCESS NONE VALUE OF BITBOARD

    pub const NOPIECE: u8 = 14;

    pub const WKING: u8 = 0;
    pub const WQUEEN: u8 = 1;
    pub const WBISHOP: u8 = 2;
    pub const WKNIGHT: u8 = 3;
    pub const WROOK: u8 = 4;
    pub const WPAWN: u8 = 5;

    pub const BKING: u8 = 6;
    pub const BQUEEN: u8 = 7;
    pub const BBISHOP: u8 = 8;
    pub const BKNIGHT: u8 = 9;
    pub const BROOK: u8 = 10;
    pub const BPAWN: u8 = 11;

    pub const ALLWHITE: u8 = 12;
    pub const ALLBLACK: u8 = 13;
}

/* ========================================
*   Board representation is an array of
|   "Bitboards," which you can read about
*   on the chessprogramming wiki.
|   It also contains a "redundant" mailbox
*   for utility.
|   Passant square, castle mask, and half
*   move counter are self explanatory.
|   prev_mov_data is a stack of previous
*   board-information, used to undo moves.
   ======================================   */

#[derive(Debug)]
pub struct BoardData {
    pub to_move: bool, // true means white, false means black
    pub bitboards: [u64; 14],
    pub mailbox: [u8; 64],
    pub passant_square: Option<u8>,
    pub castle_rights_mask: u8,
    pub half_move_counter: u8, // is reset when a pawn moves or a capture takes place
    pub prev_states: Vec<StateData>,
    pub zobrist_key: u64,
}

impl BoardData {
    pub fn generate_zobristkey(&mut self) {
        let mut key = 0;
        for (square, piece) in self.mailbox.iter().enumerate() {
            if *piece != pieces::NOPIECE {
                key ^= ZOBRIST_TABLES.piecesquares[*piece as usize][square];
            }
        }

        match self.passant_square {
            None => (),
            Some(sqr) => {
                key ^= ZOBRIST_TABLES.passant_square[(sqr & 7) /* modulo of the key gets the file */ as usize];
            }
        }

        key ^= ZOBRIST_TABLES.castling_rights[self.castle_rights_mask as usize];

        key ^= if self.to_move {
            ZOBRIST_TABLES.to_move
        } else {
            0
        };

        self.zobrist_key = key;
    }

    /* ========================================
    *   set_mailbox is a useful method for
    |   debugging, as it synchronizes the
    *   mailbox with the bitboard.
       ======================================   */

    //sets mailbox from bitboards
    pub fn set_mailbox(&mut self) {
        let mut counter: usize = 0;
        let mut vals = [0b1111; 64];
        while counter < 64 {
            vals[counter] = self.which_piece(counter);
            counter += 1;
        }
        self.mailbox = vals;
    }

    // gets which piece at an index.
    // helper method for set_mailbox
    fn which_piece(&self, index: usize) -> u8 {
        let bb_val = generate_from_index(index as u8);
        for i in 0..12 {
            if self.bitboards[i] & bb_val > 0 {
                return i as u8;
            }
        }
        pieces::NOPIECE
    }
}

pub struct Zobrist {
    pub piecesquares: Box<[[u64; 12]; 64]>, // piece-square
    pub passant_square: Box<[u64; 8]>, // a random number for passant square. To convert from the passant number to the file, just take modulo
    // alternative to mod 8: (num) & 0b111 = num & 7
    pub castling_rights: Box<[u64; 16]>,
    pub to_move: u64, // is xor'd if it is white to move
}

lazy_static! {
    pub static ref ZOBRIST_TABLES: Zobrist = generate_zobrist();
}

fn generate_zobrist() -> Zobrist {
    let mut newzobrist = Zobrist {
        piecesquares: Box::new([[0; 12]; 64]),
        passant_square: Box::new([0; 8]),
        castling_rights: Box::new([0; 16]),
        to_move: 0,
    };

    // initialize piece-square tables
    for i in 0..64 {
        for j in 0..12 {
            newzobrist.piecesquares[i][j] = random_u64();
        }
    }
    for i in 0..8 {
        newzobrist.passant_square[i] = random_u64();
    }
    for i in 0..16 {
        newzobrist.castling_rights[i] = random_u64();
    }
    newzobrist.to_move = random_u64();
    newzobrist
}

static mut SEED: u32 = 1804289383;

fn random_u64() -> u64 {
    let (a, b, c, d): (u64, u64, u64, u64);
    unsafe {
        a = (xorshift_u32() & 0xFFFF) as u64;
        b = (xorshift_u32() & 0xFFFF) as u64;
        c = (xorshift_u32() & 0xFFFF) as u64;
        d = (xorshift_u32() & 0xFFFF) as u64;
    }
    a | (b << 16) | (c << 32) | (d << 48)
}

unsafe fn xorshift_u32() -> u32 {
    let mut num = SEED;
    num ^= num << 13;
    num ^= num >> 17;
    num ^= num << 5;
    SEED = num;
    num
}
