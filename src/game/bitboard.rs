// High-performance bitboard representation for 2048
// Each tile is represented by 4 bits (0-15), allowing values up to 2^15 = 32768
// The entire board fits in a single u64

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn new() -> Self {
        BitBoard(0)
    }

    pub fn get_tile(&self, row: usize, col: usize) -> u32 {
        let shift = (row * 4 + col) * 4;
        let mask = 0xF << shift;
        let value = (self.0 & mask) >> shift;
        if value == 0 { 0 } else { 1 << value }
    }

    pub fn set_tile(&mut self, row: usize, col: usize, value: u32) {
        let shift = (row * 4 + col) * 4;
        let mask = 0xF << shift;
        self.0 &= !mask; // Clear the position
        
        if value > 0 {
            let log_value = value.trailing_zeros() as u64;
            self.0 |= log_value << shift;
        }
    }

    pub fn is_empty(&self, row: usize, col: usize) -> bool {
        let shift = (row * 4 + col) * 4;
        let mask = 0xF << shift;
        (self.0 & mask) == 0
    }

    pub fn count_empty(&self) -> u32 {
        let mut count = 0;
        for i in 0..16 {
            if (self.0 >> (i * 4)) & 0xF == 0 {
                count += 1;
            }
        }
        count
    }

    pub fn get_max_tile(&self) -> u32 {
        let mut max_log = 0;
        for i in 0..16 {
            let log_val = (self.0 >> (i * 4)) & 0xF;
            if log_val > max_log {
                max_log = log_val;
            }
        }
        if max_log == 0 { 0 } else { 1 << max_log }
    }

    // Ultra-fast move operations using bit manipulation
    pub fn move_left(&self) -> (BitBoard, bool) {
        let mut result = BitBoard(0);
        let mut moved = false;

        for row in 0..4 {
            let mut row_vals = [0u8; 4];
            for col in 0..4 {
                let shift = (row * 4 + col) * 4;
                row_vals[col] = ((self.0 >> shift) & 0xF) as u8;
            }

            let (new_row, row_moved) = Self::merge_row_bits(&row_vals);
            moved |= row_moved;

            for col in 0..4 {
                if new_row[col] > 0 {
                    let shift = (row * 4 + col) * 4;
                    result.0 |= (new_row[col] as u64) << shift;
                }
            }
        }

        (result, moved)
    }

    pub fn move_right(&self) -> (BitBoard, bool) {
        let mut result = BitBoard(0);
        let mut moved = false;

        for row in 0..4 {
            let mut row_vals = [0u8; 4];
            for col in 0..4 {
                let shift = (row * 4 + col) * 4;
                row_vals[3 - col] = ((self.0 >> shift) & 0xF) as u8;
            }

            let (new_row, row_moved) = Self::merge_row_bits(&row_vals);
            moved |= row_moved;

            for col in 0..4 {
                if new_row[col] > 0 {
                    let shift = (row * 4 + (3 - col)) * 4;
                    result.0 |= (new_row[col] as u64) << shift;
                }
            }
        }

        (result, moved)
    }

    pub fn move_up(&self) -> (BitBoard, bool) {
        let mut result = BitBoard(0);
        let mut moved = false;

        for col in 0..4 {
            let mut col_vals = [0u8; 4];
            for row in 0..4 {
                let shift = (row * 4 + col) * 4;
                col_vals[row] = ((self.0 >> shift) & 0xF) as u8;
            }

            let (new_col, col_moved) = Self::merge_row_bits(&col_vals);
            moved |= col_moved;

            for row in 0..4 {
                if new_col[row] > 0 {
                    let shift = (row * 4 + col) * 4;
                    result.0 |= (new_col[row] as u64) << shift;
                }
            }
        }

        (result, moved)
    }

    pub fn move_down(&self) -> (BitBoard, bool) {
        let mut result = BitBoard(0);
        let mut moved = false;

        for col in 0..4 {
            let mut col_vals = [0u8; 4];
            for row in 0..4 {
                let shift = (row * 4 + col) * 4;
                col_vals[3 - row] = ((self.0 >> shift) & 0xF) as u8;
            }

            let (new_col, col_moved) = Self::merge_row_bits(&col_vals);
            moved |= col_moved;

            for row in 0..4 {
                if new_col[row] > 0 {
                    let shift = ((3 - row) * 4 + col) * 4;
                    result.0 |= (new_col[row] as u64) << shift;
                }
            }
        }

        (result, moved)
    }

    // Optimized row merging using bit operations
    fn merge_row_bits(row: &[u8; 4]) -> ([u8; 4], bool) {
        let mut result = [0u8; 4];
        let mut write_pos = 0;
        let mut moved = false;

        // First, collect all non-zero values
        let mut non_zero_values = Vec::new();
        for &val in row {
            if val != 0 {
                non_zero_values.push(val);
            }
        }

        // If nothing to merge, but tiles moved positions
        if non_zero_values.len() != row.iter().filter(|&&x| x != 0).count() {
            moved = true;
        }

        // Now merge adjacent identical values
        let mut i = 0;
        while i < non_zero_values.len() {
            if i + 1 < non_zero_values.len() && non_zero_values[i] == non_zero_values[i + 1] {
                // Merge two identical tiles
                result[write_pos] = non_zero_values[i] + 1; // Add 1 to log value (double the tile)
                write_pos += 1;
                i += 2;
                moved = true;
            } else {
                // Just move the tile
                result[write_pos] = non_zero_values[i];
                write_pos += 1;
                i += 1;
            }
        }

        // Check if anything moved (comparing original positions)
        let mut original_pos = 0;
        for (pos, &val) in result.iter().enumerate() {
            if val != 0 {
                // Find where this value was originally
                while original_pos < 4 && row[original_pos] == 0 {
                    original_pos += 1;
                }
                if original_pos < 4 && pos != original_pos {
                    moved = true;
                    break;
                }
                original_pos += 1;
            }
        }

        (result, moved)
    }

    pub fn add_random_tile(&mut self) {
        let empty_positions: Vec<usize> = (0..16)
            .filter(|&i| (self.0 >> (i * 4)) & 0xF == 0)
            .collect();

        if !empty_positions.is_empty() {
            use rand::Rng;
            let pos = empty_positions[rand::thread_rng().gen_range(0..empty_positions.len())];
            let value = if rand::thread_rng().gen_bool(0.9) { 1 } else { 2 }; // log2(2) = 1, log2(4) = 2
            self.0 |= (value as u64) << (pos * 4);
        }
    }

    pub fn to_array(&self) -> [[u32; 4]; 4] {
        let mut result = [[0u32; 4]; 4];
        for row in 0..4 {
            for col in 0..4 {
                result[row][col] = self.get_tile(row, col);
            }
        }
        result
    }
}

impl Default for BitBoard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard_basic() {
        let mut board = BitBoard::new();
        board.set_tile(0, 0, 2);
        board.set_tile(0, 1, 4);
        
        assert_eq!(board.get_tile(0, 0), 2);
        assert_eq!(board.get_tile(0, 1), 4);
        assert_eq!(board.get_tile(0, 2), 0);
    }

    #[test]
    fn test_bitboard_move_left() {
        let mut board = BitBoard::new();
        board.set_tile(0, 0, 2);
        board.set_tile(0, 2, 2);
        
        let (new_board, moved) = board.move_left();
        
        assert!(moved);
        assert_eq!(new_board.get_tile(0, 0), 4);
        assert_eq!(new_board.get_tile(0, 1), 0);
        assert_eq!(new_board.get_tile(0, 2), 0);
        assert_eq!(new_board.get_tile(0, 3), 0);
    }

    #[test]
    fn test_bitboard_count_empty() {
        let mut board = BitBoard::new();
        assert_eq!(board.count_empty(), 16);
        
        board.set_tile(0, 0, 2);
        assert_eq!(board.count_empty(), 15);
    }
} 