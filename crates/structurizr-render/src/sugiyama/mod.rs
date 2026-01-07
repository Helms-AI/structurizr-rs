//! Advanced layout algorithms for diagram elements.
//!
//! This module provides layout algorithms including:
//! - Grid-based layout (original simple algorithm)
//! - Sugiyama-style layered layout with crossing minimization
//!
//! The layout pipeline consists of several phases:
//! 1. Cycle removal (convert to DAG)
//! 2. Layer (rank) assignment
//! 3. Dummy node insertion for long edges
//! 4. Crossing minimization (node ordering within layers)
//! 5. Coordinate assignment (X/Y positioning)

pub mod cycle_removal;
pub mod dummy;
pub mod graph;
pub mod ordering;
pub mod positioning;
pub mod ranking;

use structurizr_core::view::{AutoLayout, AutoLayoutDirection};

/// C4 view types for view-aware layout configuration.
///
/// Each view type has an optimal default layout direction:
/// - C1 (SystemLandscape/SystemContext): Left-to-right for showing system relationships
/// - C2 (Container): Top-to-bottom for showing container hierarchy
/// - C3 (Component): Left-to-right for showing component interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewType {
    /// System Landscape view (C1) - shows all systems in the enterprise
    SystemLandscape,
    /// System Context view (C1) - shows a system and its external dependencies
    SystemContext,
    /// Container view (C2) - shows containers within a system
    Container,
    /// Component view (C3) - shows components within a container
    Component,
    /// Dynamic view - shows sequence of interactions
    Dynamic,
    /// Deployment view - shows infrastructure and deployments
    Deployment,
    /// Filtered view - derived from another view
    Filtered,
}

pub use graph::{LayeredEdge, LayeredGraph, LayeredNode};

/// Position of an element in the layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Size of an element.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// A node in the layout graph (public output type).
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub position: Position,
    pub size: Size,
    pub rank: usize,
}

/// An edge in the layout graph (public input type).
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutEdge {
    pub source: String,
    pub target: String,
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

/// Configuration for the Sugiyama layout algorithm.
#[derive(Debug, Clone)]
pub struct SugiyamaConfig {
    pub direction: AutoLayoutDirection,
    pub rank_separation: f64,
    pub node_separation: f64,
    pub default_width: f64,
    pub default_height: f64,
    /// Maximum iterations for crossing minimization (0 = auto-detect based on complexity)
    pub max_iterations: usize,
    /// Whether to apply 2-opt local refinement
    pub local_refinement: bool,
    /// Use connectivity-based initial ordering (groups related nodes together)
    pub connectivity_ordering: bool,
    /// Apply force-directed position refinement (pulls connected nodes closer)
    pub force_directed_refinement: bool,
    /// Number of force-directed iterations (higher = better grouping, slower)
    pub force_iterations: usize,
    /// Use adaptive iteration count based on graph complexity
    pub adaptive_iterations: bool,
    /// Use weighted median instead of weighted average for barycenter (more stable)
    pub use_weighted_median: bool,
    /// Apply repulsive forces between non-connected nodes
    pub repulsion_enabled: bool,
}

impl Default for SugiyamaConfig {
    fn default() -> Self {
        Self {
            direction: AutoLayoutDirection::TopBottom,
            rank_separation: 150.0,  // Increased vertical spacing between layers
            node_separation: 150.0,  // Increased horizontal spacing between nodes
            default_width: 400.0,
            default_height: 250.0,
            max_iterations: 24,
            local_refinement: true,
            connectivity_ordering: true,
            force_directed_refinement: true,
            force_iterations: 20,  // Increased from 10 for better convergence
            adaptive_iterations: true,  // Enable adaptive iteration count
            use_weighted_median: true,  // Use median for more stable results
            repulsion_enabled: true,    // Enable repulsion between non-connected nodes
        }
    }
}

impl SugiyamaConfig {
    /// Create config from AutoLayout settings.
    pub fn from_auto_layout(
        direction: AutoLayoutDirection,
        rank_separation: u32,
        node_separation: u32,
    ) -> Self {
        Self {
            direction,
            rank_separation: rank_separation as f64,
            node_separation: node_separation as f64,
            ..Default::default()
        }
    }

    /// Create view-type-aware configuration.
    ///
    /// This method selects the optimal layout direction based on the C4 view type:
    /// - C1 (SystemLandscape/SystemContext): Left-to-right layout
    /// - C2 (Container): Top-to-bottom layout
    /// - C3 (Component): Left-to-right layout
    ///
    /// If an explicit `auto_layout` is provided in the DSL, its direction takes precedence.
    /// Otherwise, the view-type-specific default is used.
    pub fn for_view_type(view_type: ViewType, auto_layout: Option<&AutoLayout>) -> Self {
        // If user explicitly specified a direction in DSL, respect that
        if let Some(al) = auto_layout {
            return Self::from_auto_layout(al.direction, al.rank_separation, al.node_separation);
        }

        // Otherwise, use view-type-specific defaults
        let direction = match view_type {
            // C1 views: left-to-right for showing system relationships horizontally
            ViewType::SystemLandscape | ViewType::SystemContext => AutoLayoutDirection::LeftRight,
            // C2 views: top-to-bottom for showing container hierarchy
            ViewType::Container => AutoLayoutDirection::TopBottom,
            // C3 views: left-to-right for showing component interactions
            ViewType::Component => AutoLayoutDirection::LeftRight,
            // Dynamic views: top-to-bottom for showing sequence flow
            ViewType::Dynamic => AutoLayoutDirection::TopBottom,
            // Deployment views: top-to-bottom for showing infrastructure layers
            ViewType::Deployment => AutoLayoutDirection::TopBottom,
            // Filtered views: default to top-to-bottom
            ViewType::Filtered => AutoLayoutDirection::TopBottom,
        };

        // Adjust spacing based on layout direction for better aesthetics
        let (rank_separation, node_separation) = match direction {
            // Horizontal layouts need more horizontal space
            AutoLayoutDirection::LeftRight | AutoLayoutDirection::RightLeft => (200.0, 150.0),
            // Vertical layouts need more vertical space
            AutoLayoutDirection::TopBottom | AutoLayoutDirection::BottomTop => (150.0, 200.0),
        };

        Self {
            direction,
            rank_separation,
            node_separation,
            ..Default::default()
        }
    }
}

/// Result of the Sugiyama layout algorithm.
#[derive(Debug, Clone)]
pub struct SugiyamaResult {
    /// Positioned nodes (excluding dummy nodes)
    pub nodes: Vec<LayoutNode>,
    /// Edges with their routing information (waypoints for orthogonal routing)
    pub edge_routes: Vec<EdgeRoute>,
}

/// Routing information for a single edge.
#[derive(Debug, Clone)]
pub struct EdgeRoute {
    pub source_id: String,
    pub target_id: String,
    /// Waypoints for the edge path (including start and end points)
    pub waypoints: Vec<Point>,
}

/// Perform Sugiyama layout with crossing minimization.
///
/// This is the main entry point for the advanced layout algorithm.
pub fn sugiyama_layout(
    node_ids: &[String],
    node_sizes: &[(String, Size)],
    edges: &[LayoutEdge],
    config: &SugiyamaConfig,
) -> SugiyamaResult {
    if node_ids.is_empty() {
        return SugiyamaResult {
            nodes: Vec::new(),
            edge_routes: Vec::new(),
        };
    }

    // Build size map
    let size_map: std::collections::HashMap<&str, Size> = node_sizes
        .iter()
        .map(|(id, size)| (id.as_str(), *size))
        .collect();

    // Phase 1: Build layered graph
    let mut graph = LayeredGraph::from_input(node_ids, edges);

    // Phase 2: Remove cycles (make DAG)
    let reversed_edges = cycle_removal::remove_cycles(&mut graph);

    // Phase 3: Assign layers (ranks)
    ranking::assign_layers(&mut graph);
    ranking::normalize_layers(&mut graph);  // Remove gaps between layers

    // Phase 4: Insert dummy nodes for long edges
    dummy::insert_dummy_nodes(&mut graph);

    // Phase 5: Minimize crossings
    ordering::minimize_crossings(&mut graph, config);

    // Phase 6: Assign coordinates
    positioning::assign_coordinates(&mut graph, config, &size_map);

    // Phase 7: Extract results and build edge routes
    let nodes = graph.to_layout_nodes(&size_map, config);
    let edge_routes = graph.compute_edge_routes(&reversed_edges);

    SugiyamaResult { nodes, edge_routes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sugiyama_empty() {
        let result = sugiyama_layout(&[], &[], &[], &SugiyamaConfig::default());
        assert!(result.nodes.is_empty());
        assert!(result.edge_routes.is_empty());
    }

    #[test]
    fn test_sugiyama_single_node() {
        let nodes = vec!["a".to_string()];
        let sizes = vec![("a".to_string(), Size::default())];
        let result = sugiyama_layout(&nodes, &sizes, &[], &SugiyamaConfig::default());
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].id, "a");
    }

    #[test]
    fn test_sugiyama_linear_chain() {
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let sizes: Vec<_> = nodes.iter().map(|id| (id.clone(), Size::default())).collect();
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
        let result = sugiyama_layout(&nodes, &sizes, &edges, &SugiyamaConfig::default());

        assert_eq!(result.nodes.len(), 3);

        // Check ranks are assigned correctly
        let node_a = result.nodes.iter().find(|n| n.id == "a").unwrap();
        let node_b = result.nodes.iter().find(|n| n.id == "b").unwrap();
        let node_c = result.nodes.iter().find(|n| n.id == "c").unwrap();

        assert!(node_a.rank < node_b.rank);
        assert!(node_b.rank < node_c.rank);
    }
}
