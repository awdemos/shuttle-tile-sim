# shuttle-tile-sim

A Rust simulation engine for Space Shuttle thermal protection tile manufacturing, identification, traceability, and testing.

## Project layout

```
src/
├── lib.rs              # Core type definitions and module exports
├── main.rs             # CLI entry point
├── materials.rs        # Tile material types, coatings, strain isolation pads, adhesives
├── geometry.rs         # Tile dimensions and machining tolerances
├── identification.rs   # Tile ID parser, validator, and checksum
├── bonding.rs          # SIP, adhesive, and bond strength
├── audit.rs            # ID audit engine and batch tracking
├── simulation.rs       # Thermal physics model with degradation tracking
├── testing.rs          # Quality test suite (thermal, mechanical, adhesion, visual, dimensional)
├── manufacturing.rs    # Full manufacturing pipeline
└── report.rs           # Text and JSON report generation
```

## Setup commands

```bash
# Build the release binary
cargo build --release

# Run the test suite
cargo test
```

## Build/test/lint commands

```bash
# Lint with clippy
cargo clippy --all-targets

# Format code
cargo fmt --all

# Run unit and integration tests
cargo test

# Build release binary
./target/release/shuttle-tile-sim --help
```

## CLI usage

```bash
# Create a tile ID
./target/release/shuttle-tile-sim create --orbiter OV-103 --surface L --panel 12 --subpanel A

# Run full manufacturing pipeline
./target/release/shuttle-tile-sim manufacture --operator "NASA KSC"

# Run all tests for 3 thermal cycles
./target/release/shuttle-tile-sim test --cycles 3

# Run thermal simulation for 5 cycles
./target/release/shuttle-tile-sim simulate --cycles 5

# Generate a text report
./target/release/shuttle-tile-sim report --format text

# Validate a tile ID
./target/release/shuttle-tile-sim validate-id OV-103-L-12-A-001

# Full ID audit
./target/release/shuttle-tile-sim audit-id OV-103-L-12-A-001
```

## Key conventions

- Rust edition 2021.
- Uses `clap` for derive-based CLI parsing, `serde`/`serde_json` for serialization, `thiserror` for errors, `rand` with `small_rng`, and `chrono` with `serde` support.
- Tile IDs follow the orbiter-surface-panel-subpanel-serial schema (e.g. `OV-103-L-12-A-001`).
- Tests are 225 total (216 unit + 9 integration) covering validation, manufacturing, test suites, simulation physics, and report generation.
- Code style follows `cargo fmt`; clippy is the linter.

## Gotchas

- No deployment automation; build and run locally with `cargo`.
- The simulation uses deterministic degradation models over thermal cycles; cycle counts affect pass/fail thresholds in `test` and `simulate`.
- Report format must be one of the values supported by `report.rs` (`text` or `json`).
