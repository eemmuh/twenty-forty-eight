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
    // Score-optimized weights - prioritize moves that create higher scores
    pub fn for_game_state(max_tile: u32, empty_cells: usize) -> Self {
        let progress = if max_tile >= 1024 { 1.0 } 
                      else if max_tile >= 512 { 0.8 }
                      else if max_tile >= 128 { 0.6 }
                      else if max_tile >= 32 { 0.4 }
                      else { 0.2 };

        let _empty_factor = (empty_cells as f32 / 16.0).min(1.0);

        Self {
            monotonicity: 1.2 + progress * 2.0,      // Much more important for score
            smoothness: 0.15 + progress * 0.3,       // Helps maintain structure
            empty: 4.0 - progress * 1.5,             // Critical early, less later
            corner: 2.0 + progress * 3.0,            // Essential for high scores
            merge: 0.5 + progress * 0.3,             // More important as game progresses
            position: 1.2 + progress * 0.8,          // Better tile positioning
        }
    }
}

impl GameBoard {
    pub fn evaluate_board_optimized(&self) -> f32 {
        // Use the score-optimized evaluation by default for better scores
        self.evaluate_board_for_score()
    }

    // Score-optimized evaluation - specifically designed to maximize score
    pub fn evaluate_board_for_score(&self) -> f32 {
        let max_tile = self.get_max_tile();
        let empty_cells = self.count_empty_cells();
        let weights = OptimizedEvaluationWeights::for_game_state(max_tile, empty_cells);

        // Base score components
        let monotonicity = self.calculate_monotonicity();
        let smoothness = self.calculate_smoothness();
        let empty_score = empty_cells as f32;
        let corner_bonus = self.calculate_corner_bonus_optimized();
        let merge_potential = self.calculate_merge_potential();
        let position_score = self.calculate_position_score();

        // Score-specific bonuses
        let score_bonus = self.calculate_score_potential_bonus();
        let chain_bonus = self.calculate_chain_merge_bonus();
        let edge_control = self.calculate_edge_control_bonus();

        weights.monotonicity * monotonicity
            + weights.smoothness * smoothness
            + weights.empty * empty_score
            + weights.corner * corner_bonus
            + weights.merge * merge_potential
            + weights.position * position_score
            + score_bonus * 2.0           // High weight for score potential
            + chain_bonus * 1.5           // Chain merges are great for score
            + edge_control * 0.8          // Edge control helps maintain structure
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

    // Calculate bonus for moves that create scoring opportunities
    fn calculate_score_potential_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        
        // Bonus for having multiple high tiles that can merge
        let mut high_tiles = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] >= 64 {
                    high_tiles.push((i, j, self.board[i][j]));
                }
            }
        }
        
        // Check for potential merges between high tiles
        for i in 0..high_tiles.len() {
            for j in (i+1)..high_tiles.len() {
                if high_tiles[i].2 == high_tiles[j].2 {
                    let distance = (high_tiles[i].0 as i32 - high_tiles[j].0 as i32).abs() 
                                + (high_tiles[i].1 as i32 - high_tiles[j].1 as i32).abs();
                    if distance == 1 {
                        bonus += high_tiles[i].2 as f32 * 0.5; // Adjacent same tiles
                    } else if distance <= 3 {
                        bonus += high_tiles[i].2 as f32 * 0.2; // Close same tiles
                    }
                }
            }
        }
        
        bonus
    }

    // Calculate bonus for chain merge opportunities
    fn calculate_chain_merge_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        
        // Look for patterns like 2-4-8-16 that can chain merge
        for i in 0..4 {
            for j in 0..3 {
                let current = self.board[i][j];
                let next = self.board[i][j+1];
                if current > 0 && next > 0 && (current == next/2 || next == current/2) {
                    bonus += (current + next) as f32 * 0.3;
                }
            }
        }
        
        for i in 0..3 {
            for j in 0..4 {
                let current = self.board[i][j];
                let next = self.board[i+1][j];
                if current > 0 && next > 0 && (current == next/2 || next == current/2) {
                    bonus += (current + next) as f32 * 0.3;
                }
            }
        }
        
        bonus
    }

    // Calculate bonus for controlling edges (helps maintain structure)
    fn calculate_edge_control_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        
        // Bonus for having high tiles on edges
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] > 0 && (i == 0 || i == 3 || j == 0 || j == 3) {
                    bonus += self.board[i][j] as f32 * 0.1;
                }
            }
        }
        
        bonus
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