//! Layer (rank) assignment for Sugiyama layout.
//!
//! This module assigns each node to a layer based on the graph structure.
//! The algorithm ensures that for every edge (u, v), layer(u) < layer(v).

use super::LayeredGraph;

/// Assign layers to all nodes using longest path algorithm.
///
/// This ensures that edges point downward (from lower to higher layers).
pub fn assign_layers(graph: &mut LayeredGraph) {
    let n = graph.node_count();
    if n == 0 {
        return;
    }

    // Initialize all nodes to layer 0
    for node in &mut graph.nodes {
        node.layer = 0;
    }

    // Iteratively push nodes to higher layers based on edges
    let mut changed = true;
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 1000;

    while changed && iterations < MAX_ITERATIONS {
        changed = false;
        iterations += 1;

        for edge in &graph.edges {
            let source_layer = graph.nodes[edge.source].layer;
            let target_layer = graph.nodes[edge.target].layer;

            // Target should be at least one layer below source
            if target_layer <= source_layer {
                graph.nodes[edge.target].layer = source_layer + 1;
                changed = true;
            }
        }
    }

    // Build the layers structure
    graph.build_layers();
}

/// Compact layers to minimize total edge length.
///
/// This optional optimization pulls nodes closer to their predecessors
/// when possible without violating layer constraints.
pub fn compact_layers(graph: &mut LayeredGraph) {
    let n = graph.node_count();
    if n == 0 {
        return;
    }

    // For each node (in reverse topological order), try to move it
    // as close as possible to its successors
    let max_layer = graph.max_layer();

    for layer in (0..max_layer).rev() {
        let nodes_in_layer: Vec<usize> = graph
            .nodes
            .iter()
            .filter(|n| n.layer == layer)
            .map(|n| n.index)
            .collect();

        for node_idx in nodes_in_layer {
            // Find minimum layer of successors
            let successors = graph.successors(node_idx);
            if successors.is_empty() {
                continue;
            }

            let min_successor_layer = successors
                .iter()
                .map(|&s| graph.nodes[s].layer)
                .min()
                .unwrap_or(layer + 1);

            // Find maximum layer of predecessors
            let predecessors = graph.predecessors(node_idx);
            let max_predecessor_layer = predecessors
                .iter()
                .map(|&p| graph.nodes[p].layer)
                .max()
                .unwrap_or(0);

            // Place node as close to successors as possible
            // while staying above predecessors
            let new_layer = (min_successor_layer - 1).max(max_predecessor_layer + 1);

            if new_layer > graph.nodes[node_idx].layer {
                graph.nodes[node_idx].layer = new_layer;
            }
        }
    }

    // Rebuild layers structure
    graph.build_layers();
}

/// Normalize layers to start from 0 with no gaps.
pub fn normalize_layers(graph: &mut LayeredGraph) {
    if graph.node_count() == 0 {
        return;
    }

    // Find all used layers
    let mut used_layers: Vec<usize> = graph.nodes.iter().map(|n| n.layer).collect();
    used_layers.sort();
    used_layers.dedup();

    // Create mapping from old layer to new layer
    let layer_map: std::collections::HashMap<usize, usize> = used_layers
        .iter()
        .enumerate()
        .map(|(new, &old)| (old, new))
        .collect();

    // Apply mapping
    for node in &mut graph.nodes {
        if let Some(&new_layer) = layer_map.get(&node.layer) {
            node.layer = new_layer;
        }
    }

    // Rebuild layers
    graph.build_layers();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sugiyama::LayoutEdge;

    #[test]
    fn test_assign_layers_linear() {
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "b".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "c".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        assign_layers(&mut graph);

        assert_eq!(graph.nodes[0].layer, 0); // a
        assert_eq!(graph.nodes[1].layer, 1); // b
        assert_eq!(graph.nodes[2].layer, 2); // c
    }

    #[test]
    fn test_assign_layers_diamond() {
        // a -> b, a -> c, b -> d, c -> d
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "b".to_string(),
            },
            LayoutEdge {
                source: "a".to_string(),
                target: "c".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "c".to_string(),
                target: "d".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        assign_layers(&mut graph);

        assert_eq!(graph.nodes[0].layer, 0); // a at top
        assert_eq!(graph.nodes[1].layer, 1); // b in middle
        assert_eq!(graph.nodes[2].layer, 1); // c in middle
        assert_eq!(graph.nodes[3].layer, 2); // d at bottom
    }

    #[test]
    fn test_assign_layers_disconnected() {
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![LayoutEdge {
            source: "a".to_string(),
            target: "b".to_string(),
        }];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        assign_layers(&mut graph);

        assert_eq!(graph.nodes[0].layer, 0); // a
        assert_eq!(graph.nodes[1].layer, 1); // b
        assert_eq!(graph.nodes[2].layer, 0); // c (disconnected, stays at 0)
    }

    #[test]
    fn test_layers_structure() {
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "b".to_string(),
            },
            LayoutEdge {
                source: "a".to_string(),
                target: "c".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "c".to_string(),
                target: "d".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        assign_layers(&mut graph);

        assert_eq!(graph.layers.len(), 3);
        assert_eq!(graph.layers[0].len(), 1); // a
        assert_eq!(graph.layers[1].len(), 2); // b, c
        assert_eq!(graph.layers[2].len(), 1); // d
    }

    #[test]
    fn test_normalize_layers() {
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);

        // Manually set non-contiguous layers
        graph.nodes[0].layer = 0;
        graph.nodes[1].layer = 5;
        graph.build_layers();

        normalize_layers(&mut graph);

        assert_eq!(graph.nodes[0].layer, 0);
        assert_eq!(graph.nodes[1].layer, 1);
    }
}
