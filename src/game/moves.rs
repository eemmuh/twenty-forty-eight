#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn all() -> [Direction; 4] {
        [Direction::Up, Direction::Down, Direction::Left, Direction::Right]
    }
} 