use twenty_forty_eight::{GameBoard, get_cache_stats, clear_cache};

fn main() {
    let mut game = GameBoard::new();
    let mut moves = 0;
    let max_moves = 5000;

    println!("Starting score-optimized 2048 solver with enhanced AI...");
    
    while !game.is_game_over() && moves < max_moves {
        if moves % 50 == 0 || moves < 10 {
            println!("\nMove {}", moves + 1);
            for row in &game.get_board() {
                println!("{:>4} {:>4} {:>4} {:>4}", row[0], row[1], row[2], row[3]);
            }
            println!("Score: {}, Max tile: {}, Empty: {}", 
                     game.get_score(), 
                     game.get_max_tile(),
                     game.count_empty_cells());
        }
        
        // Use the optimized evaluation with original search for better performance
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
        
        // Clear transposition table less frequently and only if very large
        if moves % 200 == 0 {
            let (_hits, _misses, cache_size) = get_cache_stats();
            println!("Cache size: {} entries", cache_size);
            if cache_size > 1_000_000 {
                clear_cache();
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
    let (hits, misses, final_cache_size) = get_cache_stats();
    println!("Final transposition table entries: {}", final_cache_size);
    println!("Cache hits: {} | misses: {} | hit rate: {:.2}%", hits, misses, if hits + misses > 0 { (hits as f64 / (hits + misses) as f64) * 100.0 } else { 0.0 });
}




