# Getting Started — ternary-mesh

> *Estimated time: 5 minutes*

## Prerequisites

- **Rust 1.75+** (MSRV)
- Cargo (included with Rust)

## Installation

```toml
[dependencies]
ternary_mesh = "0.1.0"
```

Or from source:

```bash
git clone https://github.com/SuperInstance/ternary-mesh.git
cd ternary-mesh
cargo build --release
cargo test
```

## Core Concept

This crate implements ternary {-1, 0, +1} semantics for `mesh`.
The ternary principle: **0 is not nothing** — it is a meaningful neutral state.

## Quick Example

```
use ternary_mesh::NodeId;
let instance = NodeId::new();
```

## Running Tests

```bash
cargo test
```

## Next Steps

- [ARCHITECTURE.md](./ARCHITECTURE.md) — Internal design
- [PLUG_AND_PLAY.md](./PLUG_AND_PLAY.md) — Integration
- [CONTRIBUTING.md](./CONTRIBUTING.md) — Contributing
