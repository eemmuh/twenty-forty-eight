use crate::game::GameBoard;

#[derive(Debug, Clone)]
pub struct OptimizedEvaluationWeights {
    pub monotonicity: f32,
    pub smoothness: f32,
    pub empty: f32,
    pub corner: f32,
    pub merge: f32,
    pub position: f32,
}

impl OptimizedEvaluationWeights {
    // Simplified adaptive weights - less computation overhead
    pub fn for_game_state(max_tile: u32, _empty_cells: usize) -> Self {
        let progress = if max_tile >= 512 { 1.0 } 
                      else if max_tile >= 128 { 0.7 }
                      else if max_tile >= 32 { 0.4 }
                      else { 0.0 };

        Self {
            monotonicity: 1.0 + progress * 1.5,      // Increased importance in late game
            smoothness: 0.1 + progress * 0.2,        // Slightly more important later
            empty: 3.0 - progress * 1.0,             // Less important as game progresses
            corner: 1.5 + progress * 2.0,            // Much more important in late game
            merge: 0.3,                              // Constant - simple is better
            position: 1.0 + progress * 0.5,          // Slightly more important later
        }
    }
}

impl GameBoard {
    pub fn evaluate_board_optimized(&self) -> f32 {
        let max_tile = self.get_max_tile();
        let empty_cells = self.count_empty_cells();
        let weights = OptimizedEvaluationWeights::for_game_state(max_tile, empty_cells);

        // Use the existing, proven heuristics but with adaptive weights
        let monotonicity = self.calculate_monotonicity();
        let smoothness = self.calculate_smoothness();
        let empty_score = empty_cells as f32;
        let corner_bonus = self.calculate_corner_bonus_optimized();
        let merge_potential = self.calculate_merge_potential();
        let position_score = self.calculate_position_score();

        weights.monotonicity * monotonicity
            + weights.smoothness * smoothness
            + weights.empty * empty_score
            + weights.corner * corner_bonus
            + weights.merge * merge_potential
            + weights.position * position_score
    }

    // Optimized corner bonus - faster computation
    fn calculate_corner_bonus_optimized(&self) -> f32 {
        let max_tile = self.get_max_tile();
        
        // Check each corner for the max tile
        let corners = [(0, 0), (0, 3), (3, 0), (3, 3)];
        for &(row, col) in &corners {
            if self.board[row][col] == max_tile {
                return max_tile as f32 * 10.0; // Big bonus for max tile in corner
            }
        }
        
        // Check if max tile is on an edge
        for row in 0..4 {
            for col in 0..4 {
                if self.board[row][col] == max_tile {
                    if row == 0 || row == 3 || col == 0 || col == 3 {
                        return max_tile as f32 * 3.0; // Smaller bonus for edge
                    }
                }
            }
        }
        
        // Penalty if max tile is in the middle
        -(max_tile as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_weights() {
        let early_weights = OptimizedEvaluationWeights::for_game_state(4, 12);
        let late_weights = OptimizedEvaluationWeights::for_game_state(1024, 3);
        
        // Early game should prioritize empty cells more
        assert!(early_weights.empty > late_weights.empty);
        
        // Late game should prioritize corner and monotonicity more
        assert!(late_weights.corner > early_weights.corner);
        assert!(late_weights.monotonicity > early_weights.monotonicity);
    }

    #[test]
    fn test_optimized_evaluation_performance() {
        let mut board = GameBoard::new();
        board.set_board([
            [1024, 512, 256, 128],
            [64, 32, 16, 8],
            [4, 2, 4, 2],
            [0, 0, 0, 0]
        ]);
        
        // Should be much faster than advanced evaluation
        let start = std::time::Instant::now();
        let _score = board.evaluate_board_optimized();
        let duration = start.elapsed();
        
        // Should complete in under 1ms
        assert!(duration.as_millis() < 1);
    }
} 