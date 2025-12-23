//! Dummy node insertion for long edges.
//!
//! When an edge spans multiple layers, we insert dummy nodes to ensure
//! that every edge only connects adjacent layers. This enables proper
//! crossing minimization and orthogonal routing.

use super::LayeredGraph;

/// Insert dummy nodes for edges that span more than one layer.
///
/// For an edge from layer L to layer L+N where N > 1, we:
/// 1. Remove the original edge
/// 2. Insert N-1 dummy nodes in layers L+1, L+2, ..., L+N-1
/// 3. Create a chain of edges through the dummy nodes
pub fn insert_dummy_nodes(graph: &mut LayeredGraph) {
    // Collect edges that need splitting (can't modify while iterating)
    let edges_to_split: Vec<(usize, usize, usize, usize)> = graph
        .edges
        .iter()
        .enumerate()
        .filter_map(|(edge_idx, edge)| {
            let source_layer = graph.nodes[edge.source].layer;
            let target_layer = graph.nodes[edge.target].layer;

            // Only split if edge spans more than one layer
            if target_layer > source_layer + 1 {
                Some((edge_idx, edge.source, edge.target, edge.original_index))
            } else {
                None
            }
        })
        .collect();

    // Process each edge that needs splitting
    // We need to be careful about edge indices changing, so process in reverse
    for (_, source, target, original_index) in edges_to_split.into_iter().rev() {
        let source_layer = graph.nodes[source].layer;
        let target_layer = graph.nodes[target].layer;

        // Remove the original long edge
        graph.remove_edge(source, target);

        // Create chain of dummy nodes
        let mut prev_node = source;
        for layer in (source_layer + 1)..target_layer {
            let dummy_idx = graph.add_dummy_node(layer, original_index);
            graph.add_edge(prev_node, dummy_idx, original_index);
            prev_node = dummy_idx;
        }

        // Connect last dummy to target
        graph.add_edge(prev_node, target, original_index);
    }

    // Rebuild layers to include dummy nodes
    graph.build_layers();
}

/// Remove dummy nodes and restore original edges.
///
/// This is used after positioning to get clean output.
/// Note: The graph structure maintains original_edges, so this
/// is primarily for cleaning up the internal representation.
pub fn remove_dummy_nodes(graph: &mut LayeredGraph) {
    // Keep only non-dummy nodes
    let dummy_indices: Vec<usize> = graph
        .nodes
        .iter()
        .filter(|n| n.is_dummy)
        .map(|n| n.index)
        .collect();

    // Remove edges involving dummy nodes
    graph
        .edges
        .retain(|e| !dummy_indices.contains(&e.source) && !dummy_indices.contains(&e.target));

    // Remove dummy nodes (keeping indices stable is tricky, so we rebuild)
    graph.nodes.retain(|n| !n.is_dummy);

    // Reindex nodes
    for (new_idx, node) in graph.nodes.iter_mut().enumerate() {
        node.index = new_idx;
    }

    // Rebuild id_to_index
    graph.id_to_index.clear();
    for node in &graph.nodes {
        graph.id_to_index.insert(node.id.clone(), node.index);
    }

    // Rebuild layers
    graph.build_layers();
}

/// Count the number of dummy nodes in the graph.
pub fn count_dummy_nodes(graph: &LayeredGraph) -> usize {
    graph.nodes.iter().filter(|n| n.is_dummy).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sugiyama::{ranking, LayoutEdge};

    #[test]
    fn test_no_dummies_for_adjacent_layers() {
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![LayoutEdge {
            source: "a".to_string(),
            target: "b".to_string(),
        }];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        ranking::assign_layers(&mut graph);
        insert_dummy_nodes(&mut graph);

        // No dummies needed for adjacent layers
        assert_eq!(count_dummy_nodes(&graph), 0);
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_one_dummy_for_two_layer_gap() {
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
            LayoutEdge {
                source: "a".to_string(),
                target: "c".to_string(),
            }, // This spans 2 layers
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        ranking::assign_layers(&mut graph);

        // Before: a(0) -> c(2) spans 2 layers
        assert_eq!(graph.nodes[0].layer, 0); // a
        assert_eq!(graph.nodes[2].layer, 2); // c

        insert_dummy_nodes(&mut graph);

        // After: should have 1 dummy node
        assert_eq!(count_dummy_nodes(&graph), 1);
        assert_eq!(graph.node_count(), 4);

        // Find the dummy
        let dummy = graph.nodes.iter().find(|n| n.is_dummy).unwrap();
        assert_eq!(dummy.layer, 1);
        assert_eq!(dummy.original_edge_index, Some(2));
    }

    #[test]
    fn test_multiple_dummies_for_long_edge() {
        // Create a graph where a -> d spans 3 layers
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
                source: "b".to_string(),
                target: "c".to_string(),
            },
            LayoutEdge {
                source: "c".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "a".to_string(),
                target: "d".to_string(),
            }, // This spans 3 layers
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        ranking::assign_layers(&mut graph);

        // a at layer 0, d at layer 3
        assert_eq!(graph.nodes[0].layer, 0);
        assert_eq!(graph.nodes[3].layer, 3);

        insert_dummy_nodes(&mut graph);

        // Should have 2 dummy nodes (for layers 1 and 2)
        assert_eq!(count_dummy_nodes(&graph), 2);

        // Check dummy layers
        let dummies: Vec<_> = graph.nodes.iter().filter(|n| n.is_dummy).collect();
        let dummy_layers: Vec<_> = dummies.iter().map(|d| d.layer).collect();
        assert!(dummy_layers.contains(&1));
        assert!(dummy_layers.contains(&2));
    }

    #[test]
    fn test_dummy_edges_chain() {
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
            LayoutEdge {
                source: "a".to_string(),
                target: "c".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        ranking::assign_layers(&mut graph);
        insert_dummy_nodes(&mut graph);

        // Original edges: a->b, b->c (both adjacent)
        // Split edge: a->dummy, dummy->c
        // Total: 4 edges
        assert_eq!(graph.edges.len(), 4);

        // Find dummy
        let dummy_idx = graph
            .nodes
            .iter()
            .find(|n| n.is_dummy)
            .map(|n| n.index)
            .unwrap();

        // Check edge chain exists
        let has_a_to_dummy = graph.edges.iter().any(|e| e.source == 0 && e.target == dummy_idx);
        let has_dummy_to_c = graph.edges.iter().any(|e| e.source == dummy_idx && e.target == 2);

        assert!(has_a_to_dummy, "Should have edge from a to dummy");
        assert!(has_dummy_to_c, "Should have edge from dummy to c");
    }

    #[test]
    fn test_layers_include_dummies() {
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
            LayoutEdge {
                source: "a".to_string(),
                target: "c".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        ranking::assign_layers(&mut graph);
        insert_dummy_nodes(&mut graph);

        // Layer 0: a
        // Layer 1: b, dummy
        // Layer 2: c
        assert_eq!(graph.layers.len(), 3);
        assert_eq!(graph.layers[0].len(), 1); // a
        assert_eq!(graph.layers[1].len(), 2); // b and dummy
        assert_eq!(graph.layers[2].len(), 1); // c
    }
}
