use std::collections::HashMap;
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref TRANSPOSITION_TABLE: Mutex<HashMap<u64, f32>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct GameBoard {
    board: [[u32; 4]; 4],
    move_count: u32,
}

impl GameBoard {
    pub fn new() -> Self {
        let mut board = [[0; 4]; 4];
        // add two initial tiles
        Self::add_random_tile(&mut board);
        Self::add_random_tile(&mut board);
        
        GameBoard {
            board,
            move_count: 0,
        }
    }

    pub fn get_board(&self) -> [[u32; 4]; 4] {
        self.board
    }

    pub fn set_board(&mut self, board: [[u32; 4]; 4]) {
        self.board = board;
    }

    pub fn get_move_count(&self) -> u32 {
        self.move_count
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

    // convert board to u64 hash for transposition table
    fn board_hash(&self) -> u64 {
        let mut hash = 0u64;
        for i in 0..4 {
            for j in 0..4 {
                hash = hash.wrapping_mul(31).wrapping_add(self.board[i][j] as u64);
            }
        }
        hash
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
            }
        }
        
        // check if row changed
        if !moved {
            for i in 0..4 {
                if new_row[i] != row[i] {
                    moved = true;
                    break;
                }
            }
        }
        
        (new_row, moved)
    }

    pub fn is_game_over(&self) -> bool {
        // check for empty cells
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] == 0 {
                    return false;
                }
            }
        }
        
        // check for possible merges
        for i in 0..4 {
            for j in 0..4 {
                let current = self.board[i][j];
                if (i < 3 && current == self.board[i + 1][j]) ||
                   (j < 3 && current == self.board[i][j + 1]) {
                    return false;
                }
            }
        }
        
        true
    }

    pub fn count_empty_cells(&self) -> usize {
        self.board.iter().flatten().filter(|&&cell| cell == 0).count()
    }

    pub fn get_max_tile(&self) -> u32 {
        *self.board.iter().flatten().max().unwrap_or(&0)
    }

    pub fn get_score(&self) -> u32 {
        self.board.iter().flatten().sum()
    }

    pub fn add_random_tile_self(&mut self) {
        Self::add_random_tile(&mut self.board);
    }
}

// AI module
pub mod ai {
    use super::*;

    impl GameBoard {
        pub fn calculate_adaptive_depth(&self) -> u32 {
            let empty_cells = self.count_empty_cells();
            let _max_tile = self.get_max_tile();
            
            // adaptive depth based on game state
            match empty_cells {
                0..=2 => 3,  // very few empty cells, reduce depth
                3..=6 => 4,  // moderate empty cells
                7..=10 => 5, // many empty cells, can search deeper
                _ => 6,      // lots of empty cells, maximum depth
            }
        }

        pub fn evaluate_board(&self) -> f32 {
            let mut score = 0.0;
            
            // position-based scoring (prefer corners and edges)
            let position_weights = [
                [3.0, 2.0, 2.0, 3.0],
                [2.0, 1.5, 1.5, 2.0],
                [2.0, 1.5, 1.5, 2.0],
                [3.0, 2.0, 2.0, 3.0],
            ];
            
            for i in 0..4 {
                for j in 0..4 {
                    score += self.board[i][j] as f32 * position_weights[i][j];
                }
            }
            
            // bonus for empty cells
            score += self.count_empty_cells() as f32 * 100.0;
            
            // bonus for smoothness (adjacent tiles with similar values)
            score += self.calculate_smoothness() * 10.0;
            
            // bonus for monotonicity (tiles in increasing/decreasing order)
            score += self.calculate_monotonicity() * 10.0;
            
            // bonus for edge alignment
            score += self.calculate_edge_bonus();
            
            // bonus for merge potential
            score += self.calculate_merge_potential() * 5.0;
            
            score
        }

        fn calculate_monotonicity(&self) -> f32 {
            let mut monotonicity = 0.0;
            
            // check horizontal monotonicity
            for i in 0..4 {
                let mut current = 0;
                let mut next = current + 1;
                let mut current_direction = 0;
                let mut score = 0.0;
                
                while next < 4 {
                    while next < 4 && self.board[i][next] == 0 {
                        next += 1;
                    }
                    if next >= 4 {
                        break;
                    }
                    
                    let current_value = self.board[i][current] as f32;
                    let next_value = self.board[i][next] as f32;
                    
                    if current_value > next_value {
                        if current_direction > 0 {
                            score = 0.0;
                        } else if current_direction < 0 {
                            score += next_value;
                        } else {
                            score = next_value;
                        }
                        current_direction = -1;
                    } else if next_value > current_value {
                        if current_direction < 0 {
                            score = 0.0;
                        } else if current_direction > 0 {
                            score += current_value;
                        } else {
                            score = current_value;
                        }
                        current_direction = 1;
                    }
                    
                    current = next;
                    next += 1;
                }
                
                monotonicity += score;
            }
            
            // check vertical monotonicity
            for j in 0..4 {
                let mut current = 0;
                let mut next = current + 1;
                let mut current_direction = 0;
                let mut score = 0.0;
                
                while next < 4 {
                    while next < 4 && self.board[next][j] == 0 {
                        next += 1;
                    }
                    if next >= 4 {
                        break;
                    }
                    
                    let current_value = self.board[current][j] as f32;
                    let next_value = self.board[next][j] as f32;
                    
                    if current_value > next_value {
                        if current_direction > 0 {
                            score = 0.0;
                        } else if current_direction < 0 {
                            score += next_value;
                        } else {
                            score = next_value;
                        }
                        current_direction = -1;
                    } else if next_value > current_value {
                        if current_direction < 0 {
                            score = 0.0;
                        } else if current_direction > 0 {
                            score += current_value;
                        } else {
                            score = current_value;
                        }
                        current_direction = 1;
                    }
                    
                    current = next;
                    next += 1;
                }
                
                monotonicity += score;
            }
            
            monotonicity
        }

        fn calculate_smoothness(&self) -> f32 {
            let mut smoothness = 0.0;
            
            for i in 0..4 {
                for j in 0..4 {
                    if self.board[i][j] != 0 {
                        let current_value = self.board[i][j] as f32;
                        
                        // check right neighbor
                        if j < 3 && self.board[i][j + 1] != 0 {
                            smoothness -= (current_value - self.board[i][j + 1] as f32).abs();
                        }
                        
                        // check bottom neighbor
                        if i < 3 && self.board[i + 1][j] != 0 {
                            smoothness -= (current_value - self.board[i + 1][j] as f32).abs();
                        }
                    }
                }
            }
            
            smoothness
        }

        fn calculate_edge_bonus(&self) -> f32 {
            let mut bonus = 0.0;
            
            for i in 0..4 {
                for j in 0..4 {
                    let value = self.board[i][j] as f32;
                    if value >= 32.0 {
                        // bonus for being on edge
                        if i == 0 || i == 3 || j == 0 || j == 3 {
                            bonus += value * 0.1;
                        }
                        // extra bonus for corners
                        if (i == 0 || i == 3) && (j == 0 || j == 3) {
                            bonus += value * 0.1;
                        }
                    }
                }
            }
            
            bonus
        }

        fn calculate_merge_potential(&self) -> f32 {
            let mut potential = 0.0;
            
            for i in 0..4 {
                for j in 0..4 {
                    if self.board[i][j] != 0 {
                        let value = self.board[i][j];
                        
                        // check adjacent cells for merge potential
                        let adjacent_positions = [
                            (i.wrapping_sub(1), j),
                            (i + 1, j),
                            (i, j.wrapping_sub(1)),
                            (i, j + 1),
                        ];
                        
                        for (ni, nj) in adjacent_positions {
                            if ni < 4 && nj < 4 && self.board[ni][nj] == value {
                                potential += value as f32;
                            }
                        }
                    }
                }
            }
            
            potential
        }

        pub fn find_best_move(&mut self) -> Option<Direction> {
            let depth = self.calculate_adaptive_depth();
            let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];

            // use parallel evaluation for better performance
            let results: Vec<(Direction, f32)> = directions.par_iter()
                .filter_map(|&direction| {
                    let mut new_board = self.clone();
                    if new_board.move_tiles(direction) {
                        let score = new_board.expectimax(depth - 1, false);
                        Some((direction, score))
                    } else {
                        None
                    }
                })
                .collect();

            if results.is_empty() {
                return None;
            }

            // select best move
            let (best_move, _) = results.into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();
                
            Some(best_move)
        }

        fn expectimax(&mut self, depth: u32, is_maximizing: bool) -> f32 {
            if depth == 0 || self.is_game_over() {
                return self.evaluate_board();
            }
            
            if is_maximizing {
                let mut best_score = f32::NEG_INFINITY;
                let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
                
                for &direction in &directions {
                    let mut new_board = self.clone();
                    if new_board.move_tiles(direction) {
                        let score = new_board.expectimax(depth - 1, false);
                        best_score = best_score.max(score);
                    }
                }
                
                if best_score == f32::NEG_INFINITY {
                    return self.evaluate_board();
                }
                
                best_score
            } else {
                // computer's turn (add random tile)
                let mut empty_cells = Vec::new();
                for i in 0..4 {
                    for j in 0..4 {
                        if self.board[i][j] == 0 {
                            empty_cells.push((i, j));
                        }
                    }
                }
                
                if empty_cells.is_empty() {
                    return self.evaluate_board();
                }
                
                let mut total_score = 0.0;
                
                // weighted expectation: 90% chance of 2, 10% chance of 4
                for &(i, j) in &empty_cells {
                    // try placing a 2
                    let mut new_board_2 = self.clone();
                    new_board_2.board[i][j] = 2;
                    let score_2 = new_board_2.expectimax(depth - 1, true);
                    total_score += score_2 * 0.9;
                    
                    // try placing a 4
                    let mut new_board_4 = self.clone();
                    new_board_4.board[i][j] = 4;
                    let score_4 = new_board_4.expectimax(depth - 1, true);
                    total_score += score_4 * 0.1;
                }
                
                total_score / empty_cells.len() as f32
            }
        }
    }
}

// Web module
pub mod web; 