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
    refine_positions(graph, config, size_map);

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

/// Assign X coordinates based on position within layer.
fn assign_node_positions(
    graph: &mut LayeredGraph,
    config: &SugiyamaConfig,
    size_map: &HashMap<&str, Size>,
) {
    for layer_idx in 0..graph.layer_count() {
        let layer = &graph.layers[layer_idx];
        let layer_len = layer.len();

        if layer_len == 0 {
            continue;
        }

        // Calculate total width of nodes in this layer
        let total_width: f64 = layer
            .iter()
            .map(|&idx| get_node_width(&graph.nodes[idx], size_map, config))
            .sum();

        let total_gaps = if layer_len > 1 {
            (layer_len - 1) as f64 * config.node_separation
        } else {
            0.0
        };

        let layer_total_width = total_width + total_gaps;

        // Start position to center the layer
        let start_x = -layer_total_width / 2.0;

        let mut current_x = start_x;
        for &node_idx in layer {
            let node_width = get_node_width(&graph.nodes[node_idx], size_map, config);

            match config.direction {
                AutoLayoutDirection::TopBottom | AutoLayoutDirection::BottomTop => {
                    graph.nodes[node_idx].x = current_x;
                }
                AutoLayoutDirection::LeftRight | AutoLayoutDirection::RightLeft => {
                    graph.nodes[node_idx].y = current_x;
                }
            }

            current_x += node_width + config.node_separation;
        }
    }
}

/// Refine positions by pulling nodes toward their connected neighbors.
fn refine_positions(
    graph: &mut LayeredGraph,
    config: &SugiyamaConfig,
    size_map: &HashMap<&str, Size>,
) {
    const ITERATIONS: usize = 5;
    const PULL_FACTOR: f64 = 0.3;

    for _ in 0..ITERATIONS {
        // Down sweep: adjust based on predecessors
        for layer_idx in 1..graph.layer_count() {
            let layer = graph.layers[layer_idx].clone();
            for &node_idx in &layer {
                let target = calculate_target_position(graph, node_idx, true, config);
                if let Some(target_x) = target {
                    let current = get_position_coord(&graph.nodes[node_idx], config);
                    let new_pos = current + PULL_FACTOR * (target_x - current);
                    set_position_coord(&mut graph.nodes[node_idx], new_pos, config);
                }
            }
            resolve_overlaps_in_layer(graph, layer_idx, config, size_map);
        }

        // Up sweep: adjust based on successors
        for layer_idx in (0..graph.layer_count().saturating_sub(1)).rev() {
            let layer = graph.layers[layer_idx].clone();
            for &node_idx in &layer {
                let target = calculate_target_position(graph, node_idx, false, config);
                if let Some(target_x) = target {
                    let current = get_position_coord(&graph.nodes[node_idx], config);
                    let new_pos = current + PULL_FACTOR * (target_x - current);
                    set_position_coord(&mut graph.nodes[node_idx], new_pos, config);
                }
            }
            resolve_overlaps_in_layer(graph, layer_idx, config, size_map);
        }
    }
}

/// Calculate target position based on connected nodes.
fn calculate_target_position(
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

    let sum: f64 = neighbors
        .iter()
        .map(|&n| get_position_coord(&graph.nodes[n], config))
        .sum();

    Some(sum / neighbors.len() as f64)
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
        ordering::minimize_crossings(&mut graph, 10, true);
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
