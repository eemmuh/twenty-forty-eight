use twenty_forty_eight::GameBoard;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref TRANSPOSITION_TABLE: Mutex<HashMap<u64, f32>> = Mutex::new(HashMap::new());
}

fn main() {
    let mut game = GameBoard::new();
    let mut moves = 0;
    let max_moves = 5000;
    
    println!("Starting optimized 2048 solver...");
    
    while !game.is_game_over() && moves < max_moves {
        if moves % 10 == 0 || moves < 10 {
            println!("\nMove {}", moves + 1);
            for row in &game.get_board() {
                println!("{:>4} {:>4} {:>4} {:>4}", row[0], row[1], row[2], row[3]);
            }
            println!("Score: {}, Max tile: {}, Empty: {}", 
                     game.get_score(), 
                     game.get_max_tile(),
                     game.count_empty_cells());
        }
        
        if let Some(best_move) = game.find_best_move() {
            if game.move_tiles(best_move) {
                game.add_random_tile_self();
                moves += 1;
            } else {
                println!("Move failed - no changes made");
                break;
            }
        } else {
            println!("No valid moves found");
            break;
        }
        
        // Clear transposition table periodically to manage memory
        if moves % 50 == 0 {
            let cache_size = TRANSPOSITION_TABLE.lock().unwrap().len();
            println!("Cache size: {} entries", cache_size);
            if cache_size > 100000 {
                TRANSPOSITION_TABLE.lock().unwrap().clear();
                println!("Cache cleared to prevent memory bloat");
            }
        }
    }
    
    println!("\nGame Over!");
    println!("Final board state:");
    for row in &game.get_board() {
        println!("{:>4} {:>4} {:>4} {:>4}", row[0], row[1], row[2], row[3]);
    }
    println!("Total moves: {}", moves);
    println!("Highest tile: {}", game.get_max_tile());
    println!("Final score: {}", game.get_score());
    
    // Final cache statistics
    let final_cache_size = TRANSPOSITION_TABLE.lock().unwrap().len();
    println!("Final transposition table entries: {}", final_cache_size);
}




