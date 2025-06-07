# twenty-forty-eight

An AI implementation to play and solve the 2048 game.

## Features

- **AI-Powered Gameplay**: Uses the Expectimax algorithm with adaptive depth searching
- **Parallel Processing**: Utilizes Rayon for parallel move evaluation
- **Smart Board Evaluation**: Implements multiple heuristics for optimal move selection
- **Memory Efficient**: Uses transposition tables with periodic cache clearing
- **Performance Optimized**: Written in Rust for high performance

## Requirements

- Rust (latest stable version)
- Cargo (Rust's package manager)

## Dependencies

- `rand = "0.8"` - For random number generation
- `rayon = "1.7"` - For parallel processing
- `lazy_static = "1.4"` - For static initialization

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/twenty-forty-eight.git
cd twenty-forty-eight
```

2. Build the project:
```bash
cargo build --release
```

3. Run the game:
```bash
cargo run --release
```

## How It Works

### AI Strategy

1. **Expectimax Algorithm**
   - Combines elements of minimax and expectimax
   - Adapts search depth based on game state
   - Uses parallel processing for move evaluation

2. **Board Evaluation**
   - Monotonicity: Measures tile arrangement patterns
   - Position weights: Favors corners and edges
   - Smoothness: Evaluates adjacent tile relationships
   - Empty cells: Rewards available space
   - Edge preference: Bonus for strategic tile placement
   - Merge potential: Evaluates merging opportunities

3. **Performance Optimizations**
   - Transposition table for caching board evaluations
   - Parallel processing for move evaluation
   - Adaptive search depth
   - Periodic cache clearing

### Game Rules

- The game is played on a 4x4 grid
- Tiles can be moved in four directions: Up, Down, Left, Right
- When two tiles with the same number collide, they merge into one tile with the sum of their values
- After each move, a new tile appears (90% chance of 2, 10% chance of 4)
- The game is won by creating a 2048 tile
- The game ends when no more valid moves are possible

## Project Structure

```
twenty-forty-eight/
├── src/
│   └── main.rs      # Main game implementation
├── Cargo.toml       # Project dependencies
├── Cargo.lock       # Dependency lock file
└── README.md        # This file
```

## Performance

The implementation is optimized for performance:
- Uses parallel processing for move evaluation
- Implements caching to avoid redundant calculations
- Adapts search depth based on game state
- Efficient memory management with periodic cache clearing


