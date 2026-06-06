# PLUG_AND_PLAY — Mesh

> Peer-to-peer mesh network with ternary-weighted routing

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ternary-mesh = { git = "https://github.com/SuperInstance/ternary-mesh" }
```

Use in your code:

```rust
use ternary_mesh::{MeshNode, Ternary};

let mut node = MeshNode::new("node-1");
node.add_peer("node-2", Ternary::Pos);
```

## 📚 Available Documentation

| Document | Description |
|----------|-------------|
| `docs/FROM_BINARY.md` | Understanding ternary concepts as a binary programmer |
| `docs/MIGRATION.md` | Version migration guide |
| `docs/FUTURE-INTEGRATION.md` | Planned features and roadmap |

## 🔗 Integration

This crate is part of the [SuperInstance ternary fleet](https://github.com/SuperInstance). It uses the canonical `Ternary` type from `ternary-types` for cross-crate compatibility.

## 📄 License

MIT
