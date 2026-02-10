use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use super::moves::Direction;

/// Zobrist keys: 16 cells × 16 value classes (0=empty, 1–15=log2 of tile).
/// Deterministic seed so same position always hashes the same.
fn zobrist_table() -> [[u64; 16]; 16] {
    let mut rng = StdRng::seed_from_u64(0x2048_2048);
    let mut t = [[0u64; 16]; 16];
    for i in 0..16 {
        for j in 0..16 {
            t[i][j] = rng.gen();
        }
    }
    t
}

lazy_static::lazy_static! {
    static ref ZOBRIST: [[u64; 16]; 16] = zobrist_table();
}

#[derive(Debug, Clone)]
pub struct GameBoard {
    pub board: [[u32; 4]; 4],
    pub move_count: u32,
    pub empty_mask: u16,  // Bitmask of empty cells
    pub max_tile: u32,    // Cached max tile
}

impl Default for GameBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl GameBoard {
    pub fn new() -> Self {
        let mut board = [[0; 4]; 4];
        Self::add_random_tile(&mut board);
        Self::add_random_tile(&mut board);
        let empty_mask = Self::calculate_empty_mask(&board);
        let max_tile = Self::calculate_max_tile(&board);
        GameBoard {
            board,
            move_count: 0,
            empty_mask,
            max_tile,
        }
    }

    pub fn get_board(&self) -> [[u32; 4]; 4] {
        self.board
    }

    pub fn set_board(&mut self, board: [[u32; 4]; 4]) {
        self.board = board;
        self.empty_mask = Self::calculate_empty_mask(&board);
        self.max_tile = Self::calculate_max_tile(&board);
    }

    pub fn get_move_count(&self) -> u32 {
        self.move_count
    }

    pub(crate) fn calculate_empty_mask(board: &[[u32; 4]; 4]) -> u16 {
        let mut mask = 0u16;
        for (i, row) in board.iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                if cell == 0 {
                    mask |= 1 << (i * 4 + j);
                }
            }
        }
        mask
    }

    pub(crate) fn calculate_max_tile(board: &[[u32; 4]; 4]) -> u32 {
        board.iter().flatten().max().copied().unwrap_or(0)
    }

    fn add_random_tile(board: &mut [[u32; 4]; 4]) {
        let mut empty_cells = Vec::new();
        for (i, row) in board.iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                if cell == 0 {
                    empty_cells.push((i, j));
                }
            }
        }
        if let Some((i, j)) = empty_cells.choose(&mut rand::thread_rng()) {
            board[*i][*j] = if rand::random::<f32>() < 0.9 { 2 } else { 4 };
        }
    }

    pub fn move_tiles(&mut self, direction: Direction) -> bool {
        let mut moved = false;
        let mut new_board = self.board;
        match direction {
            Direction::Left => {
                for (row, board_row) in self.board.iter().enumerate() {
                    let (new_row, row_moved) = Self::merge_row(board_row);
                    new_board[row] = new_row;
                    moved |= row_moved;
                }
            }
            Direction::Right => {
                for (row, board_row) in self.board.iter().enumerate() {
                    let mut reversed_row = *board_row;
                    reversed_row.reverse();
                    let (merged_row, row_moved) = Self::merge_row(&reversed_row);
                    new_board[row] = {
                        let mut result = merged_row;
                        result.reverse();
                        result
                    };
                    moved |= row_moved;
                }
            }
            Direction::Up => {
                for col in 0..4 {
                    let column: [u32; 4] = [
                        self.board[0][col],
                        self.board[1][col],
                        self.board[2][col],
                        self.board[3][col],
                    ];
                    let (merged_col, col_moved) = Self::merge_row(&column);
                    for (row, &value) in merged_col.iter().enumerate() {
                        new_board[row][col] = value;
                    }
                    moved |= col_moved;
                }
            }
            Direction::Down => {
                for col in 0..4 {
                    let mut column: [u32; 4] = [
                        self.board[0][col],
                        self.board[1][col],
                        self.board[2][col],
                        self.board[3][col],
                    ];
                    column.reverse();
                    let (merged_col, col_moved) = Self::merge_row(&column);
                    let mut result_col = merged_col;
                    result_col.reverse();
                    for (row, &value) in result_col.iter().enumerate() {
                        new_board[row][col] = value;
                    }
                    moved |= col_moved;
                }
            }
        }
        if moved {
            self.board = new_board;
            self.move_count += 1;
            self.empty_mask = Self::calculate_empty_mask(&self.board);
            self.max_tile = Self::calculate_max_tile(&self.board);
        }
        moved
    }

    fn merge_row(row: &[u32; 4]) -> ([u32; 4], bool) {
        let mut new_row = [0; 4];
        let mut write_pos = 0;
        let mut i = 0;
        let mut moved = false;
        while i < 4 {
            if row[i] == 0 {
                i += 1;
                continue;
            }
            if i + 1 < 4 && row[i] == row[i + 1] && row[i] != 0 {
                new_row[write_pos] = row[i] * 2;
                write_pos += 1;
                i += 2;
                moved = true;
            } else {
                new_row[write_pos] = row[i];
                write_pos += 1;
                i += 1;
                if write_pos - 1 != i - 1 {
                    moved = true;
                }
            }
        }
        (new_row, moved)
    }

    pub fn is_game_over(&self) -> bool {
        if self.count_empty_cells() > 0 {
            return false;
        }
        for i in 0..4 {
            for j in 0..4 {
                let value = self.board[i][j];
                if (i < 3 && self.board[i + 1][j] == value)
                    || (j < 3 && self.board[i][j + 1] == value)
                {
                    return false;
                }
            }
        }
        true
    }

    pub fn count_empty_cells(&self) -> usize {
        self.board.iter().flatten().filter(|&&x| x == 0).count()
    }

    pub fn get_max_tile(&self) -> u32 {
        self.max_tile
    }

    pub fn get_score(&self) -> u32 {
        self.board.iter().flatten().sum()
    }

    pub fn add_random_tile_self(&mut self) {
        Self::add_random_tile(&mut self.board);
        self.empty_mask = Self::calculate_empty_mask(&self.board);
        self.max_tile = Self::calculate_max_tile(&self.board);
    }

    /// List of (row, col) for every empty cell. Used by AI chance nodes.
    pub(crate) fn get_empty_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                if (self.empty_mask & (1 << (i * 4 + j))) != 0 {
                    cells.push((i, j));
                }
            }
        }
        cells
    }

    /// 64-bit Zobrist hash for transposition table. Low collision rate so
    /// we keep more useful entries and get better cache hit rate.
    pub(crate) fn board_hash(&self) -> u64 {
        let mut hash = 0u64;
        for i in 0..4 {
            for j in 0..4 {
                let pos = i * 4 + j;
                let value = self.board[i][j];
                let value_index = if value == 0 { 0 } else { value.trailing_zeros() as usize };
                hash ^= ZOBRIST[pos][value_index];
            }
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_row_basic() {
        // Test basic merging
        let row = [2, 2, 0, 0];
        let (result, moved) = GameBoard::merge_row(&row);
        assert_eq!(result, [4, 0, 0, 0]);
        assert!(moved);
    }

    #[test]
    fn test_merge_row_no_merge() {
        // Test no possible merges
        let row = [2, 4, 8, 16];
        let (result, moved) = GameBoard::merge_row(&row);
        assert_eq!(result, [2, 4, 8, 16]);
        assert!(!moved);
    }

    #[test]
    fn test_merge_row_with_gaps() {
        // Test moving tiles without merging (gaps close but values don't merge)
        let row = [2, 0, 2, 0];
        let (result, moved) = GameBoard::merge_row(&row);
        assert_eq!(result, [2, 2, 0, 0]);
        assert!(moved);
    }

    #[test]
    fn test_merge_row_multiple_merges() {
        // Test multiple merges in one row
        let row = [2, 2, 4, 4];
        let (result, moved) = GameBoard::merge_row(&row);
        assert_eq!(result, [4, 8, 0, 0]);
        assert!(moved);
    }

    #[test]
    fn test_merge_row_no_consecutive_merges() {
        // Test that tiles don't merge consecutively
        let row = [4, 2, 2, 0];
        let (result, moved) = GameBoard::merge_row(&row);
        assert_eq!(result, [4, 4, 0, 0]);
        assert!(moved);
    }

    #[test]
    fn test_calculate_empty_mask() {
        let board = [
            [2, 0, 4, 0],
            [0, 8, 0, 16],
            [32, 0, 64, 0],
            [0, 128, 0, 256]
        ];
        let mask = GameBoard::calculate_empty_mask(&board);
        // Empty positions: (0,1), (0,3), (1,0), (1,2), (2,1), (2,3), (3,0), (3,2)
        let expected = (1 << 1) | (1 << 3) | (1 << 4) | (1 << 6) | (1 << 9) | (1 << 11) | (1 << 12) | (1 << 14);
        assert_eq!(mask, expected);
    }

    #[test]
    fn test_calculate_max_tile() {
        let board = [
            [2, 4, 8, 16],
            [32, 64, 128, 256],
            [512, 1024, 2048, 4096],
            [8192, 16384, 32768, 65536]
        ];
        assert_eq!(GameBoard::calculate_max_tile(&board), 65536);
    }

    #[test]
    fn test_count_empty_cells() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 0, 4, 0],
            [0, 8, 0, 16],
            [32, 0, 64, 0],
            [0, 128, 0, 256]
        ]);
        assert_eq!(board.count_empty_cells(), 8);
    }

    #[test]
    fn test_is_game_over_with_empty_cells() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 0, 4, 8],
            [16, 32, 64, 128],
            [256, 512, 1024, 2048],
            [4096, 8192, 16384, 32768]
        ]);
        assert!(!board.is_game_over());
    }

    #[test]
    fn test_is_game_over_with_possible_merges() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 4, 8, 16],
            [32, 64, 128, 256],
            [512, 1024, 2048, 4096],
            [8192, 16384, 32768, 32768] // Two 32768s can merge
        ]);
        assert!(!board.is_game_over());
    }

    #[test]
    fn test_is_game_over_true() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 4, 8, 16],
            [32, 64, 128, 256],
            [512, 1024, 2048, 4096],
            [8192, 16384, 32768, 65536]
        ]);
        assert!(board.is_game_over());
    }

    #[test]
    fn test_move_tiles_left() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 0, 2, 0],
            [4, 4, 0, 0],
            [0, 8, 0, 8],
            [16, 0, 0, 0]
        ]);
        
        let moved = board.move_tiles(Direction::Left);
        assert!(moved);
        
        let expected = [
            [2, 2, 0, 0],  // 2,0,2,0 -> 2,2,0,0 (no merge, just move)
            [8, 0, 0, 0],  // 4,4,0,0 -> 8,0,0,0 (merge)
            [8, 8, 0, 0],  // 0,8,0,8 -> 8,8,0,0 (no merge, just move)
            [16, 0, 0, 0]  // 16,0,0,0 -> 16,0,0,0 (no change)
        ];
        assert_eq!(board.board, expected);
    }

    #[test]
    fn test_move_tiles_right() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 0, 2, 0],
            [4, 4, 0, 0],
            [0, 8, 0, 8],
            [16, 0, 0, 0]
        ]);
        
        let moved = board.move_tiles(Direction::Right);
        assert!(moved);
        
        let expected = [
            [0, 0, 2, 2],  // 2,0,2,0 -> 0,0,2,2 (no merge, just move right)
            [0, 0, 0, 8],  // 4,4,0,0 -> 0,0,0,8 (merge and move right)
            [0, 0, 8, 8],  // 0,8,0,8 -> 0,0,8,8 (no merge, just move right)
            [0, 0, 0, 16]  // 16,0,0,0 -> 0,0,0,16 (move right)
        ];
        assert_eq!(board.board, expected);
    }
} 