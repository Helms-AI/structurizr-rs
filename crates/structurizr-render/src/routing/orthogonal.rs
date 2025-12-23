//! Orthogonal (right-angle) edge routing.
//!
//! This module implements channel-based orthogonal routing where edges
//! use only horizontal and vertical segments with 90-degree bends.

use std::collections::HashMap;

use structurizr_core::view::AutoLayoutDirection;

use crate::layout::{LayoutNode, Point};

use super::{EdgePath, Port, PortSide, RoutedEdge};

/// Configuration for the orthogonal router.
#[derive(Debug, Clone)]
pub struct OrthogonalConfig {
    /// Minimum distance from element boundary to first bend
    pub port_clearance: f64,
    /// Minimum distance between parallel edge segments
    pub channel_spacing: f64,
    /// Layout direction (affects port preference)
    pub direction: AutoLayoutDirection,
}

impl Default for OrthogonalConfig {
    fn default() -> Self {
        Self {
            port_clearance: 20.0,
            channel_spacing: 15.0,
            direction: AutoLayoutDirection::TopBottom,
        }
    }
}

impl OrthogonalConfig {
    pub fn for_direction(direction: AutoLayoutDirection) -> Self {
        Self {
            direction,
            ..Default::default()
        }
    }

    /// Get preferred exit port side for this layout direction.
    pub fn preferred_exit_side(&self) -> PortSide {
        match self.direction {
            AutoLayoutDirection::TopBottom => PortSide::Bottom,
            AutoLayoutDirection::BottomTop => PortSide::Top,
            AutoLayoutDirection::LeftRight => PortSide::Right,
            AutoLayoutDirection::RightLeft => PortSide::Left,
        }
    }

    /// Get preferred entry port side for this layout direction.
    pub fn preferred_entry_side(&self) -> PortSide {
        match self.direction {
            AutoLayoutDirection::TopBottom => PortSide::Top,
            AutoLayoutDirection::BottomTop => PortSide::Bottom,
            AutoLayoutDirection::LeftRight => PortSide::Left,
            AutoLayoutDirection::RightLeft => PortSide::Right,
        }
    }
}

/// Orthogonal edge router.
pub struct OrthogonalRouter {
    config: OrthogonalConfig,
}

impl OrthogonalRouter {
    pub fn new(config: OrthogonalConfig) -> Self {
        Self { config }
    }

    /// Route all edges with orthogonal paths.
    pub fn route_all(
        &self,
        nodes: &[LayoutNode],
        edges: &[(String, String)],
    ) -> Vec<RoutedEdge> {
        let node_map: HashMap<&str, &LayoutNode> = nodes
            .iter()
            .map(|n| (n.id.as_str(), n))
            .collect();

        // Group edges by source and target for port distribution
        let mut edges_by_source: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut edges_by_target: HashMap<&str, Vec<&str>> = HashMap::new();

        for (src, tgt) in edges {
            edges_by_source.entry(src.as_str()).or_default().push(tgt.as_str());
            edges_by_target.entry(tgt.as_str()).or_default().push(src.as_str());
        }

        edges
            .iter()
            .filter_map(|(src_id, tgt_id)| {
                let source = node_map.get(src_id.as_str())?;
                let target = node_map.get(tgt_id.as_str())?;

                let src_edges = edges_by_source.get(src_id.as_str()).map(|v| v.as_slice()).unwrap_or(&[]);
                let tgt_edges = edges_by_target.get(tgt_id.as_str()).map(|v| v.as_slice()).unwrap_or(&[]);

                let edge_idx_at_source = src_edges.iter().position(|&t| t == tgt_id.as_str()).unwrap_or(0);
                let edge_idx_at_target = tgt_edges.iter().position(|&s| s == src_id.as_str()).unwrap_or(0);

                let path = self.route_edge(
                    source,
                    target,
                    edge_idx_at_source,
                    src_edges.len(),
                    edge_idx_at_target,
                    tgt_edges.len(),
                );

                Some(RoutedEdge {
                    source_id: src_id.clone(),
                    target_id: tgt_id.clone(),
                    path,
                })
            })
            .collect()
    }

    /// Route a single edge with orthogonal path.
    pub fn route_edge(
        &self,
        source: &LayoutNode,
        target: &LayoutNode,
        edge_idx_at_source: usize,
        total_edges_at_source: usize,
        edge_idx_at_target: usize,
        total_edges_at_target: usize,
    ) -> EdgePath {
        // Determine port sides based on relative positions
        let (source_side, target_side) = self.determine_port_sides(source, target);

        // Calculate port positions
        let source_port = self.calculate_port(
            source,
            source_side,
            edge_idx_at_source,
            total_edges_at_source,
        );
        let target_port = self.calculate_port(
            target,
            target_side,
            edge_idx_at_target,
            total_edges_at_target,
        );

        // Generate the orthogonal path
        let waypoints = self.generate_path(source, target, &source_port, &target_port);

        EdgePath::Orthogonal { waypoints }
    }

    /// Determine which sides of source and target elements to connect.
    fn determine_port_sides(
        &self,
        source: &LayoutNode,
        target: &LayoutNode,
    ) -> (PortSide, PortSide) {
        // Get center positions (cy values reserved for future use)
        let src_cx = source.position.x + source.size.width / 2.0;
        let _src_cy = source.position.y + source.size.height / 2.0;
        let tgt_cx = target.position.x + target.size.width / 2.0;
        let _tgt_cy = target.position.y + target.size.height / 2.0;

        // Same layer (sibling relationship)
        if source.rank == target.rank {
            if src_cx < tgt_cx {
                return (PortSide::Right, PortSide::Left);
            } else {
                return (PortSide::Left, PortSide::Right);
            }
        }

        // Different layers - use layout direction preference
        let is_forward = target.rank > source.rank;

        match self.config.direction {
            AutoLayoutDirection::TopBottom => {
                if is_forward {
                    (PortSide::Bottom, PortSide::Top)
                } else {
                    (PortSide::Top, PortSide::Bottom)
                }
            }
            AutoLayoutDirection::BottomTop => {
                if is_forward {
                    (PortSide::Top, PortSide::Bottom)
                } else {
                    (PortSide::Bottom, PortSide::Top)
                }
            }
            AutoLayoutDirection::LeftRight => {
                if is_forward {
                    (PortSide::Right, PortSide::Left)
                } else {
                    (PortSide::Left, PortSide::Right)
                }
            }
            AutoLayoutDirection::RightLeft => {
                if is_forward {
                    (PortSide::Left, PortSide::Right)
                } else {
                    (PortSide::Right, PortSide::Left)
                }
            }
        }
    }

    /// Calculate port position on an element.
    fn calculate_port(
        &self,
        node: &LayoutNode,
        side: PortSide,
        edge_index: usize,
        total_edges: usize,
    ) -> Port {
        // Calculate offset along the side (distribute ports evenly)
        let offset = if total_edges <= 1 {
            0.5
        } else {
            let margin = 0.2;
            let usable = 1.0 - 2.0 * margin;
            margin + usable * (edge_index as f64 / (total_edges - 1) as f64)
        };

        let (x, y) = match side {
            PortSide::Top => (
                node.position.x + node.size.width * offset,
                node.position.y,
            ),
            PortSide::Bottom => (
                node.position.x + node.size.width * offset,
                node.position.y + node.size.height,
            ),
            PortSide::Left => (
                node.position.x,
                node.position.y + node.size.height * offset,
            ),
            PortSide::Right => (
                node.position.x + node.size.width,
                node.position.y + node.size.height * offset,
            ),
        };

        Port::new(side, x, y)
    }

    /// Generate orthogonal path between two ports.
    fn generate_path(
        &self,
        source: &LayoutNode,
        target: &LayoutNode,
        source_port: &Port,
        target_port: &Port,
    ) -> Vec<Point> {
        let mut waypoints = Vec::new();

        // Start at source port
        waypoints.push(Point::new(source_port.x, source_port.y));

        // Add clearance point from source
        let clear1 = self.add_clearance(source_port);
        waypoints.push(clear1);

        // Route based on relative positions and port sides
        // These methods add waypoints ending at the target clearance point
        if source.rank == target.rank {
            // Same-layer routing (sibling relationship)
            self.route_same_layer(&mut waypoints, source, target, source_port, target_port);
        } else {
            // Different-layer routing
            self.route_different_layers(&mut waypoints, source_port, target_port);
        }

        // End at target port
        waypoints.push(Point::new(target_port.x, target_port.y));

        // Simplify path (remove unnecessary waypoints)
        self.simplify_path(&mut waypoints);

        waypoints
    }

    /// Add clearance point moving away from element.
    fn add_clearance(&self, port: &Port) -> Point {
        let c = self.config.port_clearance;
        match port.side {
            PortSide::Top => Point::new(port.x, port.y - c),
            PortSide::Bottom => Point::new(port.x, port.y + c),
            PortSide::Left => Point::new(port.x - c, port.y),
            PortSide::Right => Point::new(port.x + c, port.y),
        }
    }

    /// Route between elements in the same layer.
    fn route_same_layer(
        &self,
        waypoints: &mut Vec<Point>,
        source: &LayoutNode,
        target: &LayoutNode,
        source_port: &Port,
        target_port: &Port,
    ) {
        // For same-layer, we need to route around one of the elements
        // Go above or below based on available space

        let src_top = source.position.y;
        let tgt_top = target.position.y;
        let src_bot = source.position.y + source.size.height;
        let tgt_bot = target.position.y + target.size.height;

        let top_y = src_top.min(tgt_top);
        let bot_y = src_bot.max(tgt_bot);

        // Decide whether to route above or below
        let go_above = top_y > self.config.port_clearance * 3.0;

        let route_y = if go_above {
            top_y - self.config.port_clearance * 2.0
        } else {
            bot_y + self.config.port_clearance * 2.0
        };

        // Get target clearance position
        let target_clear = self.add_clearance(target_port);

        // Add routing waypoints
        let last = waypoints.last().copied().unwrap_or(Point::new(source_port.x, source_port.y));

        // Move vertically to routing channel
        waypoints.push(Point::new(last.x, route_y));

        // Move horizontally toward target clearance X position
        waypoints.push(Point::new(target_clear.x, route_y));

        // Move vertically to target clearance Y position
        waypoints.push(Point::new(target_clear.x, target_clear.y));
    }

    /// Route between elements in different layers.
    fn route_different_layers(
        &self,
        waypoints: &mut Vec<Point>,
        source_port: &Port,
        target_port: &Port,
    ) {
        let last = waypoints.last().copied().unwrap_or(Point::new(source_port.x, source_port.y));
        let target_clear = self.add_clearance(target_port);

        // Determine if we need an L or Z shape
        let vertical_first = matches!(
            (source_port.side, target_port.side),
            (PortSide::Top, _) | (PortSide::Bottom, _)
        );

        if vertical_first {
            // Move vertically first, then horizontally
            let mid_y = (last.y + target_clear.y) / 2.0;

            // Vertical segment to mid-channel
            waypoints.push(Point::new(last.x, mid_y));

            // Horizontal segment toward target clearance X
            waypoints.push(Point::new(target_clear.x, mid_y));

            // Vertical segment to target clearance Y
            waypoints.push(Point::new(target_clear.x, target_clear.y));
        } else {
            // Move horizontally first, then vertically
            let mid_x = (last.x + target_clear.x) / 2.0;

            // Horizontal segment to mid-channel
            waypoints.push(Point::new(mid_x, last.y));

            // Vertical segment to target clearance Y
            waypoints.push(Point::new(mid_x, target_clear.y));

            // Horizontal segment to target clearance X
            waypoints.push(Point::new(target_clear.x, target_clear.y));
        }
    }

    /// Remove redundant waypoints (collinear points).
    fn simplify_path(&self, waypoints: &mut Vec<Point>) {
        if waypoints.len() <= 2 {
            return;
        }

        let mut simplified = vec![waypoints[0]];

        for i in 1..waypoints.len() - 1 {
            let prev = simplified.last().unwrap();
            let curr = &waypoints[i];
            let next = &waypoints[i + 1];

            // Check if curr is collinear with prev and next
            let is_horizontal = (prev.y - curr.y).abs() < 0.1 && (curr.y - next.y).abs() < 0.1;
            let is_vertical = (prev.x - curr.x).abs() < 0.1 && (curr.x - next.x).abs() < 0.1;

            if !is_horizontal && !is_vertical {
                simplified.push(*curr);
            }
        }

        simplified.push(*waypoints.last().unwrap());
        *waypoints = simplified;
    }
}

/// Route all edges with orthogonal paths using default config.
pub fn route_orthogonal(
    nodes: &[LayoutNode],
    edges: &[(String, String)],
    direction: AutoLayoutDirection,
) -> Vec<RoutedEdge> {
    let config = OrthogonalConfig::for_direction(direction);
    let router = OrthogonalRouter::new(config);
    router.route_all(nodes, edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Position, Size};

    fn make_node(id: &str, x: f64, y: f64, rank: usize) -> LayoutNode {
        LayoutNode {
            id: id.to_string(),
            position: Position { x, y },
            size: Size { width: 100.0, height: 50.0 },
            rank,
        }
    }

    #[test]
    fn test_vertical_routing() {
        let source = make_node("a", 0.0, 0.0, 0);
        let target = make_node("b", 0.0, 200.0, 1);

        let router = OrthogonalRouter::new(OrthogonalConfig::default());
        let path = router.route_edge(&source, &target, 0, 1, 0, 1);

        if let EdgePath::Orthogonal { waypoints } = path {
            // Should have at least start and end
            assert!(waypoints.len() >= 2);

            // All segments should be horizontal or vertical
            for i in 1..waypoints.len() {
                let dx = (waypoints[i].x - waypoints[i - 1].x).abs();
                let dy = (waypoints[i].y - waypoints[i - 1].y).abs();
                assert!(
                    dx < 0.1 || dy < 0.1,
                    "Segment {} is diagonal: dx={}, dy={}",
                    i,
                    dx,
                    dy
                );
            }
        } else {
            panic!("Expected Orthogonal path");
        }
    }

    #[test]
    fn test_same_layer_routing() {
        let source = make_node("a", 0.0, 0.0, 0);
        let target = make_node("b", 200.0, 0.0, 0);

        let router = OrthogonalRouter::new(OrthogonalConfig::default());
        let path = router.route_edge(&source, &target, 0, 1, 0, 1);

        if let EdgePath::Orthogonal { waypoints } = path {
            // Should route around (not through)
            assert!(waypoints.len() >= 4, "Expected at least 4 waypoints for same-layer routing");

            // Check all segments are orthogonal
            for i in 1..waypoints.len() {
                let dx = (waypoints[i].x - waypoints[i - 1].x).abs();
                let dy = (waypoints[i].y - waypoints[i - 1].y).abs();
                assert!(dx < 0.1 || dy < 0.1);
            }
        } else {
            panic!("Expected Orthogonal path");
        }
    }

    #[test]
    fn test_port_distribution() {
        let source = make_node("a", 0.0, 0.0, 0);

        let router = OrthogonalRouter::new(OrthogonalConfig::default());

        // Single edge - should be centered
        let port1 = router.calculate_port(&source, PortSide::Bottom, 0, 1);
        assert!((port1.x - 50.0).abs() < 1.0, "Single port should be centered");

        // Multiple edges - should be distributed
        let port_left = router.calculate_port(&source, PortSide::Bottom, 0, 3);
        let port_mid = router.calculate_port(&source, PortSide::Bottom, 1, 3);
        let port_right = router.calculate_port(&source, PortSide::Bottom, 2, 3);

        assert!(port_left.x < port_mid.x);
        assert!(port_mid.x < port_right.x);
    }

    #[test]
    fn test_backward_edge() {
        // Edge from lower layer to higher layer (backward)
        let source = make_node("a", 0.0, 200.0, 1);
        let target = make_node("b", 0.0, 0.0, 0);

        let router = OrthogonalRouter::new(OrthogonalConfig::default());
        let path = router.route_edge(&source, &target, 0, 1, 0, 1);

        if let EdgePath::Orthogonal { waypoints } = path {
            // Should still produce valid orthogonal path
            assert!(waypoints.len() >= 2);

            // Path should go upward overall
            let start_y = waypoints.first().map(|p| p.y).unwrap_or(0.0);
            let end_y = waypoints.last().map(|p| p.y).unwrap_or(0.0);
            assert!(start_y > end_y, "Backward edge should go upward");
        } else {
            panic!("Expected Orthogonal path");
        }
    }

    #[test]
    fn test_route_all() {
        let nodes = vec![
            make_node("a", 0.0, 0.0, 0),
            make_node("b", 0.0, 200.0, 1),
            make_node("c", 200.0, 200.0, 1),
        ];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("a".to_string(), "c".to_string()),
        ];

        let routes = route_orthogonal(&nodes, &edges, AutoLayoutDirection::TopBottom);

        assert_eq!(routes.len(), 2);

        for route in &routes {
            if let EdgePath::Orthogonal { waypoints } = &route.path {
                assert!(waypoints.len() >= 2);
            } else {
                panic!("Expected Orthogonal paths");
            }
        }
    }
}
