use crate::game::GameBoard;

#[derive(Debug, Clone)]
pub struct EvaluationWeights {
    pub monotonicity: f32,
    pub smoothness: f32,
    pub empty: f32,
    pub corner: f32,
    pub edge: f32,
    pub merge: f32,
    pub snake: f32,
    pub isolation: f32,
    pub position: f32,
}

impl Default for EvaluationWeights {
    fn default() -> Self {
        Self {
            monotonicity: 1.0,
            smoothness: 0.1,
            empty: 2.7,
            corner: 1.5,
            edge: 0.5,
            merge: 0.2,
            snake: 1.0,
            isolation: -0.5,
            position: 1.0,
        }
    }
}

impl GameBoard {
    pub fn evaluate_board(&self) -> f32 {
        let weights = EvaluationWeights::default();
        let monotonicity = self.calculate_monotonicity();
        let smoothness = self.calculate_smoothness();
        let empty_cells = self.count_empty_cells() as f32;
        let corner_bonus = self.calculate_corner_bonus();
        let edge_bonus = self.calculate_edge_bonus();
        let merge_potential = self.calculate_merge_potential();
        let snake_score = self.calculate_snake_pattern();
        let isolation_penalty = self.calculate_isolation_penalty();
        let position_score = self.calculate_position_score();
        
        weights.monotonicity * monotonicity
            + weights.smoothness * smoothness
            + weights.empty * empty_cells
            + weights.corner * corner_bonus
            + weights.edge * edge_bonus
            + weights.merge * merge_potential
            + weights.snake * snake_score
            + weights.isolation * isolation_penalty
            + weights.position * position_score
    }

    pub(crate) fn calculate_position_score(&self) -> f32 {
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

    pub(crate) fn calculate_corner_bonus(&self) -> f32 {
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

    pub(crate) fn calculate_edge_bonus(&self) -> f32 {
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

    pub(crate) fn calculate_monotonicity(&self) -> f32 {
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

    pub(crate) fn calculate_smoothness(&self) -> f32 {
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

    pub(crate) fn calculate_merge_potential(&self) -> f32 {
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

    pub(crate) fn calculate_snake_pattern(&self) -> f32 {
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

    pub(crate) fn calculate_isolation_penalty(&self) -> f32 {
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
} 