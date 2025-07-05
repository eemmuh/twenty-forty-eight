use crate::game::{GameBoard, Direction};
use crate::cache::enhanced_transposition::EnhancedTranspositionTable;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct IterativeDeepeningConfig {
    pub max_time_ms: u64,
    pub min_depth: u32,
    pub max_depth: u32,
    pub time_per_move_ms: u64,
}



impl Default for IterativeDeepeningConfig {
    fn default() -> Self {
        Self {
            max_time_ms: 200,     // 200ms max per move for speed
            min_depth: 4,         // Always search at least depth 4
            max_depth: 10,        // Don't go beyond depth 10
            time_per_move_ms: 150, // Target 150ms per move for responsive play
        }
    }
}

impl GameBoard {
    pub fn find_best_move_iterative(&mut self, config: &IterativeDeepeningConfig) -> Option<Direction> {
        let start_time = Instant::now();
        let max_duration = Duration::from_millis(config.max_time_ms);
        
        let directions = Direction::all();
        let mut best_move = None;
        
        // Quick move ordering for better alpha-beta pruning
        let mut move_scores: Vec<(Direction, f32)> = directions.iter()
            .map(|&direction| {
                let quick_score = self.quick_evaluate_move(direction);
                (direction, quick_score)
            })
            .filter(|(_, score)| *score > f32::NEG_INFINITY)
            .collect();
        
        if move_scores.is_empty() {
            return None;
        }
        
        move_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Iterative deepening search
        for depth in config.min_depth..=config.max_depth {
            if start_time.elapsed() >= max_duration {
                break;
            }
            
            let mut depth_best_move = None;
            let mut depth_best_score = f32::NEG_INFINITY;
            let mut alpha = f32::NEG_INFINITY;
            let beta = f32::INFINITY;
            
            // Try to get move ordering from previous iteration
            if depth > config.min_depth {
                move_scores.sort_by(|a, b| {
                    let hash_a = self.get_move_hash(a.0);
                    let hash_b = self.get_move_hash(b.0);
                    
                    let has_entry_a = EnhancedTranspositionTable::get_best_move(hash_a).is_some();
                    let has_entry_b = EnhancedTranspositionTable::get_best_move(hash_b).is_some();
                    
                    match (has_entry_a, has_entry_b) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => b.1.partial_cmp(&a.1).unwrap(),
                    }
                });
            }
            
            for (direction, _) in &move_scores {
                if start_time.elapsed() >= max_duration {
                    break;
                }
                
                let mut new_board = self.clone();
                if new_board.move_tiles(*direction) {
                    // Update cached values after move
                    new_board.empty_mask = GameBoard::calculate_empty_mask(&new_board.board);
                    new_board.max_tile = GameBoard::calculate_max_tile(&new_board.board);
                    
                    let score = new_board.expectimax_with_timeout(
                        depth - 1, 
                        false, 
                        alpha, 
                        beta,
                        start_time,
                        max_duration
                    );
                    
                    if score > depth_best_score {
                        depth_best_score = score;
                        depth_best_move = Some(*direction);
                        alpha = alpha.max(score);
                    }
                }
            }
            
            // Update best move if we completed this depth
            if start_time.elapsed() < max_duration && depth_best_move.is_some() {
                best_move = depth_best_move;
                
                // Early termination for very good moves
                if depth_best_score > 10000.0 {
                    break;
                }
            }
            
            // Adaptive time management - if we're taking too long, stop
            let elapsed_ratio = start_time.elapsed().as_millis() as f64 / config.max_time_ms as f64;
            if elapsed_ratio > 0.8 {
                break;
            }
        }
        
        best_move
    }
    
    fn expectimax_with_timeout(
        &mut self, 
        depth: u32, 
        is_maximizing: bool, 
        alpha: f32, 
        beta: f32,
        start_time: Instant,
        max_duration: Duration
    ) -> f32 {
        // Check timeout
        if start_time.elapsed() >= max_duration {
            return self.evaluate_board_optimized();
        }
        
        if depth == 0 {
            return self.evaluate_board_optimized();
        }
        
        if self.is_game_over() {
            return -100000.0;
        }
        
        let hash = self.board_hash();
        
        // Check transposition table
        if let Some(cached_score) = EnhancedTranspositionTable::lookup(hash, depth, alpha, beta) {
            return cached_score;
        }
        
        if is_maximizing {
            let mut best_score = f32::NEG_INFINITY;
            let mut alpha = alpha;
            let directions = Direction::all();
            let mut best_move = None;
            
            for &direction in &directions {
                if start_time.elapsed() >= max_duration {
                    break;
                }
                
                let mut new_board = self.clone();
                if new_board.move_tiles(direction) {
                    new_board.empty_mask = GameBoard::calculate_empty_mask(&new_board.board);
                    new_board.max_tile = GameBoard::calculate_max_tile(&new_board.board);
                    
                    let score = new_board.expectimax_with_timeout(
                        depth - 1, 
                        false, 
                        alpha, 
                        beta,
                        start_time,
                        max_duration
                    );
                    
                    if score > best_score {
                        best_score = score;
                        best_move = Some(direction);
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
            
            // Store in transposition table
            EnhancedTranspositionTable::store(hash, best_score, depth, alpha, beta, best_move);
            best_score
        } else {
            // Chance node (random tile placement)
            let empty_cells = self.get_empty_cells();
            if empty_cells.is_empty() {
                return self.evaluate_board_advanced();
            }
            
            let mut total_score = 0.0;
            let mut total_weight = 0.0;
            
            // Limit number of empty cells to consider for performance
            let cells_to_consider = if empty_cells.len() > 8 {
                &empty_cells[..8]
            } else {
                &empty_cells
            };
            
            for &(i, j) in cells_to_consider {
                if start_time.elapsed() >= max_duration {
                    break;
                }
                
                // Try placing a 2
                let mut new_board_2 = self.clone();
                new_board_2.board[i][j] = 2;
                new_board_2.empty_mask = GameBoard::calculate_empty_mask(&new_board_2.board);
                new_board_2.max_tile = GameBoard::calculate_max_tile(&new_board_2.board);
                
                let score_2 = new_board_2.expectimax_with_timeout(
                    depth - 1, 
                    true, 
                    alpha, 
                    beta,
                    start_time,
                    max_duration
                );
                total_score += score_2 * 0.9;
                total_weight += 0.9;
                
                // Try placing a 4
                let mut new_board_4 = self.clone();
                new_board_4.board[i][j] = 4;
                new_board_4.empty_mask = GameBoard::calculate_empty_mask(&new_board_4.board);
                new_board_4.max_tile = GameBoard::calculate_max_tile(&new_board_4.board);
                
                let score_4 = new_board_4.expectimax_with_timeout(
                    depth - 1, 
                    true, 
                    alpha, 
                    beta,
                    start_time,
                    max_duration
                );
                total_score += score_4 * 0.1;
                total_weight += 0.1;
            }
            
            let avg_score = if total_weight > 0.0 {
                total_score / total_weight
            } else {
                self.evaluate_board_optimized()
            };
            
            // Store in transposition table
            EnhancedTranspositionTable::store(hash, avg_score, depth, alpha, beta, None);
            avg_score
        }
    }
    
    fn get_move_hash(&self, direction: Direction) -> u64 {
        let mut temp_board = self.clone();
        temp_board.move_tiles(direction);
        temp_board.board_hash()
    }
    
    // Get adaptive time limit based on game state
    pub fn get_adaptive_time_limit(&self) -> u64 {
        let empty_cells = self.count_empty_cells();
        let max_tile = self.get_max_tile();
        
        // Early game: faster moves
        // Late game: more time for critical decisions
        match (empty_cells, max_tile) {
            (12..=16, _) => 50,         // Early game: 50ms
            (8..=11, _) => 100,         // Mid game: 100ms
            (4..=7, _) => 200,          // Late game: 200ms
            (0..=3, _) => 300,          // End game: 300ms
            (_, tile) if tile >= 1024 => 250, // High tiles: more time
            _ => 150,                   // Default: 150ms
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterative_deepening() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 4, 8, 16],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        
        let config = IterativeDeepeningConfig {
            max_time_ms: 100,
            min_depth: 2,
            max_depth: 6,
            time_per_move_ms: 50,
        };
        
        let best_move = board.find_best_move_iterative(&config);
        assert!(best_move.is_some());
    }

    #[test]
    fn test_adaptive_time_limit() {
        let mut board = GameBoard::new();
        
        // Early game
        board.set_board([
            [2, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        assert_eq!(board.get_adaptive_time_limit(), 100);
        
        // Late game
        board.set_board([
            [1024, 512, 256, 128],
            [64, 32, 16, 8],
            [4, 2, 4, 2],
            [2, 4, 2, 0]
        ]);
        assert!(board.get_adaptive_time_limit() >= 500);
    }
} 