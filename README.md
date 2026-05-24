# Shuttle Tile Sim

A Rust simulation engine for Space Shuttle thermal protection tile manufacturing, identification, traceability, and testing.

## What It Is

This project simulates the full lifecycle of a Space Shuttle thermal protection tile — from raw material selection through manufacturing, quality testing, thermal simulation, and final reporting. It models the physical properties, identification schema, and quality assurance processes used for the silica-based tiles that protected the orbiter during reentry.

## Features

- **Materials** — Tile material types (LI-900, LI-2200, FRCI-12, AETB-8), coatings (RCG, TUFI, TUFROC), strain isolation pads, and silicone adhesives with full property validation
- **Geometry** — Tile dimensions, machining tolerances, and dimensional verification
- **Identification** — Complete tile ID parser, validator, checksum generator, and encoding/decoding for the orbiter-surface-panel-subpanel numbering schema
- **Bonding** — Strain isolation pad application, adhesive curing, and bond strength calculation
- **Audit** — Full ID audit engine with duplicate detection, batch tracking, and compliance checks
- **Simulation** — Thermal physics model with conductivity, heat transfer, reentry thermal profiles, and degradation tracking over multiple cycles
- **Testing** — Five test types: thermal, mechanical, adhesion, visual, and dimensional — each with pass/fail thresholds
- **Manufacturing** — Complete pipeline from raw material to finished tile with stage tracking and component validation at each step
- **Reporting** — Structured text and JSON report generation with overall status determination

## Quick Start

```bash
cargo build --release

# Create a tile ID
./target/release/shuttle-tile-sim create --orbiter OV-103 --surface L --panel 12 --subpanel A

# Run full manufacturing pipeline
./target/release/shuttle-tile-sim manufacture --operator "NASA KSC"

# Run all tests
./target/release/shuttle-tile-sim test --cycles 3

# Run thermal simulation
./target/release/shuttle-tile-sim simulate --cycles 5

# Generate report
./target/release/shuttle-tile-sim report --format text

# Validate a tile ID
./target/release/shuttle-tile-sim validate-id OV-103-L-12-A-001

# Full ID audit
./target/release/shuttle-tile-sim audit-id OV-103-L-12-A-001
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `create` | Generate a tile ID from orbiter, surface, panel, and subpanel |
| `manufacture` | Run the full manufacturing pipeline |
| `test` | Execute quality tests (thermal, mechanical, adhesion, visual, dimensional) |
| `simulate` | Run thermal physics simulation with degradation tracking |
| `report` | Generate structured text or JSON report |
| `validate-id` | Validate a tile ID checksum and schema |
| `audit-id` | Perform full ID audit with duplicate detection |

## Project Structure

```
src/
├── lib.rs              # Core type definitions and module exports
├── materials.rs        # Material types and validation
├── geometry.rs         # Tile dimensions and tolerances
├── identification.rs   # ID parser, validator, and checksum
├── bonding.rs          # SIP, adhesive, and bond strength
├── audit.rs            # Audit engine and batch tracking
├── simulation.rs       # Thermal physics model
├── testing.rs          # Quality test suite
├── manufacturing.rs    # Full manufacturing pipeline
├── report.rs           # Report generation
└── main.rs             # CLI entry point
```

## Running Tests

```bash
cargo test
```

225 tests (216 unit + 9 integration) covering all validation logic, manufacturing stages, test suites, simulation physics, and report generation.

## License

MIT
