use crate::board::{BoardData, pieces};

impl BoardData {
/* ========================================
*   To optimize by not creating a new vector   
|   and dumping all of the contents into a        
*   "Main vector", each of the functions will
|   take in a mutable reference to a vector 
*   and add to it.              
   ======================================   */

   fn generate_all_knightmoves(&self, add_to: &mut [u16]){
      let idx = if self.to_move {pieces::WKNIGHT} else {pieces::BKNIGHT};
      
   } 
}