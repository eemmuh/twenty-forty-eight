pub fn board_hash(board: &[[u32; 4]; 4]) -> u64 {
    let mut hash = 0u64;
    for i in 0..4 {
        for j in 0..4 {
            let value = board[i][j];
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