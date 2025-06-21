use rand::prelude::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct GameBoard {
    pub board: [[u32; 4]; 4],
    pub move_count: u32,
    pub empty_mask: u16,  // Bitmask of empty cells
    pub max_tile: u32,    // Cached max tile
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
        for i in 0..4 {
            for j in 0..4 {
                if board[i][j] == 0 {
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
        for i in 0..4 {
            for j in 0..4 {
                if board[i][j] == 0 {
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
                for row in 0..4 {
                    let (new_row, row_moved) = Self::merge_row(&self.board[row]);
                    new_board[row] = new_row;
                    moved |= row_moved;
                }
            }
            Direction::Right => {
                for row in 0..4 {
                    let mut reversed_row = self.board[row];
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
} 


