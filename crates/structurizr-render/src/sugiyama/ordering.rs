//! Crossing minimization for Sugiyama layout.
//!
//! This module implements the barycentric heuristic with 2-opt local refinement
//! to minimize edge crossings in the layered graph.
//!
//! The algorithm includes:
//! 1. Connectivity-based initial ordering (groups related nodes)
//! 2. Weighted barycentric heuristic (hubs have more influence)
//! 3. 2-opt local refinement (swap adjacent nodes if it helps)

use std::collections::HashSet;

use super::{LayeredGraph, SugiyamaConfig};

/// Minimize edge crossings using barycentric heuristic with 2-opt refinement.
///
/// The algorithm:
/// 1. Initial pass: Order layers by connectivity (group related nodes)
/// 2. Iterative sweeps:
///    - Down sweep: Order each layer based on positions of predecessors
///    - Up sweep: Order each layer based on positions of successors
///    - 2-opt: Swap adjacent nodes if it reduces crossings
pub fn minimize_crossings(graph: &mut LayeredGraph, config: &SugiyamaConfig) {
    if graph.layer_count() < 2 {
        return;
    }

    // Phase 1: Connectivity-based initial ordering
    // Groups nodes with shared neighbors together before barycentric optimization
    if config.connectivity_ordering {
        apply_connectivity_ordering(graph);
    }

    let mut best_crossings = count_total_crossings(graph);
    let mut best_order = save_layer_order(graph);

    for _iteration in 0..config.max_iterations {
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
        if config.local_refinement {
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

/// Apply connectivity-based ordering to all layers.
///
/// This groups nodes with shared neighbors together, providing a better
/// starting point for the barycentric optimization.
fn apply_connectivity_ordering(graph: &mut LayeredGraph) {
    for layer_idx in 0..graph.layer_count() {
        order_layer_by_connectivity(graph, layer_idx);
    }
}

/// Order nodes in a layer by grouping those with shared neighbors together.
///
/// Algorithm:
/// 1. Calculate shared neighbor count between each pair of nodes
/// 2. Greedy clustering: start with most-connected node, add closest neighbor
/// 3. Order clusters and nodes within clusters by connectivity
fn order_layer_by_connectivity(graph: &mut LayeredGraph, layer_idx: usize) {
    let layer = &graph.layers[layer_idx];
    if layer.len() <= 2 {
        return;
    }

    let node_indices: Vec<usize> = layer.clone();
    let n = node_indices.len();

    // Build connectivity matrix: shared_neighbors[i][j] = count of shared neighbors
    let mut shared_neighbors: Vec<Vec<usize>> = vec![vec![0; n]; n];

    for i in 0..n {
        let node_i = node_indices[i];
        let preds_i: HashSet<usize> = graph.predecessors(node_i).into_iter().collect();
        let succs_i: HashSet<usize> = graph.successors(node_i).into_iter().collect();

        for j in (i + 1)..n {
            let node_j = node_indices[j];
            let preds_j: HashSet<usize> = graph.predecessors(node_j).into_iter().collect();
            let succs_j: HashSet<usize> = graph.successors(node_j).into_iter().collect();

            // Count shared predecessors and successors
            let shared_preds = preds_i.intersection(&preds_j).count();
            let shared_succs = succs_i.intersection(&succs_j).count();
            let total_shared = shared_preds + shared_succs;

            shared_neighbors[i][j] = total_shared;
            shared_neighbors[j][i] = total_shared;
        }
    }

    // Greedy clustering: build ordered list by starting with most-connected node
    // and repeatedly adding the most-related unassigned node
    let mut ordered: Vec<usize> = Vec::with_capacity(n);
    let mut assigned: HashSet<usize> = HashSet::new();

    // Start with the node that has the highest total connectivity
    let start_idx = (0..n)
        .max_by_key(|&i| {
            let node = node_indices[i];
            graph.incoming_edges(node).len() + graph.outgoing_edges(node).len()
        })
        .unwrap_or(0);

    ordered.push(node_indices[start_idx]);
    assigned.insert(start_idx);

    // Repeatedly add the most-related unassigned node
    while ordered.len() < n {
        let mut best_candidate = None;
        let mut best_score = 0;

        for i in 0..n {
            if assigned.contains(&i) {
                continue;
            }

            // Calculate total shared neighbors with already-ordered nodes
            let score: usize = assigned
                .iter()
                .map(|&j| shared_neighbors[i][j])
                .sum();

            // Tiebreaker: use connectivity degree
            let degree = graph.incoming_edges(node_indices[i]).len()
                + graph.outgoing_edges(node_indices[i]).len();

            // Combine score with degree for tiebreaking (shift score left to prioritize it)
            let combined = score * 1000 + degree;

            if best_candidate.is_none() || combined > best_score {
                best_score = combined;
                best_candidate = Some(i);
            }
        }

        if let Some(idx) = best_candidate {
            ordered.push(node_indices[idx]);
            assigned.insert(idx);
        } else {
            // Fallback: add remaining nodes in original order
            for i in 0..n {
                if !assigned.contains(&i) {
                    ordered.push(node_indices[i]);
                    assigned.insert(i);
                }
            }
            break;
        }
    }

    // Update layer and node positions
    graph.layers[layer_idx] = ordered;
    for (pos, &node_idx) in graph.layers[layer_idx].iter().enumerate() {
        graph.nodes[node_idx].position_in_layer = pos;
    }
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
///
/// Uses weighted averaging where nodes with higher connectivity (more edges)
/// have stronger influence on the barycenter. This helps group related nodes
/// together by pulling nodes more strongly toward highly-connected neighbors.
fn calculate_barycenter_from_predecessors(graph: &LayeredGraph, node_idx: usize) -> f64 {
    let predecessors = graph.predecessors(node_idx);
    if predecessors.is_empty() {
        // Keep original position if no predecessors
        return graph.nodes[node_idx].position_in_layer as f64;
    }

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for &pred_idx in &predecessors {
        // Weight by:
        // 1. Number of edges between this pair (supports multi-edge scenarios)
        // 2. Total connectivity of the predecessor (hubs have more influence)
        let edge_count = graph
            .edges
            .iter()
            .filter(|e| e.source == pred_idx && e.target == node_idx)
            .count() as f64;

        // Predecessor's total degree (incoming + outgoing edges)
        let pred_degree = graph.incoming_edges(pred_idx).len() + graph.outgoing_edges(pred_idx).len();

        // Combined weight: edge count * (1 + log(degree))
        // The log dampens the degree effect to avoid over-weighting hubs
        let degree_factor = 1.0 + (pred_degree as f64).ln().max(0.0);
        let weight = edge_count.max(1.0) * degree_factor;

        weighted_sum += graph.nodes[pred_idx].position_in_layer as f64 * weight;
        total_weight += weight;
    }

    weighted_sum / total_weight
}

/// Calculate barycenter based on successor positions.
///
/// Uses weighted averaging where nodes with higher connectivity (more edges)
/// have stronger influence on the barycenter. This helps group related nodes
/// together by pulling nodes more strongly toward highly-connected neighbors.
fn calculate_barycenter_from_successors(graph: &LayeredGraph, node_idx: usize) -> f64 {
    let successors = graph.successors(node_idx);
    if successors.is_empty() {
        return graph.nodes[node_idx].position_in_layer as f64;
    }

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for &succ_idx in &successors {
        // Weight by:
        // 1. Number of edges between this pair (supports multi-edge scenarios)
        // 2. Total connectivity of the successor (hubs have more influence)
        let edge_count = graph
            .edges
            .iter()
            .filter(|e| e.source == node_idx && e.target == succ_idx)
            .count() as f64;

        // Successor's total degree (incoming + outgoing edges)
        let succ_degree = graph.incoming_edges(succ_idx).len() + graph.outgoing_edges(succ_idx).len();

        // Combined weight: edge count * (1 + log(degree))
        // The log dampens the degree effect to avoid over-weighting hubs
        let degree_factor = 1.0 + (succ_degree as f64).ln().max(0.0);
        let weight = edge_count.max(1.0) * degree_factor;

        weighted_sum += graph.nodes[succ_idx].position_in_layer as f64 * weight;
        total_weight += weight;
    }

    weighted_sum / total_weight
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

        let config = SugiyamaConfig::default();
        minimize_crossings(&mut graph, &config);

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
        let config = SugiyamaConfig::default();
        minimize_crossings(&mut graph, &config);
        let final_crossings = count_total_crossings(&graph);

        // Should either maintain or reduce crossings
        assert!(final_crossings <= initial_crossings);
    }
}
