use crate::game::{GameBoard, Direction};
use crate::cache::{TRANSPOSITION_TABLE, CACHE_HITS, CACHE_MISSES};

impl GameBoard {
    pub fn calculate_adaptive_depth(&self) -> u32 {
        let empty_cells = self.count_empty_cells();
        let max_tile = self.get_max_tile();
        let base_depth = match empty_cells {
            0..=2 => 6,
            3..=5 => 7,
            6..=8 => 8,
            _ => 9,
        };
        if max_tile >= 512 {
            base_depth + 3
        } else if max_tile >= 256 {
            base_depth + 2
        } else if max_tile >= 128 {
            base_depth + 1
        } else {
            base_depth
        }
    }

    pub(crate) fn expectimax(&mut self, depth: u32, is_maximizing: bool, alpha: f32, beta: f32) -> f32 {
        if depth == 0 {
            return self.evaluate_board_optimized();
        }
        if self.is_game_over() {
            return -100000.0;
        }
        let hash = self.board_hash();
        if let Some(&cached_score) = TRANSPOSITION_TABLE.lock().unwrap().get(&hash) {
            let mut hits = CACHE_HITS.lock().unwrap();
            *hits += 1;
            return cached_score;
        } else {
            let mut misses = CACHE_MISSES.lock().unwrap();
            *misses += 1;
        }
        if is_maximizing {
            let mut best_score = f32::NEG_INFINITY;
            let mut alpha = alpha;
            let directions = Direction::all();
            for &direction in &directions {
                let mut new_board = self.clone();
                if new_board.move_tiles(direction) {
                    // Update cached values after move
                    new_board.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board.board);
                    new_board.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board.board);
                    let score = new_board.expectimax(depth - 1, false, alpha, beta);
                    best_score = best_score.max(score);
                    alpha = alpha.max(score);
                    if alpha >= beta {
                        break;
                    }
                }
            }
            if best_score == f32::NEG_INFINITY {
                return self.evaluate_board_optimized();
            }
            TRANSPOSITION_TABLE.lock().unwrap().insert(hash, best_score);
            best_score
        } else {
            let empty_cells = self.get_empty_cells();
            if empty_cells.is_empty() {
                return self.evaluate_board_optimized();
            }
            let mut total_score = 0.0;
            let cells_to_consider = if empty_cells.len() > 16 {
                &empty_cells[..16]
            } else {
                &empty_cells
            };
            for &(i, j) in cells_to_consider {
                let mut new_board_2 = self.clone();
                new_board_2.board[i][j] = 2;
                // Update cached values after tile placement
                new_board_2.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board_2.board);
                new_board_2.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board_2.board);
                let score_2 = new_board_2.expectimax(depth - 1, true, alpha, beta);
                total_score += score_2 * 0.9;
                let mut new_board_4 = self.clone();
                new_board_4.board[i][j] = 4;
                // Update cached values after tile placement
                new_board_4.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board_4.board);
                new_board_4.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board_4.board);
                let score_4 = new_board_4.expectimax(depth - 1, true, alpha, beta);
                total_score += score_4 * 0.1;
            }
            let avg_score = total_score / cells_to_consider.len() as f32;
            TRANSPOSITION_TABLE.lock().unwrap().insert(hash, avg_score);
            avg_score
        }
    }

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

    // Improved board hash
    pub(crate) fn board_hash(&self) -> u64 {
        let mut hash = 0u64;
        for i in 0..4 {
            for j in 0..4 {
                let value = self.board[i][j];
                if value != 0 {
                    let log_value = value.trailing_zeros() as u64;
                    let position = (i * 4 + j) as u64;
                    hash ^= (log_value << 4) | position;
                    hash = hash.rotate_left(7);
                }
            }
        }
        hash
    }
} 