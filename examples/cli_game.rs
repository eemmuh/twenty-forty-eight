use twenty_forty_eight::{GameBoard, Direction};

fn main() {
    println!("2048 Game Example");
    println!("=================");
    
    let mut game = GameBoard::new();
    
    println!("Initial board:");
    print_board(&game.get_board());
    
    // Play a few moves
    let moves = [Direction::Right, Direction::Down, Direction::Left, Direction::Up];
    
    for (i, &direction) in moves.iter().enumerate() {
        println!("\nMove {}: {:?}", i + 1, direction);
        
        if game.move_tiles(direction) {
            game.add_random_tile_self();
            print_board(&game.get_board());
            println!("Score: {}, Max tile: {}", game.get_score(), game.get_max_tile());
        } else {
            println!("Invalid move!");
        }
    }
    
    println!("\nGame state after {} moves:", moves.len());
    println!("Score: {}", game.get_score());
    println!("Max tile: {}", game.get_max_tile());
    println!("Empty cells: {}", game.count_empty_cells());
    println!("Game over: {}", game.is_game_over());
}

fn print_board(board: &[[u32; 4]; 4]) {
    for row in board {
        println!("{:>4} {:>4} {:>4} {:>4}", row[0], row[1], row[2], row[3]);
    }
} 