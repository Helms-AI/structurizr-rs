//! Auto-layout algorithms for diagram elements.
//!
//! This module provides layout algorithms for positioning diagram elements:
//! - `GridLayout`: Simple grid-based layout (original algorithm)
//! - `sugiyama_layout`: Advanced Sugiyama-style layout with crossing minimization
//!
//! For orthogonal edge routing, see the `routing` module.

use structurizr_core::view::{AutoLayout, AutoLayoutDirection};

// Import advanced layout implementation
use crate::sugiyama as advanced;

/// Position of an element in the layout.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Size of an element.
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 250.0,
        }
    }
}

/// A point in 2D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A node in the layout graph.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub position: Position,
    pub size: Size,
    pub rank: usize,
}

/// An edge in the layout graph.
#[derive(Debug, Clone)]
pub struct LayoutEdge {
    pub source: String,
    pub target: String,
}

/// Configuration for adaptive layout spacing.
#[derive(Debug, Clone, Copy)]
pub struct AdaptiveLayoutConfig {
    /// Target viewport width in pixels
    pub target_width: f64,
    /// Target viewport height in pixels
    pub target_height: f64,
    /// Minimum spacing between nodes/ranks
    pub min_spacing: f64,
    /// Maximum spacing between nodes/ranks
    pub max_spacing: f64,
    /// Padding around the entire diagram
    pub padding: f64,
}

impl Default for AdaptiveLayoutConfig {
    fn default() -> Self {
        Self {
            target_width: 2400.0,   // Doubled for larger diagrams
            target_height: 1800.0,  // Doubled for larger diagrams
            min_spacing: 150.0,     // Increased from 50 for better readability
            max_spacing: 300.0,     // Increased from 200
            padding: 80.0,          // Increased from 50
        }
    }
}

/// Simple grid-based auto-layout with adaptive spacing.
pub struct GridLayout {
    pub direction: AutoLayoutDirection,
    pub rank_separation: f64,
    pub node_separation: f64,
    pub default_width: f64,
    pub default_height: f64,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            direction: AutoLayoutDirection::TopBottom,
            rank_separation: 150.0,  // Increased for better readability
            node_separation: 150.0,  // Increased for better readability
            default_width: 400.0,
            default_height: 250.0,
        }
    }
}

impl GridLayout {
    pub fn from_config(config: &AutoLayout) -> Self {
        Self {
            direction: config.direction,
            rank_separation: config.rank_separation as f64,
            node_separation: config.node_separation as f64,
            ..Default::default()
        }
    }

    /// Calculate adaptive spacing based on element count and graph structure.
    pub fn calculate_adaptive_spacing(
        &self,
        max_rank: usize,
        max_nodes_per_rank: usize,
        config: &AdaptiveLayoutConfig,
    ) -> (f64, f64) {
        // Calculate rank separation (vertical spacing between layers)
        let ranks_count = (max_rank + 1).max(1);
        let total_rank_height = ranks_count as f64 * self.default_height;
        let available_rank_space = config.target_height - total_rank_height - (config.padding * 2.0);
        let rank_gaps = (ranks_count.saturating_sub(1)).max(1) as f64;
        let calculated_rank_sep = available_rank_space / rank_gaps;
        let rank_separation = calculated_rank_sep.clamp(config.min_spacing, config.max_spacing);

        // Calculate node separation (horizontal spacing within a layer)
        let nodes_in_widest = max_nodes_per_rank.max(1);
        let total_node_width = nodes_in_widest as f64 * self.default_width;
        let available_node_space = config.target_width - total_node_width - (config.padding * 2.0);
        let node_gaps = (nodes_in_widest.saturating_sub(1)).max(1) as f64;
        let calculated_node_sep = available_node_space / node_gaps;
        let node_separation = calculated_node_sep.clamp(config.min_spacing, config.max_spacing);

        (rank_separation, node_separation)
    }

    /// Layout nodes with adaptive spacing based on element count.
    pub fn layout_adaptive(&self, node_ids: &[String], edges: &[LayoutEdge]) -> Vec<LayoutNode> {
        let config = AdaptiveLayoutConfig::default();

        if node_ids.is_empty() {
            return Vec::new();
        }

        // Special case: single element centered
        if node_ids.len() == 1 {
            return vec![LayoutNode {
                id: node_ids[0].clone(),
                position: Position {
                    x: config.target_width / 2.0 - self.default_width / 2.0,
                    y: config.target_height / 2.0 - self.default_height / 2.0,
                },
                size: Size {
                    width: self.default_width,
                    height: self.default_height,
                },
                rank: 0,
            }];
        }

        // Compute ranks to understand graph structure
        let ranks = self.compute_ranks(node_ids, edges);
        let max_rank = ranks.iter().max().copied().unwrap_or(0);

        // Group nodes by rank to find widest layer
        let mut ranks_map: std::collections::HashMap<usize, Vec<&String>> = std::collections::HashMap::new();
        for (id, &rank) in node_ids.iter().zip(ranks.iter()) {
            ranks_map.entry(rank).or_default().push(id);
        }
        let max_nodes_per_rank = ranks_map.values().map(|v| v.len()).max().unwrap_or(1);

        // Calculate adaptive spacing
        let (rank_sep, node_sep) = self.calculate_adaptive_spacing(max_rank, max_nodes_per_rank, &config);

        // Layout with calculated spacing
        self.layout_with_spacing(node_ids, edges, rank_sep, node_sep, config.padding)
    }

    /// Layout nodes with specific spacing values.
    fn layout_with_spacing(
        &self,
        node_ids: &[String],
        edges: &[LayoutEdge],
        rank_separation: f64,
        node_separation: f64,
        padding: f64,
    ) -> Vec<LayoutNode> {
        if node_ids.is_empty() {
            return Vec::new();
        }

        let ranks = self.compute_ranks(node_ids, edges);
        let mut ranks_map: std::collections::HashMap<usize, Vec<&String>> = std::collections::HashMap::new();
        for (id, &rank) in node_ids.iter().zip(ranks.iter()) {
            ranks_map.entry(rank).or_default().push(id);
        }

        let mut nodes = Vec::new();
        let max_rank = ranks.iter().max().copied().unwrap_or(0);

        for rank in 0..=max_rank {
            let rank_nodes = ranks_map.get(&rank).map(|v| v.as_slice()).unwrap_or(&[]);
            let rank_count = rank_nodes.len();

            for (idx, id) in rank_nodes.iter().enumerate() {
                let (x, y) = match self.direction {
                    AutoLayoutDirection::TopBottom => {
                        let x = (idx as f64) * (self.default_width + node_separation);
                        let y = (rank as f64) * (self.default_height + rank_separation);
                        (x, y)
                    }
                    AutoLayoutDirection::BottomTop => {
                        let x = (idx as f64) * (self.default_width + node_separation);
                        let y = ((max_rank - rank) as f64) * (self.default_height + rank_separation);
                        (x, y)
                    }
                    AutoLayoutDirection::LeftRight => {
                        let x = (rank as f64) * (self.default_width + rank_separation);
                        let y = (idx as f64) * (self.default_height + node_separation);
                        (x, y)
                    }
                    AutoLayoutDirection::RightLeft => {
                        let x = ((max_rank - rank) as f64) * (self.default_width + rank_separation);
                        let y = (idx as f64) * (self.default_height + node_separation);
                        (x, y)
                    }
                };

                // Center the rank
                let offset = if rank_count > 1 {
                    let total_width = (rank_count as f64 - 1.0) * (self.default_width + node_separation);
                    -total_width / 2.0
                } else {
                    0.0
                };

                let (final_x, final_y) = match self.direction {
                    AutoLayoutDirection::TopBottom | AutoLayoutDirection::BottomTop => {
                        (x + offset + padding, y + padding)
                    }
                    AutoLayoutDirection::LeftRight | AutoLayoutDirection::RightLeft => {
                        (x + padding, y + offset + padding)
                    }
                };

                nodes.push(LayoutNode {
                    id: (*id).clone(),
                    position: Position { x: final_x, y: final_y },
                    size: Size {
                        width: self.default_width,
                        height: self.default_height,
                    },
                    rank,
                });
            }
        }

        nodes
    }

    /// Layout nodes in a simple grid pattern (legacy method, uses fixed spacing).
    pub fn layout(&self, node_ids: &[String], edges: &[LayoutEdge]) -> Vec<LayoutNode> {
        self.layout_with_spacing(node_ids, edges, self.rank_separation, self.node_separation, 50.0)
    }

    /// Layout nodes using the advanced Sugiyama algorithm with crossing minimization.
    ///
    /// This method provides better results for complex diagrams by:
    /// - Properly handling cycles in the graph
    /// - Minimizing edge crossings using barycentric heuristic
    /// - Providing smoother edge routing through dummy nodes
    ///
    /// Returns a `SugiyamaResult` containing positioned nodes and edge routes.
    pub fn layout_sugiyama(
        &self,
        node_ids: &[String],
        node_sizes: &[(String, Size)],
        edges: &[LayoutEdge],
    ) -> advanced::SugiyamaResult {
        let config = advanced::SugiyamaConfig {
            direction: self.direction,
            rank_separation: self.rank_separation,
            node_separation: self.node_separation,
            default_width: self.default_width,
            default_height: self.default_height,
            ..Default::default()
        };

        // Convert to advanced layout edge type
        let advanced_edges: Vec<advanced::LayoutEdge> = edges
            .iter()
            .map(|e| advanced::LayoutEdge {
                source: e.source.clone(),
                target: e.target.clone(),
            })
            .collect();

        // Convert sizes to advanced type
        let advanced_sizes: Vec<(String, advanced::Size)> = node_sizes
            .iter()
            .map(|(id, s)| {
                (
                    id.clone(),
                    advanced::Size {
                        width: s.width,
                        height: s.height,
                    },
                )
            })
            .collect();

        advanced::sugiyama_layout(node_ids, &advanced_sizes, &advanced_edges, &config)
    }

    fn compute_ranks(&self, node_ids: &[String], edges: &[LayoutEdge]) -> Vec<usize> {
        let mut ranks = vec![0; node_ids.len()];
        let id_to_idx: std::collections::HashMap<_, _> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();

        // Filter edges to only include those where both endpoints are in the view
        let filtered_edges: Vec<_> = edges
            .iter()
            .filter(|e| id_to_idx.contains_key(e.source.as_str()) && id_to_idx.contains_key(e.target.as_str()))
            .collect();

        // Maximum possible rank in a DAG is n-1 (longest path through all nodes)
        // If we detect ranks exceeding this, we have cycles and should stop
        let max_possible_rank = node_ids.len().saturating_sub(1).max(1);

        // Simple topological-ish ranking with cycle detection
        let mut changed = true;
        let mut iterations = 0;
        while changed && iterations < 100 {
            changed = false;
            iterations += 1;

            for edge in &filtered_edges {
                if let (Some(&source_idx), Some(&target_idx)) =
                    (id_to_idx.get(edge.source.as_str()), id_to_idx.get(edge.target.as_str()))
                {
                    if ranks[target_idx] <= ranks[source_idx] {
                        let new_rank = ranks[source_idx] + 1;
                        // Cap rank to prevent cycles from spiraling
                        if new_rank <= max_possible_rank {
                            ranks[target_idx] = new_rank;
                            changed = true;
                        }
                    }
                }
            }

            // Early exit if max rank reached (likely cycles present)
            if *ranks.iter().max().unwrap_or(&0) >= max_possible_rank {
                break;
            }
        }

        ranks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_layout() {
        let layout = GridLayout::default();
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

        let result = layout.layout(&nodes, &edges);
        assert_eq!(result.len(), 3);

        // Check that nodes have different ranks
        let ranks: Vec<_> = result.iter().map(|n| n.rank).collect();
        assert!(ranks.contains(&0));
        assert!(ranks.contains(&1));
        assert!(ranks.contains(&2));
    }
}
