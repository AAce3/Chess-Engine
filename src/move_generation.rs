use crate::bit_operations::shifts;
use crate::bit_operations::{self, generate_from_index, ls1b};
extern crate lazy_static;
use lazy_static::lazy_static;

/* ========================================
*   Precalculated attack tables are used    
|   for leaping pieces such as knights,     
*   kings, and pawn single-pushes. Pawn     
|   promotions and doublepushes will be     
*   generated on the fly.                   
   ======================================   */

lazy_static! {
    pub static ref KING_TABLES: Box<[u64; 64]> = Box::new(generate_king_moves());
    pub static ref KNIGHT_TABLES: Box<[u64; 64]> = Box::new(generate_knight_moves());
    pub static ref PAWN_PUSH_TABLES: Box<[[u64; 64]; 2]> = Box::new(generate_pawnpushes());
    pub static ref PAWN_CAPTURE_TABLES: Box<[[u64; 64]; 2]> = Box::new(generate_pawncaptures());
}

const EDGES: u64 = 0xff818181818181ff;

#[allow(clippy::needless_range_loop)]
pub fn generate_king_moves() -> [u64; 64] {
    let mut store_val = [0; 64];
    for i in 0..63 {
        store_val[i] = king_moves_for_square(i as u8);
    }
    store_val
}

#[allow(clippy::needless_range_loop)]
pub fn generate_knight_moves() -> [u64; 64] {
    let mut store_val = [0; 64];
    for i in 0..63 {
        store_val[i] = knight_moves_for_square(i as u8);
    }
    store_val
}

#[allow(clippy::needless_range_loop)]
pub fn generate_pawnpushes() -> [[u64; 64]; 2] {
    let mut store_val = [[0; 64]; 2];
    for i in 0..64 {
        store_val[0][i] = pawn_moves_for_square(i as u8, true);
        store_val[1][i] = pawn_moves_for_square(i as u8, false);
    }
    store_val
}

pub fn knight_moves_for_square(index: u8) -> u64 {
    let knight_board = bit_operations::generate_from_index(index);
    shifts::shift_w(shifts::shift_nw(knight_board))
        | shifts::shift_w(shifts::shift_sw(knight_board))
        | shifts::shift_e(shifts::shift_ne(knight_board))
        | shifts::shift_e(shifts::shift_se(knight_board))
        | shifts::shift_n(shifts::shift_ne(knight_board))
        | shifts::shift_n(shifts::shift_nw(knight_board))
        | shifts::shift_s(shifts::shift_se(knight_board))
        | shifts::shift_s(shifts::shift_sw(knight_board))
}

pub fn king_moves_for_square(index: u8) -> u64 {
    let king_board = bit_operations::generate_from_index(index);
    shifts::shift_n(king_board)
        | shifts::shift_ne(king_board)
        | shifts::shift_e(king_board)
        | shifts::shift_se(king_board)
        | shifts::shift_s(king_board)
        | shifts::shift_sw(king_board)
        | shifts::shift_w(king_board)
        | shifts::shift_nw(king_board)
}

// Single-push pawn moves
pub fn pawn_moves_for_square(index: u8, color: bool) -> u64 {
    let pawn_board = bit_operations::generate_from_index(index);
    if color {
        // white
        shifts::shift_n(pawn_board) & !0xff00000000000000
    } else {
        // black
        shifts::shift_s(pawn_board) & !0xff
    }
}

pub fn generate_pawncaptures() -> [[u64; 64]; 2] {
    let mut store_val = [[0; 64]; 2];
    for i in 0..64 {
        store_val[0][i] = pawn_captures_for_square(i as u8, true);
        store_val[1][i] = pawn_captures_for_square(i as u8, false);
    }
    store_val
}

pub fn pawn_captures_for_square(index: u8, color: bool) -> u64 {
    let pawn_board = bit_operations::generate_from_index(index);
    if color {
        (shifts::shift_nw(pawn_board) | shifts::shift_ne(pawn_board)) & !0xff00000000000000
    } else {
        (shifts::shift_sw(pawn_board) | shifts::shift_se(pawn_board)) & !0xff
    }
}


/* ========================================
*   "plain" magic bitboards are used for    
|   precalculated slider attack tables.     
*   I took inspiration from the chess       
|   programming wiki, as well as this       
*   stackoverflow post:                     
|   https://stackoverflow.com/questions     
*   /16925204/sliding-move-generation       
|   -using-magic-bitboard                   
*   My magics were borrowed from Code       
|   Monkey King's "Didactic" engine, though 
*   it is fairly trivial to generate your   
|   own.                                    
   ======================================   */

static RMAGICS: [u64; 64] = [
    0x0080001020400080,
    0x0040001000200040,
    0x0080081000200080,
    0x0080040800100080,
    0x0080020400080080,
    0x0080010200040080,
    0x0080008001000200,
    0x0080002040800100,
    0x0000800020400080,
    0x0000400020005000,
    0x0000801000200080,
    0x0000800800100080,
    0x0000800400080080,
    0x0000800200040080,
    0x0000800100020080,
    0x0000800040800100,
    0x0000208000400080,
    0x0000404000201000,
    0x0000808010002000,
    0x0000808008001000,
    0x0000808004000800,
    0x0000808002000400,
    0x0000010100020004,
    0x0000020000408104,
    0x0000208080004000,
    0x0000200040005000,
    0x0000100080200080,
    0x0000080080100080,
    0x0000040080080080,
    0x0000020080040080,
    0x0000010080800200,
    0x0000800080004100,
    0x0000204000800080,
    0x0000200040401000,
    0x0000100080802000,
    0x0000080080801000,
    0x0000040080800800,
    0x0000020080800400,
    0x0000020001010004,
    0x0000800040800100,
    0x0000204000808000,
    0x0000200040008080,
    0x0000100020008080,
    0x0000080010008080,
    0x0000040008008080,
    0x0000020004008080,
    0x0000010002008080,
    0x0000004081020004,
    0x0000204000800080,
    0x0000200040008080,
    0x0000100020008080,
    0x0000080010008080,
    0x0000040008008080,
    0x0000020004008080,
    0x0000800100020080,
    0x0000800041000080,
    0x00FFFCDDFCED714A,
    0x007FFCDDFCED714A,
    0x003FFFCDFFD88096,
    0x0000040810002101,
    0x0001000204080011,
    0x0001000204000801,
    0x0001000082000401,
    0x0001FFFAABFAD1A2,
];

static BMAGICS: [u64; 64] = [
    0x0002020202020200,
    0x0002020202020000,
    0x0004010202000000,
    0x0004040080000000,
    0x0001104000000000,
    0x0000821040000000,
    0x0000410410400000,
    0x0000104104104000,
    0x0000040404040400,
    0x0000020202020200,
    0x0000040102020000,
    0x0000040400800000,
    0x0000011040000000,
    0x0000008210400000,
    0x0000004104104000,
    0x0000002082082000,
    0x0004000808080800,
    0x0002000404040400,
    0x0001000202020200,
    0x0000800802004000,
    0x0000800400A00000,
    0x0000200100884000,
    0x0000400082082000,
    0x0000200041041000,
    0x0002080010101000,
    0x0001040008080800,
    0x0000208004010400,
    0x0000404004010200,
    0x0000840000802000,
    0x0000404002011000,
    0x0000808001041000,
    0x0000404000820800,
    0x0001041000202000,
    0x0000820800101000,
    0x0000104400080800,
    0x0000020080080080,
    0x0000404040040100,
    0x0000808100020100,
    0x0001010100020800,
    0x0000808080010400,
    0x0000820820004000,
    0x0000410410002000,
    0x0000082088001000,
    0x0000002011000800,
    0x0000080100400400,
    0x0001010101000200,
    0x0002020202000400,
    0x0001010101000200,
    0x0000410410400000,
    0x0000208208200000,
    0x0000002084100000,
    0x0000000020880000,
    0x0000001002020000,
    0x0000040408020000,
    0x0004040404040000,
    0x0002020202020000,
    0x0000104104104000,
    0x0000002082082000,
    0x0000000020841000,
    0x0000000000208800,
    0x0000000010020200,
    0x0000000404080200,
    0x0000040404040400,
    0x0002020202020200,
];

lazy_static! {
    pub static ref ROOK_MASKS: [u64; 64] = generate_rooktables();
    pub static ref BISHOP_MASKS: [u64; 64] = generate_bishoptables();

    pub static ref BISHOP_TABLES: Box<[[u64; 512]; 64]> = generate_all_blookups();
    pub static ref ROOK_TABLES: Box<[[u64; 4096]; 64]> = generate_all_rlookups();
}

/* ========================================
*   rook_attacks and bishop_attacks
|   are the methods to generate an attack
*   bitboard for sliding pieces. They use
|   magic bitboards, so changing RMAGICS
*   or BMAGICS will make the method not 
|   work.                
   ========================================   */

pub fn rook_attacks(occupancy: u64, square: u8) -> u64{
    let mask = generate_blockermask(square,true);
    let idx = transform(mask & occupancy, RMAGICS[square as usize], mask.count_ones());
    ROOK_TABLES[square as usize][idx as usize]
}

pub fn bishop_attacks(occupancy: u64, square: u8) -> u64 {
    let mask = generate_blockermask(square,false);
    let idx = transform(mask & occupancy, BMAGICS[square as usize], mask.count_ones());
    BISHOP_TABLES[square as usize][idx as usize]
}

pub fn queen_attacks(occupancy: u64, square: u8) -> u64{
    bishop_attacks(occupancy, square) | rook_attacks(occupancy, square)
}
 
#[allow(clippy::needless_range_loop)]
pub fn generate_rooktables() -> [u64; 64] {
    let mut arr = [0; 64];
    for i in 0..64 {
        arr[i] = generate_blockermask(i as u8, true) & !rook_endpoints(i as u8);
    }
    arr
}

#[allow(clippy::needless_range_loop)]
pub fn generate_bishoptables() -> [u64; 64] {
    let mut arr = [0; 64];
    for i in 0..64 {
        arr[i] = generate_blockermask(i as u8, false) & !EDGES;
    }
    arr
}

pub fn generate_all_blookups() -> Box<[[u64; 512]; 64]> {
    let mut vals = vec![[0; 512]; 64];
    for i in 0..64 {
        let mask = BISHOP_MASKS[i as usize];
        let permutations = 1 << mask.count_ones();
        for j in 0..permutations {
            let blockerboard = generate_permutation(mask, j);
            let key = transform(blockerboard, BMAGICS[i], mask.count_ones());
            vals[i as usize][key as usize] = to_moveboard(blockerboard, i as u8, false);
        }
    }

    let boxedslice = vals.into_boxed_slice();
    match boxedslice.try_into() {
        Ok(val) => val,
        Err(e) => panic!("oh no"),
    }
}

pub fn generate_all_bperms(square: u8) -> [u64; 512] {
    let mask = BISHOP_MASKS[square as usize];
    let permutations = 1 << mask.count_ones();
    let mut arr = [0; 512];
    for i in 0..permutations {
        arr[i as usize] = generate_permutation(mask, i);
    }
    arr
}

pub fn generate_all_rlookups() -> Box<[[u64; 4096]; 64]> {
    let mut vals = vec![[0; 4096]; 64];
    for i in 0..64 {
        let mask = ROOK_MASKS[i as usize];
        let permutations = 1 << mask.count_ones();
        for j in 0..permutations {
            let blockerboard = generate_permutation(mask, j);
            let key = transform(blockerboard, RMAGICS[i], mask.count_ones());
            vals[i as usize][key as usize] = to_moveboard(blockerboard, i as u8, true);
        }
    }

    let boxedslice = vals.into_boxed_slice();
    match boxedslice.try_into() {
        Ok(val) => val,
        Err(e) => panic!("oh no"),
    }
}

pub fn generate_all_rperms(square: u8) -> [u64; 4096] {
    let mask = ROOK_MASKS[square as usize];
    let permutations = 1 << mask.count_ones();
    let mut arr = [0; 4096];
    for i in 0..permutations {
        arr[i as usize] = generate_permutation(mask, i);
    }
    arr
}

pub fn generate_permutation(mask: u64, permutation: u16) -> u64 {
    let mut mask = mask;

    let mut blockers = 0u64;
    let mut iteration = permutation;
    while iteration != 0 {
        if (iteration & 1) != 0 {
            blockers |= generate_from_index(ls1b(mask));
        }
        iteration >>= 1;
        mask &= mask - 1;
    }

    blockers
}

pub fn transform(occupancy: u64, magic: u64, bits: u32) -> u32 {
    ((occupancy * magic) >> (64 - bits)) as u32
}

pub fn rook_endpoints(idx: u8) -> u64 {
    let x_val = idx & (7);
    let y_val = idx >> 3;
    bit_operations::generate_from_index(x_val)
        | bit_operations::generate_from_index(x_val + 56)
        | bit_operations::generate_from_index(y_val * 8)
        | bit_operations::generate_from_index((y_val + 1) * 8 - 1)
}

// given a blocker board, generate a move board
pub fn to_moveboard(blocker_board: u64, square: u8, orientation: bool) -> u64 {
    let base_val: u64 = 0;
    let board = bit_operations::generate_from_index(square);
    let not_blocker = !blocker_board;

    let loop_until_end = |f: Box<dyn Fn(u64) -> u64>| {
        let mut base = 0;
        let mut tempboard = board;
        loop {
            base |= f(tempboard);
            tempboard = f(tempboard);
            if tempboard & not_blocker == 0 {
                break;
            }
        }
        base
    };

    if orientation {
        // orthogonal
        loop_until_end(Box::new(shifts::shift_e))
            | loop_until_end(Box::new(shifts::shift_n))
            | loop_until_end(Box::new(shifts::shift_s))
            | loop_until_end(Box::new(shifts::shift_w))
    } else {
        loop_until_end(Box::new(shifts::shift_ne))
            | loop_until_end(Box::new(shifts::shift_nw))
            | loop_until_end(Box::new(shifts::shift_se))
            | loop_until_end(Box::new(shifts::shift_sw))
    }
}

pub fn generate_blockermask(square: u8, orientation: bool) -> u64 {
    if orientation {
        // is a rook
        let file = square & 7;
        let rank = square >> 3;
        ((0xff << (8 * rank)) ^ (0x101010101010101 << file)) & !rook_endpoints(square)
    } else {
        to_moveboard(0, square, false) & (!EDGES)
    }
}