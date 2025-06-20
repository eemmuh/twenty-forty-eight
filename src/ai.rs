use crate::game::{GameBoard, Direction};
use crate::cache::{TRANSPOSITION_TABLE, CACHE_HITS, CACHE_MISSES};

impl GameBoard {
    // Tunable weights for evaluation function
    const W_MONOTONICITY: f32 = 1.0;
    const W_SMOOTHNESS: f32 = 0.1;
    const W_EMPTY: f32 = 2.7;
    const W_CORNER: f32 = 1.5;
    const W_EDGE: f32 = 0.5;
    const W_MERGE: f32 = 0.2;
    const W_SNAKE: f32 = 1.0;
    const W_ISOLATION: f32 = -0.5;
    const W_POSITION: f32 = 1.0;

    pub fn calculate_adaptive_depth(&self) -> u32 {
        let empty_cells = self.count_empty_cells();
        let max_tile = self.get_max_tile();
        let base_depth = match empty_cells {
            0..=2 => 6,
            3..=5 => 7,
            6..=8 => 8,
            _ => 9,
        };
        if max_tile >= 512 {
            base_depth + 3
        } else if max_tile >= 256 {
            base_depth + 2
        } else if max_tile >= 128 {
            base_depth + 1
        } else {
            base_depth
        }
    }

    pub fn evaluate_board(&self) -> f32 {
        let hash = self.board_hash();
        if let Some(&cached_score) = TRANSPOSITION_TABLE.lock().unwrap().get(&hash) {
            let mut hits = CACHE_HITS.lock().unwrap();
            *hits += 1;
            return cached_score;
        } else {
            let mut misses = CACHE_MISSES.lock().unwrap();
            *misses += 1;
        }
        let monotonicity = self.calculate_monotonicity();
        let smoothness = self.calculate_smoothness();
        let empty_cells = self.count_empty_cells() as f32;
        let corner_bonus = self.calculate_corner_bonus();
        let edge_bonus = self.calculate_edge_bonus();
        let merge_potential = self.calculate_merge_potential();
        let snake_score = self.calculate_snake_pattern();
        let isolation_penalty = self.calculate_isolation_penalty();
        let position_score = self.calculate_position_score();
        let score = Self::W_MONOTONICITY * monotonicity
            + Self::W_SMOOTHNESS * smoothness
            + Self::W_EMPTY * empty_cells
            + Self::W_CORNER * corner_bonus
            + Self::W_EDGE * edge_bonus
            + Self::W_MERGE * merge_potential
            + Self::W_SNAKE * snake_score
            + Self::W_ISOLATION * isolation_penalty
            + Self::W_POSITION * position_score;
        TRANSPOSITION_TABLE.lock().unwrap().insert(hash, score);
        score
    }

    fn calculate_position_score(&self) -> f32 {
        let mut score = 0.0;
        let snake_weights = [
            [20.0, 19.0, 18.0, 17.0],
            [12.0, 13.0, 14.0, 15.0],
            [11.0, 10.0, 9.0,  8.0],
            [0.0,  1.0,  2.0,  3.0],
        ];
        for i in 0..4 {
            for j in 0..4 {
                let value = self.board[i][j];
                if value > 0 {
                    score += value as f32 * snake_weights[i][j];
                }
            }
        }
        score
    }

    fn calculate_corner_bonus(&self) -> f32 {
        let mut highest_tile = 0;
        let mut highest_pos = (0, 0);
        for i in 0..4 {
            for j in 0..4 {
                let value = self.board[i][j];
                if value > highest_tile {
                    highest_tile = value;
                    highest_pos = (i, j);
                }
            }
        }
        if highest_pos == (0, 0) {
            highest_tile as f32 * 8.0
        } else if (highest_pos.0 == 0 || highest_pos.0 == 3) && (highest_pos.1 == 0 || highest_pos.1 == 3) {
            highest_tile as f32 * 4.0
        } else {
            -(highest_tile as f32 * 2.0)
        }
    }

    fn calculate_edge_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        for i in 0..4 {
            for j in 0..4 {
                let value = self.board[i][j] as f32;
                if value >= 32.0 {
                    if i == 0 || i == 3 || j == 0 || j == 3 {
                        bonus += value * 0.2;
                    }
                    if (i == 0 || i == 3) && (j == 0 || j == 3) {
                        bonus += value * 0.3;
                    }
                }
            }
        }
        bonus
    }

    fn calculate_monotonicity(&self) -> f32 {
        let mut monotonicity = 0.0;
        for i in 0..4 {
            let mut current = 0;
            let mut next = current + 1;
            let mut current_direction = 0;
            let mut score = 0.0;
            while next < 4 {
                while next < 4 && self.board[i][next] == 0 {
                    next += 1;
                }
                if next >= 4 {
                    break;
                }
                let current_value = self.board[i][current] as f32;
                let next_value = self.board[i][next] as f32;
                if current_value > next_value {
                    if current_direction > 0 {
                        score = 0.0;
                    } else if current_direction < 0 {
                        score += next_value;
                    } else {
                        score = next_value;
                    }
                    current_direction = -1;
                } else if next_value > current_value {
                    if current_direction < 0 {
                        score = 0.0;
                    } else if current_direction > 0 {
                        score += current_value;
                    } else {
                        score = current_value;
                    }
                    current_direction = 1;
                }
                current = next;
                next += 1;
            }
            monotonicity += score;
        }
        for j in 0..4 {
            let mut current = 0;
            let mut next = current + 1;
            let mut current_direction = 0;
            let mut score = 0.0;
            while next < 4 {
                while next < 4 && self.board[next][j] == 0 {
                    next += 1;
                }
                if next >= 4 {
                    break;
                }
                let current_value = self.board[current][j] as f32;
                let next_value = self.board[next][j] as f32;
                if current_value > next_value {
                    if current_direction > 0 {
                        score = 0.0;
                    } else if current_direction < 0 {
                        score += next_value;
                    } else {
                        score = next_value;
                    }
                    current_direction = -1;
                } else if next_value > current_value {
                    if current_direction < 0 {
                        score = 0.0;
                    } else if current_direction > 0 {
                        score += current_value;
                    } else {
                        score = current_value;
                    }
                    current_direction = 1;
                }
                current = next;
                next += 1;
            }
            monotonicity += score;
        }
        monotonicity
    }

    fn calculate_smoothness(&self) -> f32 {
        let mut smoothness = 0.0;
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] != 0 {
                    let current_value = self.board[i][j] as f32;
                    if j < 3 && self.board[i][j + 1] != 0 {
                        smoothness -= (current_value - self.board[i][j + 1] as f32).abs();
                    }
                    if i < 3 && self.board[i + 1][j] != 0 {
                        smoothness -= (current_value - self.board[i + 1][j] as f32).abs();
                    }
                }
            }
        }
        smoothness
    }

    fn calculate_merge_potential(&self) -> f32 {
        let mut potential = 0.0;
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] != 0 {
                    let value = self.board[i][j];
                    let adjacent_positions = [
                        (i.wrapping_sub(1), j),
                        (i + 1, j),
                        (i, j.wrapping_sub(1)),
                        (i, j + 1),
                    ];
                    for (ni, nj) in adjacent_positions {
                        if ni < 4 && nj < 4 && self.board[ni][nj] == value {
                            potential += value as f32;
                        }
                    }
                }
            }
        }
        potential
    }

    fn calculate_snake_pattern(&self) -> f32 {
        let mut score = 0.0;
        let snake_path = [
            (0, 0), (0, 1), (0, 2), (0, 3),
            (1, 3), (1, 2), (1, 1), (1, 0),
            (2, 0), (2, 1), (2, 2), (2, 3),
            (3, 3), (3, 2), (3, 1), (3, 0)
        ];
        for (idx, &(i, j)) in snake_path.iter().enumerate() {
            let value = self.board[i][j] as f32;
            if value > 0.0 {
                score += value * (16 - idx) as f32;
            }
        }
        score
    }

    fn calculate_isolation_penalty(&self) -> f32 {
        let mut penalty = 0.0;
        for i in 0..4 {
            for j in 0..4 {
                let value = self.board[i][j];
                if value >= 64 {
                    let mut isolated = true;
                    let adjacent_positions = [
                        (i.wrapping_sub(1), j),
                        (i + 1, j),
                        (i, j.wrapping_sub(1)),
                        (i, j + 1),
                    ];
                    for (ni, nj) in adjacent_positions {
                        if ni < 4 && nj < 4 && self.board[ni][nj] != 0 {
                            isolated = false;
                            break;
                        }
                    }
                    if isolated {
                        penalty += value as f32;
                    }
                }
            }
        }
        penalty
    }

    pub fn find_best_move(&mut self) -> Option<Direction> {
        let depth = self.calculate_adaptive_depth();
        let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        let mut move_scores: Vec<(Direction, f32)> = directions.iter()
            .filter_map(|&direction| {
                let mut new_board = self.clone();
                if new_board.move_tiles(direction) {
                    // Update cached values after move
                    new_board.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board.board);
                    new_board.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board.board);
                    let quick_score = new_board.evaluate_board();
                    let highest_tile = new_board.get_max_tile();
                    let mut corner_bonus = 0.0;
                    for i in 0..4 {
                        for j in 0..4 {
                            if new_board.board[i][j] == highest_tile {
                                if (i == 0 || i == 3) && (j == 0 || j == 3) {
                                    corner_bonus += highest_tile as f32 * 0.2;
                                }
                            }
                        }
                    }
                    let merge_bonus = new_board.calculate_merge_potential() * 0.1;
                    let empty_bonus = new_board.count_empty_cells() as f32 * 10.0;
                    Some((direction, quick_score + corner_bonus + merge_bonus + empty_bonus))
                } else {
                    None
                }
            })
            .collect();
        if move_scores.is_empty() {
            return None;
        }
        move_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let results: Vec<(Direction, f32)> = move_scores.into_iter()
            .map(|(direction, _)| {
                let mut new_board = self.clone();
                new_board.move_tiles(direction);
                // Update cached values after move
                new_board.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board.board);
                new_board.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board.board);
                let score = new_board.expectimax(depth - 1, false, f32::NEG_INFINITY, f32::INFINITY);
                (direction, score)
            })
            .collect();
        let (best_move, _) = results.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        Some(best_move)
    }

    fn expectimax(&mut self, depth: u32, is_maximizing: bool, alpha: f32, beta: f32) -> f32 {
        if depth == 0 {
            return self.evaluate_board();
        }
        if self.is_game_over() {
            return -100000.0;
        }
        let hash = self.board_hash();
        if let Some(&cached_score) = TRANSPOSITION_TABLE.lock().unwrap().get(&hash) {
            let mut hits = CACHE_HITS.lock().unwrap();
            *hits += 1;
            return cached_score;
        } else {
            let mut misses = CACHE_MISSES.lock().unwrap();
            *misses += 1;
        }
        if is_maximizing {
            let mut best_score = f32::NEG_INFINITY;
            let mut alpha = alpha;
            let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
            for &direction in &directions {
                let mut new_board = self.clone();
                if new_board.move_tiles(direction) {
                    // Update cached values after move
                    new_board.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board.board);
                    new_board.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board.board);
                    let score = new_board.expectimax(depth - 1, false, alpha, beta);
                    best_score = best_score.max(score);
                    alpha = alpha.max(score);
                    if alpha >= beta {
                        break;
                    }
                }
            }
            if best_score == f32::NEG_INFINITY {
                return self.evaluate_board();
            }
            TRANSPOSITION_TABLE.lock().unwrap().insert(hash, best_score);
            best_score
        } else {
            let empty_cells = self.get_empty_cells();
            if empty_cells.is_empty() {
                return self.evaluate_board();
            }
            let mut total_score = 0.0;
            let cells_to_consider = if empty_cells.len() > 16 {
                &empty_cells[..16]
            } else {
                &empty_cells
            };
            for &(i, j) in cells_to_consider {
                let mut new_board_2 = self.clone();
                new_board_2.board[i][j] = 2;
                // Update cached values after tile placement
                new_board_2.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board_2.board);
                new_board_2.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board_2.board);
                let score_2 = new_board_2.expectimax(depth - 1, true, alpha, beta);
                total_score += score_2 * 0.9;
                let mut new_board_4 = self.clone();
                new_board_4.board[i][j] = 4;
                // Update cached values after tile placement
                new_board_4.empty_mask = crate::game::GameBoard::calculate_empty_mask(&new_board_4.board);
                new_board_4.max_tile = crate::game::GameBoard::calculate_max_tile(&new_board_4.board);
                let score_4 = new_board_4.expectimax(depth - 1, true, alpha, beta);
                total_score += score_4 * 0.1;
            }
            let avg_score = total_score / cells_to_consider.len() as f32;
            TRANSPOSITION_TABLE.lock().unwrap().insert(hash, avg_score);
            avg_score
        }
    }

    fn get_empty_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                if (self.empty_mask & (1 << (i * 4 + j))) != 0 {
                    cells.push((i, j));
                }
            }
        }
        cells
    }

    // Improved board hash
    fn board_hash(&self) -> u64 {
        let mut hash = 0u64;
        for i in 0..4 {
            for j in 0..4 {
                let value = self.board[i][j];
                if value != 0 {
                    let log_value = value.trailing_zeros() as u64;
                    let position = (i * 4 + j) as u64;
                    hash ^= (log_value << 4) | position;
                    hash = hash.rotate_left(7);
                }
            }
        }
        hash
    }
} 