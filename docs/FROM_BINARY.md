# From Binary to Ternary: Dynamic Mesh Networking

## The Trap

Binary mesh networking treats every connection as either present or absent. Is agent A connected to agent B? Boolean. Is the connection good? Boolean (usually derived from latency or packet loss with a hard threshold). This creates a brittle network: connections flap between "up" and "down" at the slightest perturbation, and routing algorithms oscillate as they chase binary state changes.

The real world of agent-to-agent connections isn't binary. A connection can be degraded, congested, high-latency but functional, or perfectly healthy. Binary networking forces you to pick a threshold and call everything above it "up" and everything below it "down," losing all the signal in between.

## Map to Three States

| Domain | −1 | 0 | +1 |
|--------|----|---|-----|
| Connection quality | unhealthy | degraded | healthy |
| Routing verdict | blocked | pending | routable |
| Gossip trust | distrust | unknown | trusted |
| Healer action | evict | monitor | repair |

## From Binary to Ternary

**Before: binary edge weights**

```rust
struct Edge {
    connected: bool,  // alive or dead
    // What about "almost dead"?
    // What about "reconnecting"?
}
```

Every real mesh network has connections in a grey zone. A node that drops 30% of packets isn't "dead" — it's degraded. Binary thinking would disconnect it, possibly isolating healthy nodes that route through it.

**After: ternary edge weights**

```rust
struct MeshEdge {
    weight: Trit,  // -1 = unhealthy, 0 = degraded, +1 = healthy
}
```

The `MeshRouter` uses these ternary weights in pathfinding. A path through a degraded (0) node isn't rejected — it's marked as pending. If the primary healthy path fails, the degraded path is already computed.

**0 is not nothing:** In a binary mesh, a node that doesn't respond to one ping is marked "down" and removed from routing tables. In a ternary mesh, it's marked `0` — degraded. The router doesn't send primary traffic through it, but it keeps it in the routing table. This avoids the flapping cascade where a brief network hiccup forces every node to recompute paths.

**The ternary conservation law** applies to mesh health: in a closed group of agents, the sum of connection weights is invariant under certain operations. When one connection degrades, another must improve. The `MeshHealer` leverages this: it identifies the weakest links and negotiates repairs, knowing that healing one edge necessarily degrades another somewhere in the system.

```rust
// Gossip with ternary trust
// Binary: gossip or don't gossip
// Ternary: -1 = malicious, 0 = unknown (pass-through), +1 = trusted
// The MeshGossip layer doesn't forward messages from -1 nodes
// It passes through messages from 0 nodes without endorsing them
// It amplifies messages from +1 nodes
```

## Why It Matters

Ternary mesh networking eliminates flapping, preserves routing alternatives during degradation, and models connection quality honestly. The "degraded" state isn't a failure mode — it's an active, useful state that keeps the network stable when conditions aren't perfect. Real networks are rarely binary; ternary routing is just being honest about it.
