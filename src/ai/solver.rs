use crate::game::{GameBoard, Direction};

impl GameBoard {
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