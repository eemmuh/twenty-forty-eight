use crate::game::{GameBoard, Direction};

impl GameBoard {
    // Enhanced move ordering with multiple heuristics
    pub(crate) fn order_moves(&self) -> Vec<Direction> {
        let directions = Direction::all();
        let mut move_scores: Vec<(Direction, f32)> = directions.iter()
            .map(|&direction| {
                let score = self.fast_move_score(direction);
                (direction, score)
            })
            .filter(|(_, score)| *score > f32::NEG_INFINITY)
            .collect();
        
        // Sort by score (best first) for optimal alpha-beta pruning
        move_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        move_scores.into_iter().map(|(direction, _)| direction).collect()
    }
    
    // Ultra-fast move scoring for ordering (no deep evaluation)
    fn fast_move_score(&self, direction: Direction) -> f32 {
        let mut new_board = self.clone();
        if !new_board.move_tiles(direction) {
            return f32::NEG_INFINITY;
        }
        
        let mut score = 0.0;
        
        // 1. Prioritize moves that create merges (very fast to calculate)
        let merges = self.count_merges_after_move(direction);
        score += merges as f32 * 50.0;
        
        // 2. Prioritize moves that keep max tile in corner (instant check)
        let max_tile = new_board.get_max_tile();
        let corners = [(0, 0), (0, 3), (3, 0), (3, 3)];
        for &(row, col) in &corners {
            if new_board.board[row][col] == max_tile {
                score += 100.0;
                break;
            }
        }
        
        // 3. Prefer moves that maintain monotonicity (fast row/column check)
        score += self.fast_monotonicity_score(&new_board) * 0.5;
        
        // 4. Bonus for empty cells (instant count)
        score += new_board.count_empty_cells() as f32 * 2.0;
        
        score
    }
    
    // Fast monotonicity check - only check main directions
    fn fast_monotonicity_score(&self, board: &GameBoard) -> f32 {
        let mut score = 0.0;
        
        // Check horizontal monotonicity (left-to-right)
        for row in 0..4 {
            let mut increasing = true;
            let mut decreasing = true;
            for col in 0..3 {
                let current = board.board[row][col];
                let next = board.board[row][col + 1];
                if current > 0 && next > 0 {
                    if current < next { decreasing = false; }
                    if current > next { increasing = false; }
                }
            }
            if increasing || decreasing {
                score += 10.0;
            }
        }
        
        // Check vertical monotonicity (top-to-bottom)
        for col in 0..4 {
            let mut increasing = true;
            let mut decreasing = true;
            for row in 0..3 {
                let current = board.board[row][col];
                let next = board.board[row + 1][col];
                if current > 0 && next > 0 {
                    if current < next { decreasing = false; }
                    if current > next { increasing = false; }
                }
            }
            if increasing || decreasing {
                score += 10.0;
            }
        }
        
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_ordering() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 4, 8, 16],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        
        let ordered_moves = board.order_moves();
        assert!(!ordered_moves.is_empty());
        assert!(ordered_moves.len() <= 4);
    }
    
    #[test]
    fn test_fast_move_score() {
        let mut board = GameBoard::new();
        board.set_board([
            [2, 2, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0]
        ]);
        
        // Left move should score high due to merge
        let left_score = board.fast_move_score(Direction::Left);
        let up_score = board.fast_move_score(Direction::Up);
        
        assert!(left_score > up_score);
    }
} 