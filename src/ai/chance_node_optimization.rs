use crate::game::GameBoard;

impl GameBoard {
    // Optimized empty cell selection for chance nodes
    pub(crate) fn get_strategic_empty_cells(&self) -> Vec<(usize, usize)> {
        let mut all_empty = self.get_empty_cells();
        
        // If few empty cells, consider them all
        if all_empty.len() <= 4 {
            return all_empty;
        }
        
        // If many empty cells, prioritize strategic positions
        if all_empty.len() > 8 {
            all_empty = self.prioritize_empty_cells(all_empty);
        }
        
        all_empty
    }
    
    // Prioritize empty cells based on strategic value
    fn prioritize_empty_cells(&self, empty_cells: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        let max_tile = self.get_max_tile();
        
        // Score each empty cell
        let mut cell_scores: Vec<((usize, usize), f32)> = empty_cells.iter()
            .map(|&(row, col)| {
                let score = self.score_empty_cell(row, col, max_tile);
                ((row, col), score)
            })
            .collect();
        
        // Sort by score (best first)
        cell_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Return top 6-8 cells
        let limit = (empty_cells.len() / 2).max(6).min(8);
        cell_scores.into_iter()
            .take(limit)
            .map(|(pos, _)| pos)
            .collect()
    }
    
    // Score an empty cell based on strategic factors
    fn score_empty_cell(&self, row: usize, col: usize, _max_tile: u32) -> f32 {
        let mut score = 0.0;
        
        // 1. Corner positions are valuable
        if (row == 0 || row == 3) && (col == 0 || col == 3) {
            score += 20.0;
        }
        
        // 2. Edge positions are moderately valuable
        else if row == 0 || row == 3 || col == 0 || col == 3 {
            score += 10.0;
        }
        
        // 3. Positions near the max tile are important
        let max_tile_pos = self.find_max_tile_position();
        if let Some((max_row, max_col)) = max_tile_pos {
            let distance = ((row as i32 - max_row as i32).abs() + 
                           (col as i32 - max_col as i32).abs()) as f32;
            score += 15.0 / (distance + 1.0);
        }
        
        // 4. Positions that could create merges are valuable
        let merge_potential = self.calculate_merge_potential_at(row, col);
        score += merge_potential * 5.0;
        
        // 5. Avoid positions that break monotonicity
        let monotonicity_penalty = self.calculate_monotonicity_penalty_at(row, col);
        score -= monotonicity_penalty * 3.0;
        
        score
    }
    
    // Find position of max tile
    fn find_max_tile_position(&self) -> Option<(usize, usize)> {
        let max_tile = self.get_max_tile();
        for row in 0..4 {
            for col in 0..4 {
                if self.board[row][col] == max_tile {
                    return Some((row, col));
                }
            }
        }
        None
    }
    
    // Calculate merge potential if a tile is placed at this position
    fn calculate_merge_potential_at(&self, row: usize, col: usize) -> f32 {
        let mut potential = 0.0;
        
        // Check adjacent cells for potential merges
        let neighbors = [
            (row.wrapping_sub(1), col),
            (row + 1, col),
            (row, col.wrapping_sub(1)),
            (row, col + 1),
        ];
        
        for &(nr, nc) in &neighbors {
            if nr < 4 && nc < 4 {
                let neighbor_value = self.board[nr][nc];
                if neighbor_value > 0 {
                    // Check if placing a 2 or 4 here could lead to merges
                    if neighbor_value == 2 || neighbor_value == 4 {
                        potential += 1.0;
                    }
                }
            }
        }
        
        potential
    }
    
    // Calculate penalty for breaking monotonicity
    fn calculate_monotonicity_penalty_at(&self, row: usize, col: usize) -> f32 {
        let mut penalty = 0.0;
        
        // Check if placing a tile here would break row monotonicity
        if col > 0 && col < 3 {
            let left = self.board[row][col - 1];
            let right = self.board[row][col + 1];
            if left > 0 && right > 0 {
                // Placing a 2 or 4 here might break monotonicity
                if (left > 4 && right > 4) || (left < 2 && right < 2) {
                    penalty += 1.0;
                }
            }
        }
        
        // Check if placing a tile here would break column monotonicity
        if row > 0 && row < 3 {
            let up = self.board[row - 1][col];
            let down = self.board[row + 1][col];
            if up > 0 && down > 0 {
                // Placing a 2 or 4 here might break monotonicity
                if (up > 4 && down > 4) || (up < 2 && down < 2) {
                    penalty += 1.0;
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
    fn test_strategic_empty_cells() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 4, 8, 16],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        
        let strategic_cells = board.get_strategic_empty_cells();
        assert!(!strategic_cells.is_empty());
        assert!(strategic_cells.len() <= 8);
    }
    
    #[test]
    fn test_empty_cell_scoring() {
        let mut board = GameBoard::new();
        board.set_board([
            [1024, 512, 256, 128],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        
        // Corner position near max tile should score high
        let corner_score = board.score_empty_cell(1, 0, 1024);
        let center_score = board.score_empty_cell(2, 2, 1024);
        
        assert!(corner_score > center_score);
    }
} 