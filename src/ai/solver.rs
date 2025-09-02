use crate::game::{GameBoard, Direction};

impl GameBoard {
    // Quick evaluation for move ordering - doesn't use transposition table
    pub(crate) fn quick_evaluate_move(&self, direction: Direction) -> f32 {
        let mut new_board = self.clone();
        if !new_board.move_tiles(direction) {
            return f32::NEG_INFINITY;
        }
        
        // Update cached values after move
        new_board.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board.board);
        new_board.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board.board);
        
        let mut score = 0.0;
        
        // Bonus for merges created (higher weight for score)
        let merges = self.count_merges_after_move(direction);
        score += merges as f32 * 150.0; // Increased from 100.0
        
        // Bonus for keeping high tiles in corners (critical for score)
        let highest_tile = new_board.get_max_tile();
        for i in 0..4 {
            for j in 0..4 {
                if new_board.board[i][j] == highest_tile && (i == 0 || i == 3) && (j == 0 || j == 3) {
                    score += highest_tile as f32 * 4.0; // Increased from 2.0
                }
            }
        }
        
        // Bonus for empty cells (more important for score)
        score += new_board.count_empty_cells() as f32 * 8.0; // Increased from 5.0
        
        // Bonus for maintaining snake pattern (better structure = better score)
        let snake_score = new_board.calculate_snake_pattern() * 0.2; // Increased from 0.1
        score += snake_score;
        
        // Penalty for creating isolated tiles (hurts score potential)
        let isolation_penalty = new_board.calculate_isolation_penalty() * 1.0; // Increased from 0.5
        score -= isolation_penalty;
        
        // NEW: Bonus for potential future merges
        let future_merge_potential = new_board.calculate_merge_potential() * 0.3;
        score += future_merge_potential;
        
        // NEW: Bonus for maintaining monotonicity (better for score)
        let monotonicity_bonus = new_board.calculate_monotonicity() * 0.5;
        score += monotonicity_bonus;
        
        score
    }
    
    // Count how many merges a move would create
    pub(crate) fn count_merges_after_move(&self, direction: Direction) -> u32 {
        let mut new_board = self.clone();
        if !new_board.move_tiles(direction) {
            return 0;
        }
        
        let mut merges = 0;
        match direction {
            Direction::Left | Direction::Right => {
                for i in 0..4 {
                    for j in 0..3 {
                        if new_board.board[i][j] != 0 && new_board.board[i][j] == new_board.board[i][j + 1] {
                            merges += 1;
                        }
                    }
                }
            }
            Direction::Up | Direction::Down => {
                for i in 0..3 {
                    for j in 0..4 {
                        if new_board.board[i][j] != 0 && new_board.board[i][j] == new_board.board[i + 1][j] {
                            merges += 1;
                        }
                    }
                }
            }
        }
        merges
    }

    pub fn find_best_move(&mut self) -> Option<Direction> {
        let depth = self.calculate_smart_depth();
        
        // Use optimized move ordering
        let ordered_moves = self.order_moves();
        
        if ordered_moves.is_empty() {
            return None;
        }
        
        // Deep evaluation with optimized search
        let mut best_score = f32::NEG_INFINITY;
        let mut best_move = None;
        
        for direction in ordered_moves {
            let mut new_board = self.clone();
            if new_board.move_tiles(direction) {
                // Update cached values after move
                new_board.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board.board);
                new_board.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board.board);
                
                let score = new_board.expectimax_optimized(depth - 1, false, f32::NEG_INFINITY, f32::INFINITY);
                if score > best_score {
                    best_score = score;
                    best_move = Some(direction);
                }
            }
        }
        
        best_move
    }
} 