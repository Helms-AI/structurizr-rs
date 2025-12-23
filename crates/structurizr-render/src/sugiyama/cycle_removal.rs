//! Cycle removal for converting graphs to DAGs.
//!
//! The Sugiyama algorithm requires a directed acyclic graph (DAG).
//! This module detects cycles and removes them by reversing edges.

use super::LayeredGraph;
use std::collections::HashSet;

/// Remove cycles from the graph by reversing edges.
///
/// Returns the indices of edges that were reversed.
pub fn remove_cycles(graph: &mut LayeredGraph) -> Vec<usize> {
    let mut reversed = Vec::new();

    // Use DFS-based cycle detection
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut cycle_edges = Vec::new();

    for start in 0..graph.node_count() {
        if !visited.contains(&start) {
            dfs_find_cycle_edges(
                graph,
                start,
                &mut visited,
                &mut rec_stack,
                &mut cycle_edges,
            );
        }
    }

    // Reverse the cycle-breaking edges
    for edge_idx in cycle_edges {
        if edge_idx < graph.edges.len() {
            let edge = &mut graph.edges[edge_idx];
            // Swap source and target
            std::mem::swap(&mut edge.source, &mut edge.target);
            edge.reversed = true;
            reversed.push(edge.original_index);
        }
    }

    reversed
}

/// DFS to find edges that create cycles.
fn dfs_find_cycle_edges(
    graph: &LayeredGraph,
    node: usize,
    visited: &mut HashSet<usize>,
    rec_stack: &mut HashSet<usize>,
    cycle_edges: &mut Vec<usize>,
) {
    visited.insert(node);
    rec_stack.insert(node);

    // Get outgoing edge indices and targets
    let outgoing: Vec<(usize, usize)> = graph
        .edges
        .iter()
        .enumerate()
        .filter(|(_, e)| e.source == node)
        .map(|(i, e)| (i, e.target))
        .collect();

    for (edge_idx, target) in outgoing {
        if !visited.contains(&target) {
            dfs_find_cycle_edges(graph, target, visited, rec_stack, cycle_edges);
        } else if rec_stack.contains(&target) {
            // Found a back edge (cycle)
            cycle_edges.push(edge_idx);
        }
    }

    rec_stack.remove(&node);
}

/// Check if the graph is acyclic (DAG).
pub fn is_acyclic(graph: &LayeredGraph) -> bool {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for start in 0..graph.node_count() {
        if !visited.contains(&start) {
            if has_cycle_dfs(graph, start, &mut visited, &mut rec_stack) {
                return false;
            }
        }
    }

    true
}

/// DFS helper to check for cycles.
fn has_cycle_dfs(
    graph: &LayeredGraph,
    node: usize,
    visited: &mut HashSet<usize>,
    rec_stack: &mut HashSet<usize>,
) -> bool {
    visited.insert(node);
    rec_stack.insert(node);

    for edge in &graph.edges {
        if edge.source == node {
            let target = edge.target;
            if !visited.contains(&target) {
                if has_cycle_dfs(graph, target, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&target) {
                return true;
            }
        }
    }

    rec_stack.remove(&node);
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sugiyama::LayoutEdge;

    #[test]
    fn test_acyclic_graph() {
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

        let graph = LayeredGraph::from_input(&nodes, &edges);
        assert!(is_acyclic(&graph));
    }

    #[test]
    fn test_simple_cycle() {
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
                source: "c".to_string(),
                target: "a".to_string(),
            },
        ];

        let graph = LayeredGraph::from_input(&nodes, &edges);
        assert!(!is_acyclic(&graph));
    }

    #[test]
    fn test_remove_cycle() {
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
                source: "c".to_string(),
                target: "a".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        let reversed = remove_cycles(&mut graph);

        // Should have reversed exactly one edge
        assert_eq!(reversed.len(), 1);

        // Graph should now be acyclic
        assert!(is_acyclic(&graph));
    }

    #[test]
    fn test_self_loop() {
        let nodes = vec!["a".to_string()];
        let edges = vec![LayoutEdge {
            source: "a".to_string(),
            target: "a".to_string(),
        }];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        assert!(!is_acyclic(&graph));

        let reversed = remove_cycles(&mut graph);
        assert_eq!(reversed.len(), 1);
    }

    #[test]
    fn test_two_node_cycle() {
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![
            LayoutEdge {
                source: "a".to_string(),
                target: "b".to_string(),
            },
            LayoutEdge {
                source: "b".to_string(),
                target: "a".to_string(),
            },
        ];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        let reversed = remove_cycles(&mut graph);

        assert_eq!(reversed.len(), 1);
        assert!(is_acyclic(&graph));
    }
}
