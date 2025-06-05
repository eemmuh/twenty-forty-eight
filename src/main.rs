use std::collections::HashMap;
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use std::sync::Mutex;
use lazy_static::lazy_static;



lazy_static! {
    static ref TRANSPOSITION_TABLE: Mutex<HashMap<u64, f32>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct GameBoard {
    board: [[u32; 4]; 4],
    move_count: u32,
}

impl GameBoard {
    fn new() -> Self {
        let mut board = [[0; 4]; 4];
        // add two initial tiles
        Self::add_random_tile(&mut board);
        Self::add_random_tile(&mut board);
        
        GameBoard {
            board,
            move_count: 0,
        }
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

    fn move_tiles(&mut self, direction: Direction) -> bool {
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

    fn is_game_over(&self) -> bool {
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

    fn calculate_adaptive_depth(&self) -> u32 {
        let empty_cells = self.count_empty_cells();
        let max_tile = self.get_max_tile();
        
        // adaptive depth based on game state
        match empty_cells {
            0..=2 => 7,      // deep search when critical
            3..=5 => 6,      // medium search
            6..=8 => 5,      // standard search
            _ => 4,          // shallow search when many options
        }
        
        // adjust for high-value games
        .min(if max_tile >= 1024 { 8 } else { 6 })
    }

    fn count_empty_cells(&self) -> usize {
        let mut count = 0;
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] == 0 {
                    count += 1;
                }
            }
        }
        count
    }

    fn get_max_tile(&self) -> u32 {
        let mut max_tile = 0;
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] > max_tile {
                    max_tile = self.board[i][j];
                }
            }
        }
        max_tile
    }

    fn evaluate_board(&self) -> f32 {
        let hash = self.board_hash();
        if let Some(&cached_score) = TRANSPOSITION_TABLE.lock().unwrap().get(&hash) {
            return cached_score;
        }

        let mut score = 0.0;
        
        // enhanced position weights favoring corners and edges
        let weights = [
            [7.0, 6.0, 5.0, 4.0],
            [6.0, 5.0, 4.0, 3.0],
            [5.0, 4.0, 3.0, 2.0],
            [4.0, 3.0, 2.0, 1.0],
        ];
        
        // find highest tile and its position
        let mut highest_tile = 0;
        let mut highest_pos = (0, 0);
        
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] > highest_tile {
                    highest_tile = self.board[i][j];
                    highest_pos = (i, j);
                }
            }
        }
        
        // position-weighted scoring
        for i in 0..4 {
            for j in 0..4 {
                let value = self.board[i][j] as f32;
                if value > 0.0 {
                    score += value * value.log2() * weights[i][j];
                }
            }
        }
        
        // strong corner bonus for highest tile
        let corner_bonus = if (highest_pos.0 == 0 || highest_pos.0 == 3) && 
                             (highest_pos.1 == 0 || highest_pos.1 == 3) {
            highest_tile as f32 * 0.5
        } else {
            -(highest_tile as f32 * 0.1) // penalty for not being in corner
        };
        score += corner_bonus;
        
        // monotonicity evaluation
        let monotonicity = self.calculate_monotonicity();
        score += monotonicity * 50.0;
        
        // smoothness evaluation
        let smoothness = self.calculate_smoothness();
        score += smoothness * 10.0;
        
        // empty cells bonus (logarithmic)
        let empty_cells = self.count_empty_cells() as f32;
        let empty_bonus = if empty_cells > 0.0 {
            empty_cells.ln() * 100.0
        } else {
            -500.0 // heavy penalty for full board
        };
        score += empty_bonus;
        
        // edge preference for large tiles
        let edge_bonus = self.calculate_edge_bonus();
        score += edge_bonus;
        
        // merge potential bonus
        let merge_potential = self.calculate_merge_potential();
        score += merge_potential * 20.0;
        
        let final_score = score;
        TRANSPOSITION_TABLE.lock().unwrap().insert(hash, final_score);
        final_score
    }

    fn calculate_monotonicity(&self) -> f32 {
        let mut scores = [0.0; 4]; // up, down, left, right
        
        // check horizontal monotonicity
        for i in 0..4 {
            let mut current = 0;
            let mut next = current + 1;
            while next < 4 {
                while next < 4 && self.board[i][next] == 0 {
                    next += 1;
                }
                if next >= 4 { next -= 1; }
                
                let current_val = if self.board[i][current] != 0 {
                    self.board[i][current] as f32
                } else { 1.0 };
                let next_val = if self.board[i][next] != 0 {
                    self.board[i][next] as f32
                } else { 1.0 };
                
                if current_val > next_val {
                    scores[0] += next_val - current_val;
                } else if next_val > current_val {
                    scores[1] += current_val - next_val;
                }
                
                current = next;
                next += 1;
            }
        }
        
        // check vertical monotonicity  
        for j in 0..4 {
            let mut current = 0;
            let mut next = current + 1;
            while next < 4 {
                while next < 4 && self.board[next][j] == 0 {
                    next += 1;
                }
                if next >= 4 { next -= 1; }
                
                let current_val = if self.board[current][j] != 0 {
                    self.board[current][j] as f32
                } else { 1.0 };
                let next_val = if self.board[next][j] != 0 {
                    self.board[next][j] as f32
                } else { 1.0 };
                
                if current_val > next_val {
                    scores[2] += next_val - current_val;
                } else if next_val > current_val {
                    scores[3] += current_val - next_val;
                }
                
                current = next;
                next += 1;
            }
        }
        
        scores.iter().fold(0.0f32, |acc, &x| acc.max(x))
    }

    fn calculate_smoothness(&self) -> f32 {
        let mut smoothness = 0.0;
        
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] != 0 {
                    let value = self.board[i][j] as f32;
                    let target_value = value.log2();
                    
                    // check right neighbor
                    if j < 3 && self.board[i][j + 1] != 0 {
                        let neighbor = self.board[i][j + 1] as f32;
                        let neighbor_value = neighbor.log2();
                        smoothness -= (target_value - neighbor_value).abs();
                    }
                    
                    // check down neighbor
                    if i < 3 && self.board[i + 1][j] != 0 {
                        let neighbor = self.board[i + 1][j] as f32;
                        let neighbor_value = neighbor.log2();
                        smoothness -= (target_value - neighbor_value).abs();
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

    fn find_best_move(&mut self) -> Option<Direction> {
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

    fn add_random_tile_self(&mut self) {
        Self::add_random_tile(&mut self.board);
    }

    fn get_score(&self) -> u32 {
        self.board.iter().flatten().sum()
    }
}

fn main() {
    let mut game = GameBoard::new();
    let mut moves = 0;
    let max_moves = 5000;
    
    println!("Starting improved 2048 solver...");
    
    while !game.is_game_over() && moves < max_moves {
        if moves % 10 == 0 || moves < 10 {
            println!("\nMove {}", moves + 1);
            for row in &game.board {
                println!("{:>4} {:>4} {:>4} {:>4}", row[0], row[1], row[2], row[3]);
            }
            println!("Score: {}, Max tile: {}, Empty: {}", 
                     game.get_score(), 
                     game.get_max_tile(),
                     game.count_empty_cells());
        }
        
        if let Some(best_move) = game.find_best_move() {
            if game.move_tiles(best_move) {
                game.add_random_tile_self();
                moves += 1;
            } else {
                println!("Move failed - no changes made");
                break;
            }
        } else {
            println!("No valid moves found");
            break;
        }
        
        // clear transposition table periodically to manage memory
        if moves % 100 == 0 {
            TRANSPOSITION_TABLE.lock().unwrap().clear();
        }
    }
    
    println!("\nGame Over!");
    println!("Final board state:");
    for row in &game.board {
        println!("{:>4} {:>4} {:>4} {:>4}", row[0], row[1], row[2], row[3]);
    }
    println!("Total moves: {}", moves);
    println!("Highest tile: {}", game.get_max_tile());
    println!("Final score: {}", game.get_score());
    
    // performance statistics
    let cache_size = TRANSPOSITION_TABLE.lock().unwrap().len();
    println!("Transposition table entries: {}", cache_size);
}




