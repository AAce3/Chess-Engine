pub const NOT_A_FILE: u64 = !0x101010101010101;
pub const NOT_H_FILE: u64 = !0x8080808080808080;

//generate a bitboard with a 1 at the given index
#[inline]
pub fn generate_from_index(index: u8) -> u64 {
    1 << index
}

// setting 1s and 0s
pub mod shifts {
    // shifting
    use crate::bit_operations::generate_from_index;
    use crate::bit_operations::NOT_A_FILE;
    use crate::bit_operations::NOT_H_FILE;
    use crate::board::Squares;

    #[inline]
    pub fn shift_n(bit_board: u64) -> u64 {
        bit_board << 8
    }

    #[inline]
    pub fn shift_s(bit_board: u64) -> u64 {
        bit_board >> 8
    }

    #[inline]
    pub fn shift_e(bit_board: u64) -> u64 {
        (bit_board << 1) & NOT_A_FILE
    }

    #[inline]
    pub fn shift_ne(bit_board: u64) -> u64 {
        (bit_board << 9) & NOT_A_FILE
    }

    #[inline]
    pub fn shift_se(bit_board: u64) -> u64 {
        (bit_board >> 7) & NOT_A_FILE
    }

    #[inline]
    pub fn shift_w(bit_board: u64) -> u64 {
        (bit_board >> 1) & NOT_H_FILE
    }

    #[inline]
    pub fn shift_sw(bit_board: u64) -> u64 {
        (bit_board >> 9) & NOT_H_FILE
    }

    #[inline]
    pub fn shift_nw(bit_board: u64) -> u64 {
        (bit_board << 7) & NOT_H_FILE
    }

    // bit-index manipulations
    #[inline]
    pub fn set_0_at_square(square: Squares, bit_board: &mut u64) {
        set_0_at_index(square as u8, bit_board);
    }

    #[inline]
    pub fn set_1_at_square(square: Squares, bit_board: &mut u64) {
        set_1_at_index(square as u8, bit_board);
    }

    #[inline]
    pub fn set_0_at_index(index: u8, bit_board: &mut u64) {
        *bit_board &= !generate_from_index(index)
    }

    #[inline]
    pub fn set_1_at_index(index: u8, bit_board: &mut u64) {
        *bit_board |= generate_from_index(index)
    }
}

// bitscan:

// get ls1b:
#[inline]
pub fn ls1b(bit_board: u64) -> u8 {
    assert_ne!(bit_board, 0);
    bit_board.trailing_zeros() as u8
}

#[inline]
pub fn pop_ls1b(bit_board: &mut u64) -> u8 {
    assert_ne!(*bit_board, 0);
    let cnt = bit_board.trailing_zeros() as u8;
    *bit_board &= *bit_board - 1;
    cnt
}

#[inline]
pub fn pop_count(bit_board: u64) -> u8 {
    bit_board.count_ones() as u8
}