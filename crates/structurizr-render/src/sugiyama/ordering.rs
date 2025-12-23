//! Crossing minimization for Sugiyama layout.
//!
//! This module implements the barycentric heuristic with 2-opt local refinement
//! to minimize edge crossings in the layered graph.

use super::LayeredGraph;

/// Minimize edge crossings using barycentric heuristic with 2-opt refinement.
///
/// The algorithm alternates between:
/// 1. Down sweep: Order each layer based on positions of predecessors
/// 2. Up sweep: Order each layer based on positions of successors
/// 3. 2-opt: Swap adjacent nodes if it reduces crossings
pub fn minimize_crossings(graph: &mut LayeredGraph, max_iterations: usize, local_refinement: bool) {
    if graph.layer_count() < 2 {
        return;
    }

    let mut best_crossings = count_total_crossings(graph);
    let mut best_order = save_layer_order(graph);

    for _iteration in 0..max_iterations {
        let initial_crossings = best_crossings;

        // Down sweep (top to bottom)
        for layer_idx in 1..graph.layer_count() {
            order_layer_by_barycenter(graph, layer_idx, true);
        }

        // Up sweep (bottom to top)
        for layer_idx in (0..graph.layer_count() - 1).rev() {
            order_layer_by_barycenter(graph, layer_idx, false);
        }

        // 2-opt local refinement
        if local_refinement {
            apply_two_opt(graph);
        }

        let current_crossings = count_total_crossings(graph);

        if current_crossings < best_crossings {
            best_crossings = current_crossings;
            best_order = save_layer_order(graph);
        }

        // Early termination if no improvement
        if current_crossings >= initial_crossings {
            break;
        }
    }

    // Restore best order found
    restore_layer_order(graph, &best_order);
}

/// Order a single layer using barycenter heuristic.
///
/// If `use_predecessors` is true, order based on predecessor positions.
/// Otherwise, order based on successor positions.
fn order_layer_by_barycenter(graph: &mut LayeredGraph, layer_idx: usize, use_predecessors: bool) {
    let layer = &graph.layers[layer_idx];
    if layer.len() <= 1 {
        return;
    }

    // Calculate barycenter for each node
    let mut barycenters: Vec<(usize, f64)> = layer
        .iter()
        .map(|&node_idx| {
            let bc = if use_predecessors {
                calculate_barycenter_from_predecessors(graph, node_idx)
            } else {
                calculate_barycenter_from_successors(graph, node_idx)
            };
            (node_idx, bc)
        })
        .collect();

    // Sort by barycenter, using original position as tiebreaker
    barycenters.sort_by(|(idx_a, bc_a), (idx_b, bc_b)| {
        bc_a.partial_cmp(bc_b).unwrap_or(std::cmp::Ordering::Equal).then_with(|| {
            graph.nodes[*idx_a]
                .position_in_layer
                .cmp(&graph.nodes[*idx_b].position_in_layer)
        })
    });

    // Update layer order
    graph.layers[layer_idx] = barycenters.iter().map(|(idx, _)| *idx).collect();

    // Update position_in_layer for nodes
    for (pos, &node_idx) in graph.layers[layer_idx].iter().enumerate() {
        graph.nodes[node_idx].position_in_layer = pos;
    }
}

/// Calculate barycenter based on predecessor positions.
fn calculate_barycenter_from_predecessors(graph: &LayeredGraph, node_idx: usize) -> f64 {
    let predecessors = graph.predecessors(node_idx);
    if predecessors.is_empty() {
        // Keep original position if no predecessors
        return graph.nodes[node_idx].position_in_layer as f64;
    }

    let sum: f64 = predecessors
        .iter()
        .map(|&pred| graph.nodes[pred].position_in_layer as f64)
        .sum();

    sum / predecessors.len() as f64
}

/// Calculate barycenter based on successor positions.
fn calculate_barycenter_from_successors(graph: &LayeredGraph, node_idx: usize) -> f64 {
    let successors = graph.successors(node_idx);
    if successors.is_empty() {
        return graph.nodes[node_idx].position_in_layer as f64;
    }

    let sum: f64 = successors
        .iter()
        .map(|&succ| graph.nodes[succ].position_in_layer as f64)
        .sum();

    sum / successors.len() as f64
}

/// Apply 2-opt local refinement to reduce crossings.
///
/// For each layer, try swapping adjacent nodes and keep the swap
/// if it reduces total crossings.
fn apply_two_opt(graph: &mut LayeredGraph) {
    let mut improved = true;

    while improved {
        improved = false;

        for layer_idx in 0..graph.layer_count() {
            let layer_len = graph.layers[layer_idx].len();
            if layer_len < 2 {
                continue;
            }

            for i in 0..layer_len - 1 {
                let current_crossings = count_crossings_for_layer(graph, layer_idx);

                // Swap adjacent nodes
                graph.layers[layer_idx].swap(i, i + 1);

                // Update positions
                let node_a = graph.layers[layer_idx][i];
                let node_b = graph.layers[layer_idx][i + 1];
                graph.nodes[node_a].position_in_layer = i;
                graph.nodes[node_b].position_in_layer = i + 1;

                let new_crossings = count_crossings_for_layer(graph, layer_idx);

                if new_crossings < current_crossings {
                    improved = true;
                } else {
                    // Revert swap
                    graph.layers[layer_idx].swap(i, i + 1);
                    graph.nodes[node_a].position_in_layer = i + 1;
                    graph.nodes[node_b].position_in_layer = i;
                }
            }
        }
    }
}

/// Count total crossings in the entire graph.
pub fn count_total_crossings(graph: &LayeredGraph) -> usize {
    let mut total = 0;

    for layer_idx in 0..graph.layer_count().saturating_sub(1) {
        total += count_crossings_between_layers(graph, layer_idx, layer_idx + 1);
    }

    total
}

/// Count crossings involving a specific layer (with adjacent layers).
fn count_crossings_for_layer(graph: &LayeredGraph, layer_idx: usize) -> usize {
    let mut crossings = 0;

    // Crossings with layer above
    if layer_idx > 0 {
        crossings += count_crossings_between_layers(graph, layer_idx - 1, layer_idx);
    }

    // Crossings with layer below
    if layer_idx + 1 < graph.layer_count() {
        crossings += count_crossings_between_layers(graph, layer_idx, layer_idx + 1);
    }

    crossings
}

/// Count crossings between two adjacent layers.
///
/// Two edges cross if their endpoints are in opposite order in their layers.
/// Edge (u1, v1) crosses edge (u2, v2) if:
///   (pos(u1) < pos(u2) and pos(v1) > pos(v2)) or
///   (pos(u1) > pos(u2) and pos(v1) < pos(v2))
pub fn count_crossings_between_layers(
    graph: &LayeredGraph,
    upper_layer_idx: usize,
    lower_layer_idx: usize,
) -> usize {
    // Get all edges between these layers
    let edges_between: Vec<(usize, usize)> = graph
        .edges
        .iter()
        .filter_map(|e| {
            let source_layer = graph.nodes[e.source].layer;
            let target_layer = graph.nodes[e.target].layer;

            if source_layer == upper_layer_idx && target_layer == lower_layer_idx {
                Some((
                    graph.nodes[e.source].position_in_layer,
                    graph.nodes[e.target].position_in_layer,
                ))
            } else {
                None
            }
        })
        .collect();

    // Count inversions (crossings)
    let mut crossings = 0;

    for i in 0..edges_between.len() {
        for j in (i + 1)..edges_between.len() {
            let (u1, v1) = edges_between[i];
            let (u2, v2) = edges_between[j];

            // Edges cross if relative order is different
            if (u1 < u2 && v1 > v2) || (u1 > u2 && v1 < v2) {
                crossings += 1;
            }
        }
    }

    crossings
}

/// Save current layer ordering.
fn save_layer_order(graph: &LayeredGraph) -> Vec<Vec<usize>> {
    graph.layers.clone()
}

/// Restore a saved layer ordering.
fn restore_layer_order(graph: &mut LayeredGraph, order: &[Vec<usize>]) {
    graph.layers = order.to_vec();
    graph.sync_layers_to_nodes();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sugiyama::{dummy, ranking, LayoutEdge};

    fn setup_graph(nodes: Vec<String>, edges: Vec<LayoutEdge>) -> LayeredGraph {
        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        ranking::assign_layers(&mut graph);
        dummy::insert_dummy_nodes(&mut graph);
        graph
    }

    #[test]
    fn test_no_crossings_linear() {
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

        let graph = setup_graph(nodes, edges);
        assert_eq!(count_total_crossings(&graph), 0);
    }

    #[test]
    fn test_crossing_detection() {
        // Create a graph with crossing:
        // Layer 0: a, b
        // Layer 1: c, d
        // Edges: a->d, b->c (these cross)
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "c".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);

        // Manually set layers
        graph.nodes[0].layer = 0; // a
        graph.nodes[1].layer = 0; // b
        graph.nodes[2].layer = 1; // c
        graph.nodes[3].layer = 1; // d
        graph.build_layers();

        // Set positions to create crossing
        // a at position 0, b at position 1
        // c at position 0, d at position 1
        // a->d and b->c cross
        graph.nodes[0].position_in_layer = 0;
        graph.nodes[1].position_in_layer = 1;
        graph.nodes[2].position_in_layer = 0;
        graph.nodes[3].position_in_layer = 1;

        assert_eq!(count_total_crossings(&graph), 1);
    }

    #[test]
    fn test_minimize_removes_crossing() {
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "c".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        graph.nodes[0].layer = 0;
        graph.nodes[1].layer = 0;
        graph.nodes[2].layer = 1;
        graph.nodes[3].layer = 1;
        graph.build_layers();

        // Create initial crossing
        graph.nodes[0].position_in_layer = 0;
        graph.nodes[1].position_in_layer = 1;
        graph.nodes[2].position_in_layer = 0;
        graph.nodes[3].position_in_layer = 1;
        graph.layers[0] = vec![0, 1];
        graph.layers[1] = vec![2, 3];

        let initial_crossings = count_total_crossings(&graph);
        assert_eq!(initial_crossings, 1);

        minimize_crossings(&mut graph, 10, true);

        let final_crossings = count_total_crossings(&graph);
        assert_eq!(final_crossings, 0);
    }

    #[test]
    fn test_barycenter_calculation() {
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "c".to_string(),
                target: "e".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        // Layer 0: a(0), b(1), c(2)
        // Layer 1: d(3), e(4)
        graph.nodes[0].layer = 0;
        graph.nodes[0].position_in_layer = 0;
        graph.nodes[1].layer = 0;
        graph.nodes[1].position_in_layer = 1;
        graph.nodes[2].layer = 0;
        graph.nodes[2].position_in_layer = 2;
        graph.nodes[3].layer = 1;
        graph.nodes[3].position_in_layer = 0;
        graph.nodes[4].layer = 1;
        graph.nodes[4].position_in_layer = 1;
        graph.build_layers();

        // d has predecessors a(0) and b(1), barycenter = 0.5
        let bc_d = calculate_barycenter_from_predecessors(&graph, 3);
        assert!((bc_d - 0.5).abs() < 0.001);

        // e has predecessor c(2), barycenter = 2.0
        let bc_e = calculate_barycenter_from_predecessors(&graph, 4);
        assert!((bc_e - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_complex_crossing_minimization() {
        // More complex graph with potential for multiple crossings
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
            "f".to_string(),
        ];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "e".to_string(),
            },
            LayoutEdge {
                source: "a".to_string(),
                target: "f".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "c".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "c".to_string(),
                target: "e".to_string(),
            },
        ];

        let mut graph = setup_graph(nodes, edges);

        let initial_crossings = count_total_crossings(&graph);
        minimize_crossings(&mut graph, 24, true);
        let final_crossings = count_total_crossings(&graph);

        // Should either maintain or reduce crossings
        assert!(final_crossings <= initial_crossings);
    }
}
