use crate::game::GameBoard;

#[derive(Debug, Clone)]
pub struct AdaptiveEvaluationWeights {
    pub monotonicity: f32,
    pub smoothness: f32,
    pub empty: f32,
    pub corner: f32,
    pub edge: f32,
    pub merge: f32,
    pub potential_merges: f32,
    pub gradient: f32,
    pub clustering: f32,
    pub trap_avoidance: f32,
}

impl AdaptiveEvaluationWeights {
    // Adaptive weights based on game state
    pub fn for_game_state(max_tile: u32, empty_cells: usize) -> Self {
        let progress = (max_tile as f32).log2() / 16.0; // Normalize to 0-1 range
        let density = 1.0 - (empty_cells as f32 / 16.0);

        Self {
            // Early game: focus on empty cells and merges
            // Late game: focus on monotonicity and position
            monotonicity: 1.0 + progress * 2.0,
            smoothness: 0.1 + progress * 0.3,
            empty: 3.0 - progress * 1.5, // Less important as game progresses
            corner: 2.0 + progress * 3.0, // More important in late game
            edge: 0.5 + progress * 1.0,
            merge: 0.5 - progress * 0.3, // Less important in late game
            potential_merges: 1.0 + density * 0.5,
            gradient: 1.5 + progress * 1.0,
            clustering: -1.0 - density * 0.5, // Penalize clustering more when board is full
            trap_avoidance: 2.0 + progress * 2.0,
        }
    }
}

impl GameBoard {
    pub fn evaluate_board_advanced(&self) -> f32 {
        let max_tile = self.get_max_tile();
        let empty_cells = self.count_empty_cells();
        let weights = AdaptiveEvaluationWeights::for_game_state(max_tile, empty_cells);

        let monotonicity = self.calculate_monotonicity_improved();
        let smoothness = self.calculate_smoothness_improved();
        let empty_score = empty_cells as f32;
        let corner_bonus = self.calculate_corner_bonus_improved();
        let edge_bonus = self.calculate_edge_bonus();
        let merge_potential = self.calculate_merge_potential();
        let potential_merges = self.calculate_potential_merges();
        let gradient_score = self.calculate_gradient_score();
        let clustering_penalty = self.calculate_clustering_penalty();
        let trap_penalty = self.calculate_trap_avoidance();

        weights.monotonicity * monotonicity
            + weights.smoothness * smoothness
            + weights.empty * empty_score
            + weights.corner * corner_bonus
            + weights.edge * edge_bonus
            + weights.merge * merge_potential
            + weights.potential_merges * potential_merges
            + weights.gradient * gradient_score
            + weights.clustering * clustering_penalty
            + weights.trap_avoidance * trap_penalty
    }

    // Improved monotonicity calculation using log values
    fn calculate_monotonicity_improved(&self) -> f32 {
        let mut total_score: f32 = 0.0;

        // Calculate monotonicity for all four directions
        for &(dr, dc) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let mut score = 0.0;
            
            for row in 0..4 {
                for col in 0..4 {
                    let current = self.board[row][col];
                    if current == 0 { continue; }
                    
                    let next_row = row as i32 + dr;
                    let next_col = col as i32 + dc;
                    
                    if next_row >= 0 && next_row < 4 && next_col >= 0 && next_col < 4 {
                        let next = self.board[next_row as usize][next_col as usize];
                        if next > 0 {
                            let current_log = (current as f32).log2();
                            let next_log = (next as f32).log2();
                            
                            if current_log >= next_log {
                                score += current_log - next_log;
                            } else {
                                score -= (next_log - current_log) * 2.0; // Penalize violations more
                            }
                        }
                    }
                }
            }
            
            total_score = total_score.max(score);
        }

        total_score
    }

    // Improved smoothness using log differences
    fn calculate_smoothness_improved(&self) -> f32 {
        let mut smoothness = 0.0;
        
        for row in 0..4 {
            for col in 0..4 {
                let current = self.board[row][col];
                if current == 0 { continue; }
                
                let current_log = (current as f32).log2();
                
                // Check right neighbor
                if col < 3 && self.board[row][col + 1] > 0 {
                    let neighbor_log = (self.board[row][col + 1] as f32).log2();
                    smoothness -= (current_log - neighbor_log).abs();
                }
                
                // Check down neighbor
                if row < 3 && self.board[row + 1][col] > 0 {
                    let neighbor_log = (self.board[row + 1][col] as f32).log2();
                    smoothness -= (current_log - neighbor_log).abs();
                }
            }
        }
        
        smoothness
    }

    // Improved corner bonus with multiple corner strategies
    fn calculate_corner_bonus_improved(&self) -> f32 {
        let corners = [(0, 0), (0, 3), (3, 0), (3, 3)];
        let mut best_score = f32::NEG_INFINITY;
        
        for &(corner_row, corner_col) in &corners {
            let mut score = 0.0;
            let corner_value = self.board[corner_row][corner_col];
            
            if corner_value > 0 {
                let corner_log = (corner_value as f32).log2();
                score += corner_log * corner_log; // Quadratic bonus for high corner values
                
                // Bonus for having the max tile in corner
                if corner_value == self.get_max_tile() {
                    score += corner_log * 10.0;
                }
                
                // Bonus for having second-highest adjacent to corner
                let adjacent_positions = [
                    (corner_row, if corner_col == 0 { 1 } else { 2 }),
                    (if corner_row == 0 { 1 } else { 2 }, corner_col),
                ];
                
                for &(adj_row, adj_col) in &adjacent_positions {
                    let adj_value = self.board[adj_row][adj_col];
                    if adj_value > 0 {
                        let adj_log = (adj_value as f32).log2();
                        if adj_log >= corner_log - 2.0 { // Within 2 powers of 2
                            score += adj_log * 2.0;
                        }
                    }
                }
            }
            
            best_score = best_score.max(score);
        }
        
        best_score
    }

    // Calculate potential merges in next few moves
    fn calculate_potential_merges(&self) -> f32 {
        let mut potential = 0.0;
        
        // Look for tiles that can be merged with one move
        for row in 0..4 {
            for col in 0..4 {
                let current = self.board[row][col];
                if current == 0 { continue; }
                
                // Check if there's a matching tile that can be brought together
                for other_row in 0..4 {
                    for other_col in 0..4 {
                        if (other_row, other_col) == (row, col) { continue; }
                        
                        if self.board[other_row][other_col] == current {
                            // Check if they can be merged (same row/column with only zeros between)
                            if self.can_merge_tiles(row, col, other_row, other_col) {
                                potential += (current as f32).log2();
                            }
                        }
                    }
                }
            }
        }
        
        potential
    }

    fn can_merge_tiles(&self, row1: usize, col1: usize, row2: usize, col2: usize) -> bool {
        // Same row - check if path is clear
        if row1 == row2 {
            let start_col = col1.min(col2);
            let end_col = col1.max(col2);
            for col in (start_col + 1)..end_col {
                if self.board[row1][col] != 0 {
                    return false;
                }
            }
            return true;
        }
        
        // Same column - check if path is clear
        if col1 == col2 {
            let start_row = row1.min(row2);
            let end_row = row1.max(row2);
            for row in (start_row + 1)..end_row {
                if self.board[row][col1] != 0 {
                    return false;
                }
            }
            return true;
        }
        
        false
    }

    // Calculate gradient score (prefer decreasing values from corner)
    fn calculate_gradient_score(&self) -> f32 {
        let corners = [(0, 0), (0, 3), (3, 0), (3, 3)];
        let mut best_score = f32::NEG_INFINITY;
        
        for &(corner_row, corner_col) in &corners {
            let mut score = 0.0;
            
            for row in 0..4 {
                for col in 0..4 {
                    let value = self.board[row][col];
                    if value > 0 {
                        let distance = ((row as i32 - corner_row as i32).abs() + 
                                       (col as i32 - corner_col as i32).abs()) as f32;
                        let value_log = (value as f32).log2();
                        
                        // Prefer high values close to corner
                        score += value_log * (4.0 - distance);
                    }
                }
            }
            
            best_score = best_score.max(score);
        }
        
        best_score
    }

    // Penalize clustering of similar values
    fn calculate_clustering_penalty(&self) -> f32 {
        let mut penalty = 0.0;
        
        for row in 0..4 {
            for col in 0..4 {
                let current = self.board[row][col];
                if current == 0 { continue; }
                
                let mut similar_neighbors = 0;
                let neighbors = [
                    (row.wrapping_sub(1), col),
                    (row + 1, col),
                    (row, col.wrapping_sub(1)),
                    (row, col + 1),
                ];
                
                for &(nr, nc) in &neighbors {
                    if nr < 4 && nc < 4 {
                        let neighbor = self.board[nr][nc];
                        if neighbor > 0 {
                            let current_log = (current as f32).log2();
                            let neighbor_log = (neighbor as f32).log2();
                            
                            if (current_log - neighbor_log).abs() <= 1.0 {
                                similar_neighbors += 1;
                            }
                        }
                    }
                }
                
                if similar_neighbors >= 3 {
                    penalty += (current as f32).log2();
                }
            }
        }
        
        penalty
    }

    // Avoid creating traps (situations where high tiles get stuck)
    fn calculate_trap_avoidance(&self) -> f32 {
        let mut penalty = 0.0;
        
        for row in 1..3 {
            for col in 1..3 {
                let current = self.board[row][col];
                if current >= 64 { // Only check for medium-high tiles
                    let neighbors = [
                        self.board[row - 1][col],
                        self.board[row + 1][col],
                        self.board[row][col - 1],
                        self.board[row][col + 1],
                    ];
                    
                    let surrounded = neighbors.iter().all(|&n| n > 0);
                    if surrounded {
                        let current_log = (current as f32).log2();
                        let min_neighbor_log = neighbors.iter()
                            .map(|&n| (n as f32).log2())
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or(0.0);
                        
                        if current_log > min_neighbor_log + 2.0 {
                            penalty -= current_log * 5.0; // Heavy penalty for trapped high tiles
                        }
                    }
                }
            }
        }
        
        penalty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_weights() {
        let early_weights = AdaptiveEvaluationWeights::for_game_state(4, 12);
        let late_weights = AdaptiveEvaluationWeights::for_game_state(1024, 3);
        
        // Early game should prioritize empty cells more
        assert!(early_weights.empty > late_weights.empty);
        
        // Late game should prioritize corner and monotonicity more
        assert!(late_weights.corner > early_weights.corner);
        assert!(late_weights.monotonicity > early_weights.monotonicity);
    }

    #[test]
    fn test_potential_merges() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 0, 2, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        
        let potential = board.calculate_potential_merges();
        assert!(potential > 0.0); // Should detect potential merge
    }
} 