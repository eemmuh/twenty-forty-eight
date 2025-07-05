use crate::game::{GameBoard, Direction};

impl GameBoard {
    // Smarter adaptive depth calculation
    pub fn calculate_smart_depth(&self) -> u32 {
        let empty_cells = self.count_empty_cells();
        let max_tile = self.get_max_tile();
        let board_complexity = self.calculate_board_complexity();
        
        // Base depth based on empty cells (more empty = deeper search possible)
        let base_depth = match empty_cells {
            0..=2 => 4,   // Endgame: shallow but fast
            3..=5 => 5,   // Late game: moderate depth
            6..=8 => 6,   // Mid game: deeper search
            9..=12 => 7,  // Early-mid game: good depth
            _ => 8,       // Early game: deepest search
        };
        
        // Adjust based on max tile (higher tiles need more careful analysis)
        let tile_bonus = match max_tile {
            1024..=u32::MAX => 2,  // Very high tiles: need deep analysis
            512..=1023 => 1,       // High tiles: slightly deeper
            256..=511 => 0,        // Medium tiles: normal depth
            _ => 0,                // Low tiles: normal depth
        };
        
        // Adjust based on board complexity
        let complexity_adjustment = if board_complexity > 0.7 {
            1  // Complex board: deeper search
        } else if board_complexity < 0.3 {
            0  // Simple board: normal depth
        } else {
            0  // Normal complexity: no adjustment
        };
        
        // Ensure depth is within reasonable bounds
        let total_depth = base_depth + tile_bonus + complexity_adjustment;
        total_depth.max(3).min(9)
    }
    
    // Calculate board complexity (0.0 = simple, 1.0 = complex)
    fn calculate_board_complexity(&self) -> f32 {
        let mut complexity = 0.0;
        
        // Check for value variety (more different values = more complex)
        let mut unique_values = std::collections::HashSet::new();
        for row in 0..4 {
            for col in 0..4 {
                if self.board[row][col] > 0 {
                    unique_values.insert(self.board[row][col]);
                }
            }
        }
        complexity += (unique_values.len() as f32) / 10.0; // Normalize to 0-1
        
        // Check for merge opportunities (more merges = more complex decisions)
        let mut merge_count = 0;
        for &direction in &Direction::all() {
            merge_count += self.count_merges_after_move(direction);
        }
        complexity += (merge_count as f32) / 8.0; // Normalize to 0-1
        
        // Check for monotonicity breaks (less monotonic = more complex)
        let monotonicity = self.calculate_monotonicity();
        complexity += (1.0 - (monotonicity / 1000.0).min(1.0)) * 0.5;
        
        complexity.min(1.0)
    }
    
    // Early termination conditions for search
    pub fn should_terminate_early(&self, depth: u32, current_score: f32, best_score: f32) -> bool {
        // If we have a very good move, don't waste time searching deeper
        if current_score > best_score + 1000.0 && depth > 2 {
            return true;
        }
        
        // If the board is nearly full, focus on immediate moves
        if self.count_empty_cells() <= 2 && depth > 3 {
            return true;
        }
        
        // If we have a dominant move (much better than others), use it
        if current_score > best_score * 1.5 && current_score > 500.0 {
            return true;
        }
        
        false
    }
    
    // Optimized expectimax with early termination
    pub fn expectimax_optimized(&mut self, depth: u32, is_maximizing: bool, alpha: f32, beta: f32) -> f32 {
        if depth == 0 {
            return self.evaluate_board_optimized();
        }
        
        if self.is_game_over() {
            return -100000.0;
        }
        
        // Check transposition table
        let hash = self.board_hash();
        if let Some(&cached_score) = crate::cache::TRANSPOSITION_TABLE.lock().unwrap().get(&hash) {
            let mut hits = crate::cache::CACHE_HITS.lock().unwrap();
            *hits += 1;
            return cached_score;
        } else {
            let mut misses = crate::cache::CACHE_MISSES.lock().unwrap();
            *misses += 1;
        }
        
        if is_maximizing {
            let mut best_score = f32::NEG_INFINITY;
            let mut alpha = alpha;
            
            // Use optimized move ordering
            let ordered_moves = self.order_moves();
            
            for direction in ordered_moves {
                let mut new_board = self.clone();
                if new_board.move_tiles(direction) {
                    new_board.empty_mask = GameBoard::calculate_empty_mask(&new_board.board);
                    new_board.max_tile = GameBoard::calculate_max_tile(&new_board.board);
                    
                    let score = new_board.expectimax_optimized(depth - 1, false, alpha, beta);
                    
                    if score > best_score {
                        best_score = score;
                        
                        // Early termination check
                        if self.should_terminate_early(depth, score, best_score) {
                            break;
                        }
                    }
                    
                    alpha = alpha.max(score);
                    if alpha >= beta {
                        break; // Alpha-beta cutoff
                    }
                }
            }
            
            if best_score == f32::NEG_INFINITY {
                best_score = self.evaluate_board_optimized();
            }
            
            crate::cache::TRANSPOSITION_TABLE.lock().unwrap().insert(hash, best_score);
            best_score
        } else {
            // Chance node - use strategic empty cell selection
            let empty_cells = self.get_strategic_empty_cells();
            if empty_cells.is_empty() {
                return self.evaluate_board_optimized();
            }
            
            let mut total_score = 0.0;
            let mut total_weight = 0.0;
            
            for &(i, j) in &empty_cells {
                // Try placing a 2 (90% probability)
                let mut new_board_2 = self.clone();
                new_board_2.board[i][j] = 2;
                new_board_2.empty_mask = GameBoard::calculate_empty_mask(&new_board_2.board);
                new_board_2.max_tile = GameBoard::calculate_max_tile(&new_board_2.board);
                
                let score_2 = new_board_2.expectimax_optimized(depth - 1, true, alpha, beta);
                total_score += score_2 * 0.9;
                total_weight += 0.9;
                
                // Try placing a 4 (10% probability)
                let mut new_board_4 = self.clone();
                new_board_4.board[i][j] = 4;
                new_board_4.empty_mask = GameBoard::calculate_empty_mask(&new_board_4.board);
                new_board_4.max_tile = GameBoard::calculate_max_tile(&new_board_4.board);
                
                let score_4 = new_board_4.expectimax_optimized(depth - 1, true, alpha, beta);
                total_score += score_4 * 0.1;
                total_weight += 0.1;
            }
            
            let avg_score = if total_weight > 0.0 {
                total_score / total_weight
            } else {
                self.evaluate_board_optimized()
            };
            
            crate::cache::TRANSPOSITION_TABLE.lock().unwrap().insert(hash, avg_score);
            avg_score
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_depth_calculation() {
        let mut board = GameBoard::new();
        
        // Early game should have deeper search
        board.set_board([
            [2, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        let early_depth = board.calculate_smart_depth();
        
        // Late game should have shallower search
        board.set_board([
            [1024, 512, 256, 128],
            [64, 32, 16, 8],
            [4, 2, 4, 2],
            [2, 4, 0, 0]
        ]);
        let late_depth = board.calculate_smart_depth();
        
        assert!(early_depth >= late_depth);
    }
    
    #[test]
    fn test_board_complexity() {
        let mut board = GameBoard::new();
        
        // Simple board
        board.set_board([
            [2, 2, 2, 2],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        let simple_complexity = board.calculate_board_complexity();
        
        // Complex board
        board.set_board([
            [1024, 512, 256, 128],
            [64, 32, 16, 8],
            [4, 2, 4, 2],
            [2, 4, 8, 16]
        ]);
        let complex_complexity = board.calculate_board_complexity();
        
        assert!(complex_complexity > simple_complexity);
    }
} 