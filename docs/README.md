# Project Documentation

## Project Structure

```
twenty-forty-eight/
├── Cargo.toml              # Project configuration and dependencies
├── README.md               # Main project README
├── .gitignore              # Git ignore rules
├── src/
│   ├── lib.rs              # Library entry point and public exports
│   ├── main.rs             # CLI binary entry point (AI solver)
│   ├── game/               # Game logic module
│   │   ├── mod.rs          # Game module entry point
│   │   ├── board.rs        # GameBoard implementation (main game logic)
│   │   ├── moves.rs        # Direction enum (Up, Down, Left, Right)
│   │   └── bitboard.rs     # Bitboard representation (unused alternative)
│   ├── ai/                 # AI and solver module
│   │   ├── mod.rs          # AI module entry point
│   │   ├── solver.rs       # Main AI solver - find_best_move() entry point
│   │   ├── adaptive_search.rs # Optimized expectimax with smart depth
│   │   ├── evaluation.rs   # Basic board evaluation heuristics
│   │   ├── optimized_evaluation.rs # Score-optimized evaluation (active)
│   │   ├── advanced_evaluation.rs # Advanced heuristics (dormant)
│   │   ├── search.rs       # Basic expectimax (dormant)
│   │   ├── iterative_deepening.rs # Time-bounded search (dormant)
│   │   ├── move_ordering.rs # Move ordering for alpha-beta pruning
│   │   └── chance_node_optimization.rs # Strategic empty cell selection
│   ├── cache/              # Caching module
│   │   ├── mod.rs          # Cache module entry point
│   │   ├── transposition.rs # Basic transposition table (active)
│   │   └── enhanced_transposition.rs # Enhanced caching (dormant)
│   └── bin/                # Additional binaries (empty)
├── examples/
│   └── cli_game.rs         # Example CLI game usage
└── docs/
    └── README.md           # This documentation file
```

## Module Overview

### Game Module (`src/game/`)
- **`board.rs`**: Core game board implementation with move logic, tile merging, and game state management
- **`moves.rs`**: Direction enum (Up, Down, Left, Right) and helper functions
- **`bitboard.rs`**: Alternative bitboard representation (unused - kept for potential optimization)
- **`mod.rs`**: Public interface - exports `GameBoard`, `Direction`, and `BitBoard`

### AI Module (`src/ai/`)
**Active Modules:**
- **`solver.rs`**: Main AI solver entry point - `find_best_move()` method
- **`adaptive_search.rs`**: Optimized expectimax with adaptive depth, move ordering, and early termination
- **`optimized_evaluation.rs`**: Score-focused board evaluation (currently used)
- **`move_ordering.rs`**: Fast move scoring and ordering for better alpha-beta pruning
- **`chance_node_optimization.rs`**: Strategic empty cell selection for chance nodes

**Dormant Modules (available but not used in main execution):**
- **`evaluation.rs`**: Basic evaluation heuristics
- **`advanced_evaluation.rs`**: Advanced evaluation with trap detection
- **`search.rs`**: Basic expectimax implementation
- **`iterative_deepening.rs`**: Time-bounded iterative deepening search

**`mod.rs`**: Public interface - exports evaluation weights and configs

### Cache Module (`src/cache/`)
- **`transposition.rs`**: Basic transposition table for caching board evaluations (active)
- **`enhanced_transposition.rs`**: Enhanced caching with depth bounds and node types (dormant)
- **`mod.rs`**: Public interface - exports cache stats, clear functions, and table types

## Key Features

### 1. Modular Architecture
- Clean separation of concerns (game logic, AI, caching)
- Easy to maintain and extend
- Clear public interfaces via `mod.rs` files
- Multiple algorithm implementations available

### 2. Advanced AI Algorithm
- **Expectimax search** with alpha-beta pruning
- **Adaptive depth** - adjusts search depth based on game state (3-9 levels)
- **Move ordering** - evaluates moves by quality before deep search
- **Strategic chance nodes** - only considers important empty cell positions
- **Early termination** - stops searching when dominant move found
- **Transposition table** - caches evaluated positions to avoid recomputation

### 3. Performance Optimizations
- **Bitmask-based empty cell tracking** - fast empty cell detection
- **Cached max tile calculation** - avoids repeated scans
- **Efficient board hashing** - fast position lookups
- **Memory management** - automatic cache clearing when too large
- **Move ordering** - improves alpha-beta pruning efficiency by 50%+

### 4. Multiple Evaluation Strategies
- **Optimized evaluation** (active) - score-focused with adaptive weights
- **Advanced evaluation** (available) - trap detection and gradient scoring
- **Basic evaluation** (available) - simple heuristics

## Usage Examples

### Basic Game Usage
```rust
use twenty_forty_eight::{GameBoard, Direction};

let mut game = GameBoard::new();
game.move_tiles(Direction::Right);
game.add_random_tile_self();
println!("Score: {}, Max tile: {}", game.get_score(), game.get_max_tile());
```

### AI Solver Usage
```rust
use twenty_forty_eight::GameBoard;

let mut game = GameBoard::new();
while !game.is_game_over() {
    if let Some(best_move) = game.find_best_move() {
        game.move_tiles(best_move);
        game.add_random_tile_self();
    }
}
println!("Final score: {}, Highest tile: {}", game.get_score(), game.get_max_tile());
```

### Cache Statistics
```rust
use twenty_forty_eight::{get_cache_stats, clear_cache};

let (hits, misses, size) = get_cache_stats();
println!("Cache: {} hits, {} misses, {} entries", hits, misses, size);

// Clear cache if it gets too large
if size > 1_000_000 {
    clear_cache();
}
```

## Building and Running

### Build the project
```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized runtime) - RECOMMENDED
cargo build --release
```

### Run the AI solver
```bash
# Debug mode
cargo run

# Release mode (much faster - recommended)
cargo run --release
```

### Run the example
```bash
cargo run --example cli_game
```

### Run tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Performance

The AI solver typically achieves:
- **Highest tile**: 1024+ consistently, often reaches 2048+
- **Search depth**: 4-9 levels (adaptive based on game state)
- **Cache efficiency**: 20-50% hit rate
- **Move speed**: ~1000-5000+ moves per game
- **Evaluation speed**: <1ms per board evaluation (optimized)

### Performance Tips
- Always use `--release` flag for running the solver (10-100x faster)
- Cache is automatically cleared when it exceeds 1M entries
- Early game uses deeper search (more empty cells = more possibilities)
- Late game uses shallower search (fewer empty cells = faster decisions)

## Dependencies

### Runtime Dependencies
- **`rand = "0.8"`**: Random number generation for tile placement
- **`lazy_static = "1.4"`**: Global static initialization for transposition tables

### Dev Dependencies
- **`criterion = "0.5"`**: Benchmarking framework (not currently used)

### Standard Library APIs Used
- `std::collections::HashMap` - Transposition table storage
- `std::sync::Mutex` - Thread-safe cache access
- `std::time::{Instant, Duration}` - Time-bounded search (iterative deepening)

## Algorithm Details

### Expectimax Search
The solver uses an **Expectimax** algorithm (variant of minimax for games with randomness):

1. **MAX nodes** (player's turn): Chooses the move with maximum expected score
2. **CHANCE nodes** (random tile placement): Calculates weighted average of all possible tile spawns
3. **Alpha-Beta Pruning**: Prunes branches that can't affect the final decision
4. **Transposition Table**: Caches evaluated positions to avoid recomputation

### Step-by-Step Move Selection Process

For each move, the AI follows this process:

1. **Calculate Adaptive Depth** (3-9 levels)
   - Early game (many empty cells): Deeper search (7-9 levels)
   - Late game (few empty cells): Shallower search (4-6 levels)
   - High tiles (512+): Additional depth bonus

2. **Order Moves by Quality**
   - Quick evaluation of all 4 directions
   - Scores based on: merges created, corner position, monotonicity, empty cells
   - Sorts moves best-to-worst for optimal alpha-beta pruning

3. **Deep Search (Expectimax)**
   - For each move (in quality order):
     - Simulate the move
     - Check transposition table (cache lookup)
     - If cached: return cached score immediately
     - If not cached: recursively evaluate resulting position
   - MAX nodes: Choose maximum score among moves
   - CHANCE nodes: Weighted average (90% chance of 2, 10% chance of 4)
   - Only considers strategic empty cell positions (corners, edges, near max tile)

4. **Store Results in Cache**
   - After evaluating a position, store hash → score mapping
   - Future identical positions use cached value

5. **Select Best Move**
   - Return the direction with highest expected score

### Evaluation Heuristics
The board evaluation considers:
- **Monotonicity**: Tiles arranged in increasing/decreasing order
- **Smoothness**: Similar tiles adjacent to each other
- **Empty cells**: More empty = more flexibility
- **Corner bonus**: Max tile in corner = huge bonus
- **Merge potential**: Adjacent tiles of same value
- **Position score**: Snake pattern (high tiles in top-left)
- **Score bonus**: Potential for creating high-scoring merges
- **Chain bonus**: Sequences like 2→4→8→16 that can chain merge
- **Edge control**: High tiles on edges help maintain structure

### Transposition Table (Caching)

The transposition table is a critical performance optimization that caches previously evaluated board positions.

#### How It Works

1. **Board Hashing**
   - Each board position is converted to a unique 64-bit hash
   - Uses tile values (log₂) and positions
   - Fast hash computation for lookups

2. **Cache Lookup (Before Evaluation)**
   ```rust
   let hash = board.board_hash();
   if let Some(cached_score) = TRANSPOSITION_TABLE.get(&hash) {
       return cached_score;  // Skip evaluation!
   }
   ```

3. **Cache Storage (After Evaluation)**
   ```rust
   TRANSPOSITION_TABLE.insert(hash, score);
   ```

#### Benefits

- **20-50% cache hit rate** in typical games
- **Significant speedup**: Avoids recomputing identical positions
- **Memory efficient**: Auto-clears when exceeding 1M entries
- **Thread-safe**: Uses Mutex for concurrent access

#### Example

```
Without Cache:
- Evaluates 1,000,000 positions
- Time: 10 seconds

With Cache (30% hit rate):
- Evaluates 700,000 positions (300,000 from cache)
- Time: ~7 seconds (30% faster!)
```

#### Cache Statistics

You can monitor cache performance:
```rust
let (hits, misses, size) = get_cache_stats();
let hit_rate = hits as f64 / (hits + misses) as f64 * 100.0;
println!("Cache: {} hits, {} misses, {:.1}% hit rate, {} entries", 
         hits, misses, hit_rate, size);
```

#### Memory Management

- Cache automatically cleared every 200 moves if size > 1,000,000 entries
- Prevents unbounded memory growth
- Statistics reset on clear

## Testing

The project includes **30 comprehensive unit tests** covering:

- **Game Logic**: Board operations, move validation, tile merging, game over detection
- **AI Components**: Move ordering, evaluation functions, depth calculation, complexity analysis
- **Cache System**: Transposition table operations, cache statistics
- **Bitboard**: Alternative representation (if used)

### Running Tests

```bash
# Run all tests
cargo test

# Run with output (see println! statements)
cargo test -- --nocapture

# Run specific test module
cargo test game::board

# Run specific test
cargo test test_move_ordering

# Run in release mode (faster)
cargo test --release
```

### Test Coverage

- All 30 tests passing
- Game logic fully tested
- AI components validated
- Cache operations verified

## Troubleshooting

### Common Issues

**Problem**: Compilation error about missing module
- **Solution**: Ensure all modules are declared in `mod.rs` files

**Problem**: Cache growing too large
- **Solution**: Cache auto-clears at 1M entries, or manually call `clear_cache()`

**Problem**: AI seems slow
- **Solution**: Always use `cargo run --release` (10-100x faster than debug mode)

**Problem**: Tests failing
- **Solution**: Run `cargo test` to see specific failures, check test expectations match implementation

## Contributing

When adding new features:
1. Follow the modular structure
2. Add appropriate documentation
3. Update the relevant module's `mod.rs` to export new items
4. Add tests for new functionality
5. Update this documentation if needed
6. Run `cargo test` to ensure all tests pass

### Code Organization
- **Active code**: Used in main execution path (`solver.rs`, `adaptive_search.rs`, etc.)
- **Dormant code**: Available but not used (`iterative_deepening.rs`, `advanced_evaluation.rs`)
- Keep dormant code if it provides alternative implementations or future features

### Adding New Features

1. **New AI Algorithm**: Add to `src/ai/` and update `src/ai/mod.rs`
2. **New Evaluation**: Add to `src/ai/` and export via `mod.rs`
3. **New Game Feature**: Add to `src/game/board.rs` or create new module
4. **New Binary**: Add to `src/bin/` and update `Cargo.toml` if needed

