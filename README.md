# mjlog-parser

A high-performance parser for Tenhou (天鳳) mahjong game logs written in Rust.

## Features

- Parse Tenhou mjlog files (both `.xml` and `.mjlog` gzipped formats)
- Convert Shift_JIS encoded XML to structured JSON
- Support for all game events (draws, discards, calls, wins, draws)
- Type-safe data structures with serde
- CLI interface with flexible output options
- Library API for integration into other projects

## Installation

### From source

```bash
git clone https://github.com/yourusername/mjlog-parser
cd mjlog-parser
cargo build --release
```

## Usage

### CLI

```bash
# Parse and output to stdout
mjlog-parser input.mjlog --stream

# Parse and save to file
mjlog-parser input.mjlog -o output.json

# Verbose mode
mjlog-parser input.mjlog --stream --verbose
```

### Library

```rust
use mjlog_parser::{parse_file, ParserOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ParserOptions {
        verbose: false,
    };
    
    parse_file("game.mjlog", "output.json", &options)?;
    println!("Parsed mjlog file to output.json");
    
    Ok(())
}
```

## Output Format

The parser outputs JSON with the following structure:

```json
{
  "mjlogVersion": "2.3",
  "gameId": "unique-game-id",
  "rules": {
    "typeFlags": 169,
    "lobbyId": null
  },
  "players": [
    {
      "seat": 0,
      "playerId": "player1",
      "rank": 5,
      "rate": 1800,
      "gender": "M"
    }
  ],
  "rounds": [
    {
      "roundId": "Round 1",
      "dealerSeat": 0,
      "init": {
        "roundNumber": 0,
        "honba": 0,
        "kyoutaku": 0,
        "dice": [3, 4],
        "doraIndicator": 52,
        "initialScores": [250, 250, 250, 250],
        "initialHands": [...]
      },
      "events": [...]
    }
  ]
}
```

## Development

### Running tests

```bash
cargo test
```

### Code coverage

Generate coverage report:

```bash
# Install tarpaulin (one time)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html --output-dir ./coverage

# Or use the convenience script
./scripts/coverage.sh
```

Current coverage: **90.96%** (comprehensive test coverage across all modules)

### Code quality checks

```bash
cargo clippy -- -D warnings
cargo fmt -- --check
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.