//! Coordinate assignment for Sugiyama layout.
//!
//! This module assigns X and Y coordinates to nodes based on their
//! layer assignment and position within each layer.

use std::collections::HashMap;

use structurizr_core::view::AutoLayoutDirection;

use super::{LayeredGraph, Size, SugiyamaConfig};

/// Assign X and Y coordinates to all nodes.
pub fn assign_coordinates(
    graph: &mut LayeredGraph,
    config: &SugiyamaConfig,
    size_map: &HashMap<&str, Size>,
) {
    if graph.node_count() == 0 {
        return;
    }

    // First pass: assign Y coordinates based on layer
    assign_layer_positions(graph, config, size_map);

    // Second pass: assign X coordinates based on position in layer
    assign_node_positions(graph, config, size_map);

    // Third pass: refine positions to minimize edge length
    if config.force_directed_refinement {
        refine_positions(graph, config, size_map);
    }

    // Final pass: center the diagram
    center_diagram(graph, config);
}

/// Assign Y coordinates based on layer (rank).
fn assign_layer_positions(
    graph: &mut LayeredGraph,
    config: &SugiyamaConfig,
    size_map: &HashMap<&str, Size>,
) {
    let max_layer = graph.max_layer();

    for node in &mut graph.nodes {
        let _height = get_node_height(node, size_map, config);

        let layer_offset = match config.direction {
            AutoLayoutDirection::TopBottom => node.layer as f64,
            AutoLayoutDirection::BottomTop => (max_layer - node.layer) as f64,
            AutoLayoutDirection::LeftRight | AutoLayoutDirection::RightLeft => node.layer as f64,
        };

        match config.direction {
            AutoLayoutDirection::TopBottom | AutoLayoutDirection::BottomTop => {
                node.y = layer_offset * (config.default_height + config.rank_separation);
            }
            AutoLayoutDirection::LeftRight => {
                node.x = layer_offset * (config.default_width + config.rank_separation);
            }
            AutoLayoutDirection::RightLeft => {
                node.x = (max_layer as f64 - layer_offset)
                    * (config.default_width + config.rank_separation);
            }
        }
    }
}

/// Assign X coordinates based on barycenter of connected nodes.
///
/// Instead of uniform spacing, positions nodes based on where their
/// neighbors are located. This groups related nodes together naturally.
fn assign_node_positions(
    graph: &mut LayeredGraph,
    config: &SugiyamaConfig,
    size_map: &HashMap<&str, Size>,
) {
    // First pass: assign initial positions based on order
    // Use tight initial spacing that will be expanded by overlap resolution
    for layer_idx in 0..graph.layer_count() {
        let layer = &graph.layers[layer_idx];
        let layer_len = layer.len();

        if layer_len == 0 {
            continue;
        }

        // Calculate rough center position for each node based on order
        let spacing = config.default_width + config.node_separation;
        let total_width = (layer_len as f64 - 1.0) * spacing;
        let start_x = -total_width / 2.0;

        for (i, &node_idx) in layer.iter().enumerate() {
            let x = start_x + i as f64 * spacing;
            match config.direction {
                AutoLayoutDirection::TopBottom | AutoLayoutDirection::BottomTop => {
                    graph.nodes[node_idx].x = x;
                }
                AutoLayoutDirection::LeftRight | AutoLayoutDirection::RightLeft => {
                    graph.nodes[node_idx].y = x;
                }
            }
        }
    }

    // Second pass: pull non-first layers toward their predecessors (top-down)
    for layer_idx in 1..graph.layer_count() {
        let layer = graph.layers[layer_idx].clone();

        // Position each node at the barycenter of its predecessors
        for &node_idx in &layer {
            let predecessors = graph.predecessors(node_idx);
            if !predecessors.is_empty() {
                let barycenter: f64 = predecessors
                    .iter()
                    .map(|&p| get_position_coord(&graph.nodes[p], config))
                    .sum::<f64>()
                    / predecessors.len() as f64;
                set_position_coord(&mut graph.nodes[node_idx], barycenter, config);
            }
        }

        // Resolve overlaps after barycenter positioning
        resolve_overlaps_in_layer(graph, layer_idx, config, size_map);
    }

    // Third pass: pull ALL layers toward their successors (bottom-up)
    // This includes the first layer, so nodes at the top get positioned
    // based on where their children ended up
    for layer_idx in (0..graph.layer_count()).rev() {
        let layer = graph.layers[layer_idx].clone();
        for &node_idx in &layer {
            let successors = graph.successors(node_idx);
            if !successors.is_empty() {
                let current = get_position_coord(&graph.nodes[node_idx], config);
                let barycenter: f64 = successors
                    .iter()
                    .map(|&s| get_position_coord(&graph.nodes[s], config))
                    .sum::<f64>()
                    / successors.len() as f64;
                // Pull 50% toward successor barycenter (stronger pull)
                let new_pos = current + 0.5 * (barycenter - current);
                set_position_coord(&mut graph.nodes[node_idx], new_pos, config);
            }
        }
        resolve_overlaps_in_layer(graph, layer_idx, config, size_map);
    }
}

/// Refine positions by pulling nodes toward their connected neighbors.
///
/// Uses force-directed refinement with connectivity-weighted attraction:
/// - Highly-connected "hub" nodes have stronger influence
/// - Multiple iterations with constant pull (no damping) for aggressive grouping
/// - Maintains minimum separation between nodes
/// - Applies repulsive forces between non-connected nodes (if enabled)
fn refine_positions(
    graph: &mut LayeredGraph,
    config: &SugiyamaConfig,
    size_map: &HashMap<&str, Size>,
) {
    // Use constant pull factor without damping for more aggressive grouping
    const PULL_FACTOR: f64 = 0.4;
    // Repulsion factor (weaker than attraction to prioritize grouping connected nodes)
    const REPULSION_FACTOR: f64 = 0.15;
    // Minimum distance for repulsion to activate (prevents division by zero)
    const MIN_REPULSION_DISTANCE: f64 = 50.0;

    // More iterations for better convergence
    let iterations = config.force_iterations.max(15);

    // Build connectivity set for quick lookup (used for repulsion)
    let connected_pairs: std::collections::HashSet<(usize, usize)> = if config.repulsion_enabled {
        graph
            .edges
            .iter()
            .flat_map(|e| {
                vec![
                    (e.source.min(e.target), e.source.max(e.target)),
                ]
            })
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    for _ in 0..iterations {
        // Down sweep: adjust based on predecessors
        for layer_idx in 1..graph.layer_count() {
            let layer = graph.layers[layer_idx].clone();
            for &node_idx in &layer {
                let target = calculate_weighted_target_position(graph, node_idx, true, config);
                if let Some(target_x) = target {
                    let current = get_position_coord(&graph.nodes[node_idx], config);
                    let new_pos = current + PULL_FACTOR * (target_x - current);
                    set_position_coord(&mut graph.nodes[node_idx], new_pos, config);
                }
            }

            // Apply repulsive forces between non-connected nodes in same layer
            if config.repulsion_enabled {
                apply_repulsion_in_layer(
                    graph,
                    layer_idx,
                    &connected_pairs,
                    config,
                    REPULSION_FACTOR,
                    MIN_REPULSION_DISTANCE,
                );
            }

            resolve_overlaps_in_layer(graph, layer_idx, config, size_map);
        }

        // Up sweep: adjust based on successors
        for layer_idx in (0..graph.layer_count().saturating_sub(1)).rev() {
            let layer = graph.layers[layer_idx].clone();
            for &node_idx in &layer {
                let target = calculate_weighted_target_position(graph, node_idx, false, config);
                if let Some(target_x) = target {
                    let current = get_position_coord(&graph.nodes[node_idx], config);
                    let new_pos = current + PULL_FACTOR * (target_x - current);
                    set_position_coord(&mut graph.nodes[node_idx], new_pos, config);
                }
            }

            // Apply repulsive forces between non-connected nodes in same layer
            if config.repulsion_enabled {
                apply_repulsion_in_layer(
                    graph,
                    layer_idx,
                    &connected_pairs,
                    config,
                    REPULSION_FACTOR,
                    MIN_REPULSION_DISTANCE,
                );
            }

            resolve_overlaps_in_layer(graph, layer_idx, config, size_map);
        }
    }
}

/// Apply repulsive forces between non-connected nodes in the same layer.
///
/// This helps separate visually distinct clusters while connected nodes
/// remain grouped due to the stronger attraction forces.
fn apply_repulsion_in_layer(
    graph: &mut LayeredGraph,
    layer_idx: usize,
    connected_pairs: &std::collections::HashSet<(usize, usize)>,
    config: &SugiyamaConfig,
    repulsion_factor: f64,
    min_distance: f64,
) {
    let layer = graph.layers[layer_idx].clone();
    if layer.len() < 2 {
        return;
    }

    // Calculate repulsion forces for each node
    let mut forces: Vec<f64> = vec![0.0; layer.len()];

    for i in 0..layer.len() {
        let node_i = layer[i];
        // Skip dummy nodes
        if graph.nodes[node_i].is_dummy {
            continue;
        }

        for j in (i + 1)..layer.len() {
            let node_j = layer[j];
            // Skip dummy nodes
            if graph.nodes[node_j].is_dummy {
                continue;
            }

            // Check if nodes are connected (directly)
            let key = (node_i.min(node_j), node_i.max(node_j));
            if connected_pairs.contains(&key) {
                // Connected nodes don't repel each other
                continue;
            }

            // Also check if they share a common neighbor (indirect connection)
            // This prevents repulsion between nodes that should be grouped together
            let i_neighbors: std::collections::HashSet<_> = graph
                .predecessors(node_i)
                .into_iter()
                .chain(graph.successors(node_i))
                .collect();
            let j_neighbors: std::collections::HashSet<_> = graph
                .predecessors(node_j)
                .into_iter()
                .chain(graph.successors(node_j))
                .collect();

            // If they share neighbors, reduce repulsion significantly
            let shared_neighbors = i_neighbors.intersection(&j_neighbors).count();
            if shared_neighbors > 0 {
                continue; // Don't repel nodes that share neighbors
            }

            let pos_i = get_position_coord(&graph.nodes[node_i], config);
            let pos_j = get_position_coord(&graph.nodes[node_j], config);
            let distance = (pos_j - pos_i).abs().max(min_distance);

            // Inverse-square repulsion force (weaker at larger distances)
            // Force magnitude: repulsion_factor * (ideal_distance / actual_distance)^2
            let ideal_distance = config.node_separation + config.default_width;
            let force_magnitude = repulsion_factor * (ideal_distance / distance).powi(2);

            // Cap the force to prevent extreme movements
            let capped_force = force_magnitude.min(repulsion_factor * 2.0);

            // Apply forces in opposite directions
            if pos_i < pos_j {
                forces[i] -= capped_force * (ideal_distance - distance).max(0.0);
                forces[j] += capped_force * (ideal_distance - distance).max(0.0);
            } else {
                forces[i] += capped_force * (ideal_distance - distance).max(0.0);
                forces[j] -= capped_force * (ideal_distance - distance).max(0.0);
            }
        }
    }

    // Apply accumulated forces
    for (i, &node_idx) in layer.iter().enumerate() {
        if graph.nodes[node_idx].is_dummy {
            continue;
        }
        let current = get_position_coord(&graph.nodes[node_idx], config);
        let new_pos = current + forces[i];
        set_position_coord(&mut graph.nodes[node_idx], new_pos, config);
    }
}

/// Calculate target position based on connected nodes.
///
/// Uses weighted averaging where highly-connected "hub" nodes have more
/// influence on the target position. This helps group related nodes together.
fn calculate_weighted_target_position(
    graph: &LayeredGraph,
    node_idx: usize,
    use_predecessors: bool,
    config: &SugiyamaConfig,
) -> Option<f64> {
    let neighbors = if use_predecessors {
        graph.predecessors(node_idx)
    } else {
        graph.successors(node_idx)
    };

    if neighbors.is_empty() {
        return None;
    }

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for &neighbor_idx in &neighbors {
        // Calculate weight based on:
        // 1. Number of edges between this pair
        // 2. Total connectivity of the neighbor
        let edge_count = if use_predecessors {
            graph
                .edges
                .iter()
                .filter(|e| e.source == neighbor_idx && e.target == node_idx)
                .count() as f64
        } else {
            graph
                .edges
                .iter()
                .filter(|e| e.source == node_idx && e.target == neighbor_idx)
                .count() as f64
        };

        // Neighbor's total degree (incoming + outgoing edges)
        let neighbor_degree =
            graph.incoming_edges(neighbor_idx).len() + graph.outgoing_edges(neighbor_idx).len();

        // Combined weight: edge count * (1 + log(degree))
        // The log dampens the degree effect to avoid over-weighting hubs
        let degree_factor = 1.0 + (neighbor_degree as f64).ln().max(0.0);
        let weight = edge_count.max(1.0) * degree_factor;

        weighted_sum += get_position_coord(&graph.nodes[neighbor_idx], config) * weight;
        total_weight += weight;
    }

    Some(weighted_sum / total_weight)
}

/// Get the position coordinate (X for TopBottom/BottomTop, Y for LeftRight/RightLeft).
fn get_position_coord(node: &super::LayeredNode, config: &SugiyamaConfig) -> f64 {
    match config.direction {
        AutoLayoutDirection::TopBottom | AutoLayoutDirection::BottomTop => node.x,
        AutoLayoutDirection::LeftRight | AutoLayoutDirection::RightLeft => node.y,
    }
}

/// Set the position coordinate.
fn set_position_coord(node: &mut super::LayeredNode, value: f64, config: &SugiyamaConfig) {
    match config.direction {
        AutoLayoutDirection::TopBottom | AutoLayoutDirection::BottomTop => node.x = value,
        AutoLayoutDirection::LeftRight | AutoLayoutDirection::RightLeft => node.y = value,
    }
}

/// Resolve overlaps within a layer by pushing nodes apart.
fn resolve_overlaps_in_layer(
    graph: &mut LayeredGraph,
    layer_idx: usize,
    config: &SugiyamaConfig,
    size_map: &HashMap<&str, Size>,
) {
    let layer = &graph.layers[layer_idx];
    if layer.len() < 2 {
        return;
    }

    // Sort nodes by current position
    let mut sorted_indices: Vec<usize> = layer.clone();
    sorted_indices.sort_by(|&a, &b| {
        let pos_a = get_position_coord(&graph.nodes[a], config);
        let pos_b = get_position_coord(&graph.nodes[b], config);
        pos_a.partial_cmp(&pos_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Push nodes apart to eliminate overlaps
    for i in 1..sorted_indices.len() {
        let prev_idx = sorted_indices[i - 1];
        let curr_idx = sorted_indices[i];

        let prev_pos = get_position_coord(&graph.nodes[prev_idx], config);
        let prev_size = get_node_size(&graph.nodes[prev_idx], size_map, config);
        let curr_pos = get_position_coord(&graph.nodes[curr_idx], config);

        let min_pos = prev_pos + prev_size + config.node_separation;

        if curr_pos < min_pos {
            set_position_coord(&mut graph.nodes[curr_idx], min_pos, config);
        }
    }
}

/// Get node width (or height for horizontal layouts).
fn get_node_size(
    node: &super::LayeredNode,
    size_map: &HashMap<&str, Size>,
    config: &SugiyamaConfig,
) -> f64 {
    if node.is_dummy {
        // Dummy nodes have minimal size
        return 0.0;
    }

    let size = size_map.get(node.id.as_str()).copied().unwrap_or(Size {
        width: config.default_width,
        height: config.default_height,
    });

    match config.direction {
        AutoLayoutDirection::TopBottom | AutoLayoutDirection::BottomTop => size.width,
        AutoLayoutDirection::LeftRight | AutoLayoutDirection::RightLeft => size.height,
    }
}

#[allow(dead_code)]
fn get_node_width(
    node: &super::LayeredNode,
    size_map: &HashMap<&str, Size>,
    config: &SugiyamaConfig,
) -> f64 {
    if node.is_dummy {
        return 0.0;
    }

    size_map
        .get(node.id.as_str())
        .map(|s| s.width)
        .unwrap_or(config.default_width)
}

fn get_node_height(
    node: &super::LayeredNode,
    size_map: &HashMap<&str, Size>,
    config: &SugiyamaConfig,
) -> f64 {
    if node.is_dummy {
        return 0.0;
    }

    size_map
        .get(node.id.as_str())
        .map(|s| s.height)
        .unwrap_or(config.default_height)
}

/// Center the entire diagram around origin and add padding.
fn center_diagram(graph: &mut LayeredGraph, _config: &SugiyamaConfig) {
    if graph.node_count() == 0 {
        return;
    }

    // Find bounding box
    let min_x = graph
        .nodes
        .iter()
        .map(|n| n.x)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);
    let min_y = graph
        .nodes
        .iter()
        .map(|n| n.y)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    // Shift all nodes to start from padding
    let padding = 50.0;
    let offset_x = padding - min_x;
    let offset_y = padding - min_y;

    for node in &mut graph.nodes {
        node.x += offset_x;
        node.y += offset_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sugiyama::{dummy, ordering, ranking, LayoutEdge, LayeredGraph};

    fn setup_and_position(nodes: Vec<String>, edges: Vec<LayoutEdge>) -> LayeredGraph {
        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        let size_map: HashMap<&str, Size> = nodes.iter().map(|id| (id.as_str(), Size::default())).collect();
        let config = SugiyamaConfig::default();

        ranking::assign_layers(&mut graph);
        dummy::insert_dummy_nodes(&mut graph);
        ordering::minimize_crossings(&mut graph, &config);
        assign_coordinates(&mut graph, &config, &size_map);

        graph
    }

    #[test]
    fn test_linear_positions() {
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

        let graph = setup_and_position(nodes, edges);

        // Check that Y positions increase with layer
        let a = graph.nodes.iter().find(|n| n.id == "a").unwrap();
        let b = graph.nodes.iter().find(|n| n.id == "b").unwrap();
        let c = graph.nodes.iter().find(|n| n.id == "c").unwrap();

        assert!(a.y < b.y, "a.y ({}) should be less than b.y ({})", a.y, b.y);
        assert!(b.y < c.y, "b.y ({}) should be less than c.y ({})", b.y, c.y);
    }

    #[test]
    fn test_same_layer_separation() {
        let nodes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "c".to_string(),
            },
            LayoutEdge {
                source: "a".to_string(),
                target: "d".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "c".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "d".to_string(),
            },
        ];

        let graph = setup_and_position(nodes, edges);

        // Find nodes in same layer (a and b should be in layer 0)
        let a = graph.nodes.iter().find(|n| n.id == "a").unwrap();
        let b = graph.nodes.iter().find(|n| n.id == "b").unwrap();

        // They should have different X positions
        assert!(
            (a.x - b.x).abs() > 1.0,
            "a.x ({}) and b.x ({}) should be different",
            a.x,
            b.x
        );

        // They should be at the same Y level
        assert!(
            (a.y - b.y).abs() < 0.001,
            "a.y ({}) and b.y ({}) should be equal",
            a.y,
            b.y
        );
    }

    #[test]
    fn test_no_overlaps() {
        let nodes: Vec<String> = (0..10).map(|i| format!("node_{}", i)).collect();
        let edges: Vec<LayoutEdge> = (0..9)
            .map(|i| LayoutEdge {
                source: format!("node_{}", i),
                target: format!("node_{}", i + 1),
            })
            .collect();

        let graph = setup_and_position(nodes.clone(), edges);
        let config = SugiyamaConfig::default();

        // Check no overlapping positions in same layer
        for layer in &graph.layers {
            for i in 0..layer.len() {
                for j in (i + 1)..layer.len() {
                    let node_i = &graph.nodes[layer[i]];
                    let node_j = &graph.nodes[layer[j]];

                    let dist = (node_i.x - node_j.x).abs();
                    assert!(
                        dist >= config.default_width || node_i.is_dummy || node_j.is_dummy,
                        "Nodes {} and {} overlap: x={}, x={}",
                        node_i.id,
                        node_j.id,
                        node_i.x,
                        node_j.x
                    );
                }
            }
        }
    }

    #[test]
    fn test_positions_positive() {
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![LayoutEdge {
            source: "a".to_string(),
            target: "b".to_string(),
        }];

        let graph = setup_and_position(nodes, edges);

        // All positions should be positive (after centering with padding)
        for node in &graph.nodes {
            assert!(node.x >= 0.0, "Node {} has negative x: {}", node.id, node.x);
            assert!(node.y >= 0.0, "Node {} has negative y: {}", node.id, node.y);
        }
    }
}
