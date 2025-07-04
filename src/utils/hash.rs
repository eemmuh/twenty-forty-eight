pub fn board_hash(board: &[[u32; 4]; 4]) -> u64 {
    let mut hash = 0u64;
    for (i, row) in board.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
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