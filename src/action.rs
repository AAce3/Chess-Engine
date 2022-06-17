use core::panic;
use std::u8;

use crate::{
    bit_operations::shifts::{set_0_at_index, set_1_at_index},
    board::{pieces, BoardData, Squares, ZOBRIST_TABLES},
};

/* == == == == == == == == == == == == == == == == == == == == == == == == == == ==
*   A move is represented by a 16-bit number
|
*   Promoted piece is only read if move type is promotion. So, we can use two bits
|   00 is queen
*   01 is knight
|   10 is bishop
*   11 is rook
|   It is important to not use these if it isn't a promotion
*   [15]
|   [14]
*
|   Move type requires 2 bits -
*   - 00 is normal move
|   - 01 is castle
*   - 10 is promotion
|   - 11 is en passant
*   [13]
|   [12]
*
|   Move-To requires 6 bits, because it is an index from 0 to 63
*   [11]
|   [10]
*   [9]
|   [8]
*   [7]
|   [6]
*
|   Move-From requires 6 bits, because it is an index from 0 to 63
*   [5]
|   [4]
*   [3]
|   [2]
*   [1]
|   [0]
*
   == == == == == == == == == == == == == == == == == == == == == == == == == == ==  */
pub type Action = u16;

pub mod actions {
    use crate::action::Action;
    // move types
    pub const NORMAL: u16 = 0;
    pub const CASTLE: u16 = 1;
    pub const PROMOTION: u16 = 0b10;
    pub const PASSANT: u16 = 0b11;

    pub const PR_QUEEN: u16 = 0;
    pub const PR_KNIGHT: u16 = 0b01;
    pub const PR_BISHOP: u16 = 0b10;
    pub const PR_ROOK: u16 = 0b11;

    pub fn new(move_from: u8, move_to: u8, move_type: u16, promote_to: u16) -> Action {
        (move_from as u16) | ((move_to as u16) << 6) | (move_type << 12) | (promote_to << 14)
    }
}

pub trait Move {
    fn move_from(&self) -> u8;

    fn move_to(&self) -> u8;

    fn move_type(&self) -> u16;

    fn promote_to(&self) -> u16;
}

impl Move for Action {
    fn move_from(&self) -> u8 {
        (self & 0b111111) as u8
    }

    fn move_to(&self) -> u8 {
        ((self >> 6) & 0b111111) as u8
    }

    fn move_type(&self) -> u16 {
        (self >> 12) & 0b11
    }

    fn promote_to(&self) -> u16 {
        (self >> 14) & 0b11
    }
}

impl BoardData {
    pub fn make_move(&mut self, action: Action) {
        let movefrom = action.move_from();
        let moveto = action.move_to();
        assert!(movefrom < 64);
        assert!(moveto < 64);

        let tag = action.move_type();
        let mut savestate = StateData::new(self);

        // default actions are performed
        self.half_move_counter += 1;
        // set the passant square to none in the zobrist key
        self.zobrist_key ^= match self.passant_square {
            None => 0,
            Some(sqr) => (sqr & 7) as u64,
        };

        self.zobrist_key ^= ZOBRIST_TABLES.castling_rights[self.castle_rights_mask as usize];
        // undos the castle mask

        match tag {
            actions::NORMAL => {
                self.passant_square = None;
                let movingpiece = self.mailbox[movefrom as usize];
                let maybe_piececapture = self.mailbox[moveto as usize];

                if maybe_piececapture != pieces::NOPIECE {
                    savestate.set_captured(maybe_piececapture); // a piece is captured, so the state must be saved
                    set_0_at_index(moveto, &mut self.bitboards[maybe_piececapture as usize]);
                    self.half_move_counter = 0; // a capture resets 50 move counter
                }
                //println!("Movefrom: {}, Moveto: {}", movefrom, moveto);
                self.move_piece(movefrom, moveto);

                // update the board data, with castle rights and en passant
                match movingpiece {
                    pieces::WPAWN => {
                        if moveto - movefrom == 16 {
                            // it is a doublemove, so we update the possible en passant square
                            self.passant_square = Some(moveto - 8);
                            // update zobrist key:
                            self.zobrist_key ^=
                                ZOBRIST_TABLES.passant_square[(moveto & 7) as usize];
                        }
                        self.half_move_counter = 0; // a pawn move resets hmc
                    }
                    pieces::BPAWN => {
                        if movefrom - moveto == 16 {
                            self.passant_square = Some(moveto + 8);
                            self.zobrist_key ^=
                                ZOBRIST_TABLES.passant_square[(moveto & 7) as usize];
                        }
                        self.half_move_counter = 0;
                    }
                    pieces::WKING => {
                        self.castle_rights_mask &= 0b0011; // sets the first two bits, WK and WQ to 0, as the king has moved
                    }
                    pieces::BKING => {
                        self.castle_rights_mask &= 0b1100; // sets the last two bits, BK and BQ to 0
                    }
                    pieces::WROOK => match movefrom {
                        0 => self.castle_rights_mask &= 0b1011, // sets WQ to 0, rook has moved from starting square
                        7 => self.castle_rights_mask &= 0b0111, // sets WK to 0
                        _ => (),
                    },
                    pieces::BROOK => match movefrom {
                        63 => self.castle_rights_mask &= 0b1101, // sets BK to 0
                        46 => self.castle_rights_mask &= 0b0110, // sets BQ to 0
                        _ => (),
                    },
                    _ => (),
                }
            }

            actions::PROMOTION => {
                self.passant_square = None;
                self.removepiece(movefrom);
                let maybe_piececapture = self.mailbox[moveto as usize];
                if maybe_piececapture != pieces::NOPIECE {
                    savestate.set_captured(maybe_piececapture);
                    self.removepiece(moveto);
                }
                let tomovetag = if self.to_move { 0 } else { 6 };
                let promote_to = match action.promote_to() {
                    actions::PR_QUEEN => pieces::WQUEEN + tomovetag,
                    actions::PR_KNIGHT => pieces::WKNIGHT + tomovetag,
                    actions::PR_BISHOP => pieces::WBISHOP + tomovetag,
                    actions::PR_ROOK => pieces::WROOK + tomovetag,
                    _ => panic!("Invalid promotion!"),
                };
                self.set_piece(moveto, promote_to);
            }

            actions::CASTLE => {
                self.passant_square = None;
                let rookfrom: u8;
                let rookto: u8;
                match moveto {
                    6 => {
                        //g1
                        rookfrom = 7;
                        rookto = 5;
                    }
                    2 => {
                        //c1
                        rookfrom = 0;
                        rookto = 3;
                    }
                    62 => {
                        //g8
                        rookfrom = 63;
                        rookto = 61;
                    }
                    58 => {
                        //c8
                        rookfrom = 56;
                        rookto = 59;
                    }
                    _ => panic!("Invalid castle!"),
                }

                self.move_piece(movefrom, moveto);
                self.move_piece(rookfrom, rookto);

                let removemask: u8 = if self.to_move { 0b0011 } else { 0b1100 };
                self.castle_rights_mask &= removemask;
            }
            actions::PASSANT => {
                let mut passantsq = match self.passant_square {
                    Some(square) => square,
                    None => panic!("Invalid En Passant!"),
                };
                if self.to_move {
                    passantsq -= 8;
                } else {
                    passantsq += 8;
                }
                savestate.set_captured(self.mailbox[passantsq as usize]);
                self.removepiece(passantsq);
                
                self.move_piece(movefrom, moveto);
                self.half_move_counter = 0; // resets halfmove ctr, as it is a pawn move
            }
            _ => panic!("Not an available move!"),
        }

        self.zobrist_key ^= ZOBRIST_TABLES.castling_rights[self.castle_rights_mask as usize];
        self.prev_states.push(savestate);
        self.to_move = !self.to_move;
    }

    pub fn undo_move(&mut self, action: Action) {
        let undo = match self.prev_states.pop() {
            Some(state) => state,
            None => return,
        };

        let movefrom = action.move_from();
        let moveto = action.move_to();
        let tag = action.move_type();

        if tag != actions::PROMOTION {
            self.move_piece(moveto, movefrom);
        } else {
            self.removepiece(moveto);
            self.set_piece(
                movefrom,
                if !self.to_move { // self.move has been swapped. Therefore if it is blacks move it must have been
                    // previously white's move
                    pieces::WPAWN
                } else {
                    pieces::BPAWN
                },
            );
        }
        if undo.captured_piece != pieces::NOPIECE && tag != actions::PASSANT {
            self.set_piece(moveto, undo.captured_piece);
        } else if tag == actions::PASSANT {
            match undo.passant_square {
                None => panic!("en passant invalid!"),
                Some(sqr) => {
                    let passantsq = if !self.to_move {
                        sqr - 8
                    } else {
                        sqr + 8
                    };
                    self.set_piece(passantsq, undo.captured_piece)
                },
            }
        } else if tag == actions::CASTLE {
            let rookfrom: Squares;
            let rookto: Squares;
            match moveto {
                6 => {
                    //g1
                    rookfrom = Squares::h1;
                    rookto = Squares::f1;
                }
                2 => {
                    //c1
                    rookfrom = Squares::a1;
                    rookto = Squares::d1;
                }
                62 => {
                    //g8
                    rookfrom = Squares::h1;
                    rookto = Squares::f1;
                }
                58 => {
                    //f8
                    rookfrom = Squares::a1;
                    rookto = Squares::d1;
                }
                _ => panic!("bad castle"),
            }
            self.move_piece(rookto as u8, rookfrom as u8);
        }

        undo.set_self(self); // resets board data
        self.to_move = !self.to_move
    }

    #[inline]
    fn move_piece(&mut self, start_square: u8, end_square: u8) {
        let moving_piece = self.mailbox[start_square as usize];
        let moving_bitboard = &mut self.bitboards[moving_piece as usize];

        set_0_at_index(start_square, moving_bitboard);
        set_1_at_index(end_square, moving_bitboard);
        self.mailbox[start_square as usize] = pieces::NOPIECE;
        self.mailbox[end_square as usize] = moving_piece;

        self.zobrist_key ^=
            ZOBRIST_TABLES.piecesquares[start_square as usize][moving_piece as usize];
        self.zobrist_key ^= ZOBRIST_TABLES.piecesquares[end_square as usize][moving_piece as usize];
    }

    // having a specific function for captures should speed up quiescence search
    pub fn do_capture(&mut self, action: Action) {
        unimplemented!()
    }
    // DONT USE IF THERES A PIECE ALREADY AT THE INDEX
    #[inline]
    fn set_piece(&mut self, square: u8, piece: u8) {
        self.mailbox[square as usize] = piece;
        set_1_at_index(square, &mut self.bitboards[piece as usize]);
        self.zobrist_key ^= ZOBRIST_TABLES.piecesquares[square as usize][piece as usize];
    }

    #[inline]
    fn removepiece(&mut self, square: u8) {
        let piece = self.mailbox[square as usize];
        set_0_at_index(square, &mut self.bitboards[piece as usize]);
        self.mailbox[square as usize] = pieces::NOPIECE;
        self.zobrist_key ^= ZOBRIST_TABLES.piecesquares[square as usize][piece as usize];
    }
}

#[derive(Debug)]
pub struct StateData {
    captured_piece: u8,
    passant_square: Option<u8>,
    castlemask: u8,
    halfmove_ctr: u8,
    zobrist_key: u64,
}

impl StateData {
    fn new(board: &BoardData) -> StateData {
        Self {
            captured_piece: pieces::NOPIECE,
            passant_square: board.passant_square,
            castlemask: board.castle_rights_mask,
            halfmove_ctr: board.half_move_counter,
            zobrist_key: board.zobrist_key,
        }
    }

    #[inline]
    fn set_captured(&mut self, captured: u8) {
        self.captured_piece = captured;
    }

    fn set_self(&self, board: &mut BoardData) {
        board.castle_rights_mask = self.castlemask;
        board.passant_square = self.passant_square;
        board.half_move_counter = self.halfmove_ctr;
        board.zobrist_key = self.zobrist_key;
    }
}
