use crate::game::{GameBoard, Direction};

impl GameBoard {
    // Quick evaluation for move ordering - doesn't use transposition table
    fn quick_evaluate_move(&self, direction: Direction) -> f32 {
        let mut new_board = self.clone();
        if !new_board.move_tiles(direction) {
            return f32::NEG_INFINITY;
        }
        
        // Update cached values after move
        new_board.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board.board);
        new_board.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board.board);
        
        let mut score = 0.0;
        
        // Bonus for merges created
        let merges = self.count_merges_after_move(direction);
        score += merges as f32 * 100.0;
        
        // Bonus for keeping high tiles in corners
        let highest_tile = new_board.get_max_tile();
        for i in 0..4 {
            for j in 0..4 {
                if new_board.board[i][j] == highest_tile {
                    if (i == 0 || i == 3) && (j == 0 || j == 3) {
                        score += highest_tile as f32 * 2.0;
                    }
                }
            }
        }
        
        // Bonus for empty cells
        score += new_board.count_empty_cells() as f32 * 5.0;
        
        // Bonus for maintaining snake pattern
        let snake_score = new_board.calculate_snake_pattern() * 0.1;
        score += snake_score;
        
        // Penalty for creating isolated tiles
        let isolation_penalty = new_board.calculate_isolation_penalty() * 0.5;
        score -= isolation_penalty;
        
        score
    }
    
    // Count how many merges a move would create
    fn count_merges_after_move(&self, direction: Direction) -> u32 {
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
        let depth = self.calculate_adaptive_depth();
        let directions = Direction::all();
        
        // First pass: quick evaluation for move ordering
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
        
        // Sort moves by quick evaluation (best first) for optimal alpha-beta pruning
        move_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Second pass: deep evaluation with move ordering
        let mut best_score = f32::NEG_INFINITY;
        let mut best_move = None;
        
        for (direction, _) in move_scores {
            let mut new_board = self.clone();
            if new_board.move_tiles(direction) {
                // Update cached values after move
                new_board.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board.board);
                new_board.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board.board);
                let score = new_board.expectimax(depth - 1, false, f32::NEG_INFINITY, f32::INFINITY);
                if score > best_score {
                    best_score = score;
                    best_move = Some(direction);
                }
            }
        }
        
        best_move
    }
} 