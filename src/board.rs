use core::panic;
use std::u8;

use crate::bit_operations::generate_from_index;
use crate::bit_operations::shifts;
use crate::bit_operations::shifts::set_0_at_square;
use crate::bit_operations::shifts::set_1_at_index;
use crate::bit_operations::shifts::set_1_at_square;

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
    pub is_white_move: bool,
    pub bitboards: [u64; 14],
    pub mailbox: [u8; 64],
    pub passant_square: Option<u8>,
    pub castle_rights_mask: u8,
    pub half_move_counter: u8,
    pub prev_move_data: Vec<UndoData>, 
}


impl BoardData {
    pub fn make_move(&mut self, action: &Move) {
        self.passant_square = None;
        match action.tag {
            // create UndoData
            // modify the bitboard
            // modify the mailbox
            MoveTag::Normal => self.make_normal(action),
            MoveTag::Castle => self.make_castle(action),
            MoveTag::Promotion => self.make_promotion(action),
            MoveTag::EPassant => self.make_passant(action),
        }
    }

    #[inline]
    pub fn make_normal(&mut self, action: &Move) {
        {
            let total_bb = if self.is_white_move { 12 } else { 13 };
            shifts::set_0_at_square(action.move_from, &mut self.bitboards[total_bb]);
            shifts::set_1_at_square(action.move_to, &mut self.bitboards[total_bb]);
        }

        let moving_piece = self.mailbox[action.move_from as usize];

        {
            let moving_piece_bitboard = &mut self.bitboards[moving_piece as usize];
            shifts::set_0_at_square(action.move_from, moving_piece_bitboard);
            shifts::set_1_at_square(action.move_to, moving_piece_bitboard);
        }
        let possible_pcapture: u8;
        {
            let piece_at_moveto = self.mailbox[action.move_to as usize];
            possible_pcapture = piece_at_moveto;
            if piece_at_moveto != pieces::NOPIECE {
                shifts::set_0_at_square(
                    action.move_to,
                    &mut self.bitboards[piece_at_moveto as usize],
                );
                let enemy_index = if self.is_white_move { 13 } else { 12 };
                shifts::set_0_at_square(action.move_to, &mut self.bitboards[enemy_index]);
            }
        }

        self.prev_move_data.push(UndoData {
            from: action.move_from,
            to: action.move_to,
            prev_castle_mask: self.castle_rights_mask,
            prev_passant_square: self.passant_square,
            tag: MoveTag::Normal,
            previously_captured: possible_pcapture,
        });

        // make the move on the mailbox
        self.mailbox[action.move_from as usize] = pieces::NOPIECE;
        self.mailbox[action.move_to as usize] = moving_piece;

        let is_double_move = (action.move_from as u8).abs_diff(action.move_to as u8) == 16;

        match moving_piece {
            // check if the pawn doublemoved
            pieces::WPAWN => {
                if is_double_move {
                    self.passant_square = Some(action.move_to as u8 - 8)
                }
            }

            pieces::BPAWN => {
                if is_double_move {
                    self.passant_square = Some(action.move_to as u8 + 8)
                }
            }

            // modify castle rights mask
            pieces::WKING => self.castle_rights_mask &= !(WK | WQ),
            pieces::BKING => self.castle_rights_mask &= !(BK | BQ),
            pieces::WROOK => match action.move_from {
                Squares::h1 => self.castle_rights_mask &= !WK,
                Squares::a1 => self.castle_rights_mask &= !WQ,
                _ => (),
            },
            pieces::BROOK => match action.move_from {
                Squares::h8 => self.castle_rights_mask &= !BK,
                Squares::a8 => self.castle_rights_mask &= !BQ,
                _ => (),
            },
            _ => (),
        }
    }

    #[inline]
    pub fn make_castle(&mut self, action: &Move) {
        let rook_start: Squares;
        let rook_to: Squares;

        match action.move_to {
            Squares::c1 => {
                rook_start = Squares::a1;
                rook_to = Squares::d1;
            }
            Squares::g1 => {
                rook_start = Squares::h1;
                rook_to = Squares::f1;
            }
            Squares::c8 => {
                rook_start = Squares::a8;
                rook_to = Squares::d8;
            }
            Squares::g8 => {
                rook_start = Squares::h8;
                rook_to = Squares::f8;
            }
            _ => panic!("Not an available castle!"),
        }

        //mutate the main bitboard
        {
            let total_bb = if self.is_white_move { 12 } else { 13 };
            let full_bb = &mut self.bitboards[total_bb];
            shifts::set_0_at_square(action.move_from, full_bb);
            shifts::set_1_at_square(action.move_to, full_bb);
            shifts::set_0_at_square(rook_start, full_bb);
            shifts::set_1_at_square(rook_to, full_bb);
        }

        let moving_piece = self.mailbox[action.move_from as usize];
        let moving_rook = self.mailbox[rook_start as usize];
        // bitboard move
        {
            let moving_piece_bitboard = &mut self.bitboards[moving_piece as usize];

            shifts::set_0_at_square(action.move_from, moving_piece_bitboard);
            shifts::set_1_at_square(action.move_to, moving_piece_bitboard);
        }

        {
            let moving_rook_bitboard = &mut self.bitboards[moving_rook as usize];
            shifts::set_0_at_square(rook_start, moving_rook_bitboard);
            shifts::set_1_at_square(rook_to, moving_rook_bitboard);
        }

        self.prev_move_data.push(UndoData {
            from: action.move_from,
            to: action.move_to,
            prev_castle_mask: self.castle_rights_mask,
            prev_passant_square: self.passant_square,
            tag: MoveTag::Castle,
            previously_captured: pieces::NOPIECE,
        });

        //mailbox move
        self.mailbox[action.move_from as usize] = pieces::NOPIECE;
        self.mailbox[action.move_to as usize] = moving_piece;
        self.mailbox[rook_start as usize] = pieces::NOPIECE;
        self.mailbox[rook_to as usize] = moving_rook;

        if self.is_white_move {
            self.castle_rights_mask &= !(WK | WQ);
        } else {
            self.castle_rights_mask &= !(BK | BQ);
        }
    }

    #[inline]
    pub fn make_promotion(&mut self, action: &Move) {
        //mutate the total bitboard
        {
            let total_bb = if self.is_white_move { 12 } else { 13 };
            shifts::set_0_at_square(action.move_from, &mut self.bitboards[total_bb]);
            shifts::set_1_at_square(action.move_to, &mut self.bitboards[total_bb]);
        }

        let moving_piece = self.mailbox[action.move_from as usize];

        // bitboard move
        {
            shifts::set_0_at_square(action.move_from, &mut self.bitboards[moving_piece as usize]);
            shifts::set_1_at_square(
                action.move_to,
                &mut self.bitboards[action.promote_to as usize],
            );
        }

        let possible_pcapture: u8;
        {
            let piece_at_moveto = self.mailbox[action.move_to as usize];
            possible_pcapture = piece_at_moveto;
            if piece_at_moveto != pieces::NOPIECE {
                shifts::set_0_at_square(
                    action.move_to,
                    &mut self.bitboards[piece_at_moveto as usize],
                );
                let enemy_index = if self.is_white_move { 13 } else { 12 };
                shifts::set_0_at_square(action.move_to, &mut self.bitboards[enemy_index]);
            }
        }

        self.prev_move_data.push(UndoData {
            from: action.move_from,
            to: action.move_to,
            prev_castle_mask: self.castle_rights_mask,
            prev_passant_square: self.passant_square,
            tag: MoveTag::Normal,
            previously_captured: possible_pcapture,
        });

        //mailbox move
        self.mailbox[action.move_from as usize] = pieces::NOPIECE;
        self.mailbox[action.move_to as usize] = action.promote_to;
    }

    #[inline]
    pub fn make_passant(&mut self, action: &Move) {
        //mutate the total bitboard
        {
            let total_bb = if self.is_white_move { 12 } else { 13 };
            shifts::set_0_at_square(action.move_from, &mut self.bitboards[total_bb]);
            shifts::set_1_at_square(action.move_to, &mut self.bitboards[total_bb]);
        }

        let moving_piece = self.mailbox[action.move_from as usize];

        // bitboard move
        {
            let moving_piece_bitboard = &mut self.bitboards[moving_piece as usize];
            shifts::set_0_at_square(action.move_from, moving_piece_bitboard);
            shifts::set_1_at_square(action.move_to, moving_piece_bitboard);
        }

        let square_val: i8 = if self.is_white_move { -8 } else { 8 };
        let pawn_bb_index: u8 = if self.is_white_move { 11 } else { 5 };

        // pawn bitboard
        let square_captured = action.move_to as i8 + square_val;

        {
            shifts::set_0_at_index(
                square_captured as u8,
                &mut self.bitboards[pawn_bb_index as usize],
            );

            // enemy allbitboards
            let enemy_index = if self.is_white_move { 13 } else { 12 };
            shifts::set_0_at_index(square_captured as u8, &mut self.bitboards[enemy_index]);
        }

        self.prev_move_data.push(UndoData {
            from: action.move_from,
            to: action.move_to,
            prev_castle_mask: self.castle_rights_mask,
            prev_passant_square: self.passant_square,
            tag: MoveTag::EPassant,
            previously_captured: pawn_bb_index,
        });

        //mailbox move
        self.mailbox[action.move_from as usize] = pieces::NOPIECE;
        self.mailbox[action.move_to as usize] = moving_piece;
        self.mailbox[square_captured as usize] = pieces::NOPIECE;
    }

    pub fn undo_move(&mut self) {
        let undo_data = self.prev_move_data.pop();
        match undo_data {
            Some(undo) => {
                self.passant_square = undo.prev_passant_square;
                self.castle_rights_mask = undo.prev_castle_mask;

                match undo.tag {
                    MoveTag::Normal => self.undo_normal(&undo),
                    MoveTag::Castle => self.undo_castle(&undo),
                    MoveTag::Promotion => self.undo_normal(&undo), // promotions are undo'd the same way as normals
                    MoveTag::EPassant => self.undo_passant(&undo),
                }
            }
            None => (),
        }
    }

    #[inline]
    pub fn undo_normal(&mut self, undo: &UndoData) {
        // move the old piece back to its original square
        let val_of_oldpiece = self.mailbox[undo.to as usize];
        // set the bitboard
        {
            let return_piece_bb = &mut self.bitboards[val_of_oldpiece as usize];
            set_0_at_square(undo.to, return_piece_bb);
            set_1_at_square(undo.from, return_piece_bb);
        }

        // reset captured piece
        if undo.previously_captured != pieces::NOPIECE {
            set_1_at_square(
                undo.to,
                &mut self.bitboards[undo.previously_captured as usize],
            );
        }

        // reset the mailbox
        self.mailbox[undo.from as usize] = val_of_oldpiece;
        self.mailbox[undo.to as usize] = undo.previously_captured;
    }

    #[inline]
    pub fn undo_castle(&mut self, undo: &UndoData) {
        let rook_from: Squares;
        let rook_to: Squares;
        match undo.to {
            Squares::c1 => {
                rook_from = Squares::a1;
                rook_to = Squares::d1;

            }
            Squares::g1 => {
                rook_from = Squares::h1;
                rook_to = Squares::f1;

            },
            Squares::c8 => {
                rook_from = Squares::a8;
                rook_to = Squares::d8;
                
            },
            Squares::g8 => {
                rook_from = Squares::h8;
                rook_to = Squares::f8;

            },
            _ => panic!("Not an available castle! This was triggered in the 'Undo' phase - consider checking the moves"),
        }

        // move the king back to its original square
        let old_king = self.mailbox[undo.to as usize];
        let old_rook = self.mailbox[rook_to as usize];
        // set the bitboard
        {
            let return_king_bb = &mut self.bitboards[old_king as usize];
            set_0_at_square(undo.to, return_king_bb);
            set_1_at_square(undo.from, return_king_bb);

            let return_rook_bb = &mut self.bitboards[old_rook as usize];
            set_0_at_square(rook_to, return_rook_bb);
            set_1_at_square(rook_from, return_rook_bb);
        }

        // reset the mailbox
        self.mailbox[undo.from as usize] = old_king;
        self.mailbox[undo.to as usize] = pieces::NOPIECE;

        self.mailbox[rook_from as usize] = old_rook;
        self.mailbox[rook_to as usize] = pieces::NOPIECE;
    }

    #[inline]
    pub fn undo_passant(&mut self, undo: &UndoData) {
        let val_of_oldpiece = self.mailbox[undo.to as usize];

        let old_pawn_square: i8 =
        match val_of_oldpiece {
            pieces::BPAWN => {
                undo.to as i8 + 8
            }
            pieces::WPAWN => {
                undo.to as i8 - 8
            }
            _ => panic!("Not a pawn!"),
        }; // determines the amount to add by when finding passant square


        {
            let return_piece_bb = &mut self.bitboards[val_of_oldpiece as usize];
            set_0_at_square(undo.to, return_piece_bb);
            set_1_at_square(undo.from, return_piece_bb);

            assert_ne!(undo.previously_captured, pieces::NOPIECE);
            set_1_at_index(old_pawn_square.try_into().unwrap(), &mut self.bitboards[undo.previously_captured as usize]);
        }

        self.mailbox[undo.from as usize] = val_of_oldpiece;
        self.mailbox[undo.to as usize] = pieces::NOPIECE;
        self.mailbox[old_pawn_square as usize] = undo.previously_captured;
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

pub struct Move {
    pub move_from: Squares,
    pub move_to: Squares,
    pub tag: MoveTag,
    pub promote_to: u8, // piece type - only seen if promotion is tagged
}

#[derive(Debug)]
pub enum MoveTag {
    Normal,
    Castle,
    Promotion,
    EPassant,
}

#[derive(Debug)]
pub struct UndoData {
    pub from: Squares,
    pub to: Squares,
    pub previously_captured: u8,
    pub prev_castle_mask: u8,
    pub prev_passant_square: Option<u8>,
    pub tag: MoveTag,
}
