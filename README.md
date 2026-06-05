# ternary-mesh: Dynamic mesh networking between agents with ternary-weighted connections

## Why This Exists

In a fleet of agents moving between rooms, connectivity is fragile. Rooms come and go, agents relocate, and the network topology shifts constantly. Standard graph libraries assume a static structure; a fleet mesh needs to detect broken links, route around failures, and propagate information through whatever path is available — all while tracking whether each connection is healthy, degraded, or hostile.

Ternary weights (negative, neutral, positive) give each link a quality signal beyond "up or down," letting routers prefer reliable paths and healers prioritize repairing the worst connections first.

## Core Concepts

- **Ternary weight**: Each mesh edge carries a value of `Neg` (-1), `Zero` (0), or `Pos` (+1), representing connection quality from hostile to healthy.
- **MeshNode**: An agent or room participating in the mesh. Each has a unique `NodeId` and can be active or inactive.
- **MeshEdge**: A directed connection between two nodes with a ternary weight and an alive/dead flag.
- **MeshRouter**: Finds paths through the mesh using BFS, preferring positive-weight edges.
- **MeshHealer**: Detects dead edges and repairs them, with a configurable attempt limit.
- **MeshPartitions**: Identifies disconnected groups (partitions) when the mesh splits, using flood fill.
- **MeshGossip**: Epidemic-style information propagation with a configurable hop limit to prevent infinite forwarding.

## Quick Start

```toml
[dependencies]
ternary-mesh = "0.1"
```

```rust
use ternary_mesh::*;

let mut router = MeshRouter::new();
router.add_node(MeshNode::new(1, "agent-alpha"));
router.add_node(MeshNode::new(2, "room-bravo"));
router.add_node(MeshNode::new(3, "agent-charlie"));

router.add_edge(MeshEdge::new(NodeId(1), NodeId(2), Ternary::Pos));
router.add_edge(MeshEdge::new(NodeId(2), NodeId(3), Ternary::Pos));

if let Some(path) = router.find_path(NodeId(1), NodeId(3)) {
    println!("Route found with {} hops", path.len() - 1);
}
```

## API Overview

| Type | Description |
|------|-------------|
| `MeshNode` | Agent or room in the mesh, identified by `NodeId` |
| `MeshEdge` | Directed connection with ternary weight and alive flag |
| `MeshRouter` | BFS pathfinding that prefers positive-weight edges |
| `MeshHealer` | Detects and repairs dead edges up to a configurable limit |
| `MeshPartitions` | Finds disconnected groups via flood fill |
| `MeshGossip` | Hop-limited epidemic message propagation |
| `GossipMessage` | A single gossip payload with origin, hops counter, and ID |

## How It Works

Pathfinding uses breadth-first search over alive edges, sorted so positive-weight edges are explored first. This doesn't guarantee shortest-path optimality but biases routes toward healthy connections without requiring a full weighted shortest-path algorithm.

Partition detection runs flood fill from each unvisited node, collecting reachable nodes into groups. This is O(V + E) and can be expensive for large meshes; it's best run periodically rather than on every message.

Gossip propagation is epidemic: each node forwards messages to its neighbors, incrementing a hop counter. The `max_hops` parameter prevents messages from circulating forever. Duplicate detection uses a `seen_ids` set, so a node won't re-process a message it has already handled.

The healer iterates over dead edges and revives them with a neutral weight, up to `max_repair_attempts`. It doesn't validate whether a repaired connection actually works — that requires external health checks.

## Known Limitations

- Pathfinding uses BFS with a heuristic sort rather than Dijkstra; it may not find the truly optimal weighted path.
- `find_path` uses a fixed-size array (256 entries) for visited/parent tracking; node IDs above 255 will be ignored.
- Partition detection is O(V + E) and allocates a new `HashSet` per call; not suitable for per-message invocation on large meshes.
- Gossip doesn't support message expiry or TTL-based cleanup; `seen_ids` grows without bound.
- The healer revives edges with neutral weight regardless of previous quality; it has no memory of historical edge reliability.

## Use Cases

- **Fleet connectivity monitoring**: Track which agents can reach which rooms and detect when partitions form.
- **Message routing in dynamic topologies**: Route commands through the best-available path when rooms are joining and leaving.
- **Health propagation**: Use gossip to broadcast "room X is degraded" to the entire fleet without a central broker.
- **Self-healing networks**: Periodically run the healer to automatically restore connections after transient failures.

## Ecosystem Context

Part of the SuperInstance ternary crate family. Works alongside `ternary-channel` (communication primitives) and `ternary-beacon` (service discovery). The mesh provides the routing layer that higher-level fleet communication builds on.

## License

MIT

## See Also
- **ternary-network** — related
- **ternary-graph** — related
- **ternary-topology** — related
- **ternary-beacon** — related
- **ternary-channel** — related
- **ternary-bridge** — related

