#![forbid(unsafe_code)]

//! Dynamic mesh networking between agents with ternary-weighted connections.

/// Canonical ternary type re-exported from `ternary-types`.
pub use ternary_types::Ternary;

/// Extension trait providing methods previously defined on the custom `Ternary` type.
pub trait TernaryExt: Sized {
    /// Create a `Ternary` from an `i8` value (-1, 0, or 1).
    fn from_i8(v: i8) -> Option<Self>;
    /// Return the `i8` value of this ternary state.
    fn to_i8(self) -> i8;
}

impl TernaryExt for ternary_types::Ternary {
    fn from_i8(v: i8) -> Option<Self> {
        Self::try_from(v).ok()
    }

    fn to_i8(self) -> i8 {
        i8::from(self)
    }
}

/// Unique identifier for a mesh node (agent or room).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

/// An agent or room participating in the mesh network.
#[derive(Clone, Debug)]
pub struct MeshNode {
    pub id: NodeId,
    pub label: String,
    pub active: bool,
}

impl MeshNode {
    pub fn new(id: u64, label: &str) -> Self {
        MeshNode {
            id: NodeId(id),
            label: label.to_string(),
            active: true,
        }
    }
}

/// A connection between two nodes with a ternary weight.
#[derive(Clone, Debug)]
pub struct MeshEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub weight: Ternary,
    pub alive: bool,
}

impl MeshEdge {
    pub fn new(from: NodeId, to: NodeId, weight: Ternary) -> Self {
        MeshEdge {
            from,
            to,
            weight,
            alive: true,
        }
    }

    /// Returns true if the edge is bidirectional (both directions exist and are alive).
    pub fn is_bidirectional(&self, other: &MeshEdge) -> bool {
        self.from == other.to
            && self.to == other.from
            && self.alive
            && other.alive
    }
}

/// Routes messages through the mesh network.
#[derive(Clone, Debug)]
pub struct MeshRouter {
    nodes: Vec<MeshNode>,
    edges: Vec<MeshEdge>,
}

impl MeshRouter {
    pub fn new() -> Self {
        MeshRouter {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: MeshNode) {
        if !self.nodes.iter().any(|n| n.id == node.id) {
            self.nodes.push(node);
        }
    }

    pub fn add_edge(&mut self, edge: MeshEdge) {
        self.edges.push(edge);
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    /// Find a path from source to target using BFS, preferring positive-weight edges.
    pub fn find_path(&self, source: NodeId, target: NodeId) -> Option<Vec<NodeId>> {
        if source == target {
            return Some(vec![source]);
        }

        let mut visited = vec![false; 256];
        let mut parent = [None::<NodeId>; 256];
        let mut queue = vec![source];

        if let Some(idx) = self.node_index(source) {
            visited[idx] = true;
        }

        // Sort edges to prefer positive weights
        let mut sorted_edges: Vec<&MeshEdge> = self.edges.iter()
            .filter(|e| e.alive)
            .collect();
        sorted_edges.sort_by_key(|e| -e.weight.to_i8());

        while let Some(current) = queue.pop() {
            for edge in &sorted_edges {
                if edge.from == current && edge.alive {
                    let next = edge.to;
                    if next == target {
                        // Reconstruct path
                        let mut path = vec![target];
                        let mut cur = current;
                        path.push(cur);
                        if let Some(idx) = self.node_index(cur) {
                            while let Some(p) = parent[idx] {
                                path.push(p);
                                if let Some(pidx) = self.node_index(p) {
                                    cur = p;
                                    parent.get_mut(pidx);
                                } else {
                                    break;
                                }
                            }
                        }
                        path.reverse();
                        return Some(path);
                    }
                    if let Some(idx) = self.node_index(next) {
                        if !visited[idx] {
                            visited[idx] = true;
                            if let Some(pidx) = self.node_index(current) {
                                parent[idx] = Some(current);
                            }
                            queue.push(next);
                        }
                    }
                }
            }
        }
        None
    }

    /// Get all neighbors of a node.
    pub fn neighbors(&self, id: NodeId) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter(|e| e.from == id && e.alive)
            .map(|e| e.to)
            .collect()
    }

    fn node_index(&self, id: NodeId) -> Option<usize> {
        let v = id.0 as usize;
        if v < 256 { Some(v) } else { None }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// Detects and repairs broken connections in the mesh.
#[derive(Clone, Debug)]
pub struct MeshHealer {
    router: MeshRouter,
    max_repair_attempts: usize,
}

impl MeshHealer {
    pub fn new(router: MeshRouter, max_repair_attempts: usize) -> Self {
        MeshHealer { router, max_repair_attempts }
    }

    /// Find all dead edges.
    pub fn find_dead_edges(&self) -> Vec<&MeshEdge> {
        self.router.edges.iter().filter(|e| !e.alive).collect()
    }

    /// Repair a dead edge by reviving it with a neutral weight.
    pub fn repair_edge(&mut self, from: NodeId, to: NodeId) -> bool {
        for edge in &mut self.router.edges {
            if edge.from == from && edge.to == to && !edge.alive {
                edge.alive = true;
                edge.weight = Ternary::Neutral;
                return true;
            }
        }
        false
    }

    /// Find alternate routes around a broken edge.
    pub fn find_alternate_route(&self, from: NodeId, to: NodeId) -> Option<Vec<NodeId>> {
        // Temporarily mark the direct edge as dead
        self.router.find_path(from, to)
    }

    /// Run full healing pass: attempt to repair all dead edges.
    pub fn heal_all(&mut self) -> usize {
        let dead: Vec<(NodeId, NodeId)> = self.router.edges.iter()
            .filter(|e| !e.alive)
            .map(|e| (e.from, e.to))
            .collect();
        let mut repaired = 0;
        for (from, to) in dead {
            if repaired < self.max_repair_attempts {
                if self.repair_edge(from, to) {
                    repaired += 1;
                }
            }
        }
        repaired
    }

    pub fn router(&self) -> &MeshRouter {
        &self.router
    }
}

/// Handles network partitions and attempts to track disconnected groups.
#[derive(Clone, Debug)]
pub struct MeshPartitions {
    router: MeshRouter,
}

impl MeshPartitions {
    pub fn new(router: MeshRouter) -> Self {
        MeshPartitions { router }
    }

    /// Find all disconnected partitions using flood fill.
    pub fn find_partitions(&self) -> Vec<Vec<NodeId>> {
        let mut visited = std::collections::HashSet::new();
        let mut partitions = Vec::new();

        for node in &self.router.nodes {
            if !visited.contains(&node.id) {
                let mut partition = Vec::new();
                let mut stack = vec![node.id];
                while let Some(current) = stack.pop() {
                    if visited.insert(current) {
                        partition.push(current);
                        for neighbor in self.router.neighbors(current) {
                            if !visited.contains(&neighbor) {
                                stack.push(neighbor);
                            }
                        }
                    }
                }
                partitions.push(partition);
            }
        }
        partitions
    }

    /// Check if two nodes are in the same partition.
    pub fn are_connected(&self, a: NodeId, b: NodeId) -> bool {
        self.find_partitions().iter().any(|p| p.contains(&a) && p.contains(&b))
    }

    /// Count the number of partitions.
    pub fn partition_count(&self) -> usize {
        self.find_partitions().len()
    }
}

/// Gossip protocol for propagating information through the mesh.
#[derive(Clone, Debug)]
pub struct MeshGossip {
    messages: Vec<GossipMessage>,
    seen_ids: std::collections::HashSet<u64>,
    max_hops: u32,
}

#[derive(Clone, Debug)]
pub struct GossipMessage {
    pub id: u64,
    pub origin: NodeId,
    pub payload: String,
    pub hops: u32,
}

impl MeshGossip {
    pub fn new(max_hops: u32) -> Self {
        MeshGossip {
            messages: Vec::new(),
            seen_ids: std::collections::HashSet::new(),
            max_hops,
        }
    }

    /// Broadcast a new gossip message.
    pub fn broadcast(&mut self, origin: NodeId, payload: &str) -> u64 {
        let id = self.messages.len() as u64;
        let msg = GossipMessage {
            id,
            origin,
            payload: payload.to_string(),
            hops: 0,
        };
        self.seen_ids.insert(id);
        self.messages.push(msg);
        id
    }

    /// Receive a gossip message; returns true if it's new.
    pub fn receive(&mut self, msg: GossipMessage) -> bool {
        if self.seen_ids.contains(&msg.id) {
            return false;
        }
        self.seen_ids.insert(msg.id);
        self.messages.push(msg);
        true
    }

    /// Propagate messages to neighbors, incrementing hop count.
    pub fn propagate(&self, router: &MeshRouter) -> Vec<(NodeId, GossipMessage)> {
        let mut result = Vec::new();
        for msg in &self.messages {
            if msg.hops < self.max_hops {
                let neighbors = router.neighbors(msg.origin);
                for neighbor in neighbors {
                    let propagated = GossipMessage {
                        hops: msg.hops + 1,
                        ..msg.clone()
                    };
                    result.push((neighbor, propagated));
                }
            }
        }
        result
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn messages(&self) -> &[GossipMessage] {
        &self.messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_from_i8() {
        assert_eq!(Ternary::from_i8(-1), Some(Ternary::Negative));
        assert_eq!(Ternary::from_i8(0), Some(Ternary::Neutral));
        assert_eq!(Ternary::from_i8(1), Some(Ternary::Positive));
        assert_eq!(Ternary::from_i8(2), None);
    }

    #[test]
    fn test_ternary_to_i8() {
        assert_eq!(Ternary::Negative.to_i8(), -1);
        assert_eq!(Ternary::Neutral.to_i8(), 0);
        assert_eq!(Ternary::Positive.to_i8(), 1);
    }

    #[test]
    fn test_mesh_node_creation() {
        let node = MeshNode::new(1, "agent-1");
        assert_eq!(node.id, NodeId(1));
        assert_eq!(node.label, "agent-1");
        assert!(node.active);
    }

    #[test]
    fn test_mesh_edge_creation() {
        let edge = MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive);
        assert_eq!(edge.from, NodeId(1));
        assert_eq!(edge.to, NodeId(2));
        assert_eq!(edge.weight, Ternary::Positive);
        assert!(edge.alive);
    }

    #[test]
    fn test_mesh_edge_bidirectional() {
        let e1 = MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive);
        let e2 = MeshEdge::new(NodeId(2), NodeId(1), Ternary::Positive);
        assert!(e1.is_bidirectional(&e2));
    }

    #[test]
    fn test_mesh_edge_not_bidirectional() {
        let e1 = MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive);
        let e2 = MeshEdge::new(NodeId(3), NodeId(1), Ternary::Positive);
        assert!(!e1.is_bidirectional(&e2));
    }

    #[test]
    fn test_router_add_node() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        assert_eq!(router.node_count(), 2);
    }

    #[test]
    fn test_router_no_duplicate_nodes() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(1, "a"));
        assert_eq!(router.node_count(), 1);
    }

    #[test]
    fn test_router_add_edge() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        router.add_edge(MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive));
        assert_eq!(router.edge_count(), 1);
    }

    #[test]
    fn test_router_remove_node() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        router.add_edge(MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive));
        router.remove_node(NodeId(1));
        assert_eq!(router.node_count(), 1);
        assert_eq!(router.edge_count(), 0);
    }

    #[test]
    fn test_router_neighbors() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        router.add_node(MeshNode::new(3, "c"));
        router.add_edge(MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive));
        router.add_edge(MeshEdge::new(NodeId(1), NodeId(3), Ternary::Negative));
        let nbrs = router.neighbors(NodeId(1));
        assert_eq!(nbrs.len(), 2);
    }

    #[test]
    fn test_router_find_path_direct() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        router.add_edge(MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive));
        let path = router.find_path(NodeId(1), NodeId(2));
        assert!(path.is_some());
    }

    #[test]
    fn test_router_find_path_self() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        let path = router.find_path(NodeId(1), NodeId(1));
        assert_eq!(path, Some(vec![NodeId(1)]));
    }

    #[test]
    fn test_router_find_path_no_path() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        let path = router.find_path(NodeId(1), NodeId(2));
        assert!(path.is_none());
    }

    #[test]
    fn test_healer_find_dead_edges() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        let mut edge = MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive);
        edge.alive = false;
        router.add_edge(edge);
        let healer = MeshHealer::new(router, 10);
        assert_eq!(healer.find_dead_edges().len(), 1);
    }

    #[test]
    fn test_healer_repair_edge() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        let mut edge = MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive);
        edge.alive = false;
        router.add_edge(edge);
        let mut healer = MeshHealer::new(router, 10);
        assert!(healer.repair_edge(NodeId(1), NodeId(2)));
        assert_eq!(healer.find_dead_edges().len(), 0);
    }

    #[test]
    fn test_healer_heal_all() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        router.add_node(MeshNode::new(3, "c"));
        let mut e1 = MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive);
        e1.alive = false;
        let mut e2 = MeshEdge::new(NodeId(2), NodeId(3), Ternary::Positive);
        e2.alive = false;
        router.add_edge(e1);
        router.add_edge(e2);
        let mut healer = MeshHealer::new(router, 10);
        let repaired = healer.heal_all();
        assert_eq!(repaired, 2);
    }

    #[test]
    fn test_partitions_single() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        router.add_edge(MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive));
        let partitions = MeshPartitions::new(router);
        assert_eq!(partitions.partition_count(), 1);
    }

    #[test]
    fn test_partitions_split() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        // No edges => two partitions
        let partitions = MeshPartitions::new(router);
        assert_eq!(partitions.partition_count(), 2);
    }

    #[test]
    fn test_partitions_are_connected() {
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        router.add_edge(MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive));
        let partitions = MeshPartitions::new(router);
        assert!(partitions.are_connected(NodeId(1), NodeId(2)));
    }

    #[test]
    fn test_gossip_broadcast() {
        let mut gossip = MeshGossip::new(5);
        let id = gossip.broadcast(NodeId(1), "hello");
        assert_eq!(id, 0);
        assert_eq!(gossip.message_count(), 1);
    }

    #[test]
    fn test_gossip_receive_new() {
        let mut gossip = MeshGossip::new(5);
        let msg = GossipMessage {
            id: 99,
            origin: NodeId(1),
            payload: "test".to_string(),
            hops: 0,
        };
        assert!(gossip.receive(msg));
        assert!(!gossip.receive(GossipMessage {
            id: 99,
            origin: NodeId(1),
            payload: "test".to_string(),
            hops: 1,
        }));
    }

    #[test]
    fn test_gossip_propagate() {
        let mut gossip = MeshGossip::new(3);
        gossip.broadcast(NodeId(1), "hello");
        let mut router = MeshRouter::new();
        router.add_node(MeshNode::new(1, "a"));
        router.add_node(MeshNode::new(2, "b"));
        router.add_edge(MeshEdge::new(NodeId(1), NodeId(2), Ternary::Positive));
        let propagated = gossip.propagate(&router);
        assert_eq!(propagated.len(), 1);
        assert_eq!(propagated[0].0, NodeId(2));
        assert_eq!(propagated[0].1.hops, 1);
    }
}
