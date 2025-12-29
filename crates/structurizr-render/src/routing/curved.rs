//! Curved (Bezier) edge routing - JointJS-compatible smooth connector.
//!
//! This module computes smooth curved paths between elements using
//! cubic Bezier curves, matching JointJS's 'smooth' connector behavior.
//! The curve shape varies based on edge index to help distinguish parallel edges.

use crate::layout::{LayoutNode, Point};

use super::direct::line_rect_intersection;
use super::EdgePath;

/// Route an edge as a smooth Bezier curve (JointJS 'smooth' connector).
///
/// The curve is created with control points that produce natural-looking
/// curves. When multiple edges exist between the same pair of elements,
/// they are offset perpendicular to the line to avoid overlap.
///
/// # Arguments
/// * `source` - The source element node
/// * `target` - The target element node
/// * `edge_idx_at_source` - Index of this edge among all edges from source
/// * `total_source_edges` - Total number of edges from source
/// * `edge_idx_at_target` - Index of this edge among all edges to target
/// * `total_target_edges` - Total number of edges to target
pub fn route_curved(
    source: &LayoutNode,
    target: &LayoutNode,
    edge_idx_at_source: usize,
    total_source_edges: usize,
    edge_idx_at_target: usize,
    total_target_edges: usize,
) -> EdgePath {
    // Calculate center points
    let src_cx = source.position.x + source.size.width / 2.0;
    let src_cy = source.position.y + source.size.height / 2.0;
    let tgt_cx = target.position.x + target.size.width / 2.0;
    let tgt_cy = target.position.y + target.size.height / 2.0;

    // Find intersection points with element boundaries
    let (start_x, start_y) = line_rect_intersection(
        tgt_cx, tgt_cy, src_cx, src_cy,
        source.position.x, source.position.y,
        source.size.width, source.size.height,
    );

    let (end_x, end_y) = line_rect_intersection(
        src_cx, src_cy, tgt_cx, tgt_cy,
        target.position.x, target.position.y,
        target.size.width, target.size.height,
    );

    // Calculate control points for the cubic Bezier curve
    let (ctrl1, ctrl2) = calculate_cubic_control_points(
        start_x, start_y,
        end_x, end_y,
        edge_idx_at_source,
        total_source_edges,
        edge_idx_at_target,
        total_target_edges,
    );

    // Return curved path with 4 points: start, ctrl1, ctrl2, end (cubic bezier)
    EdgePath::Curved {
        control_points: vec![
            Point::new(start_x, start_y),
            ctrl1,
            ctrl2,
            Point::new(end_x, end_y),
        ],
    }
}

/// Calculate control points for a cubic Bezier curve.
///
/// JointJS's smooth connector uses cubic beziers that create natural
/// S-curves. The control points are positioned to create a smooth
/// transition from the source to the target.
///
/// For parallel edges, control points are offset perpendicular to the
/// main line, similar to JointJS's adjustVertices algorithm.
fn calculate_cubic_control_points(
    start_x: f64, start_y: f64,
    end_x: f64, end_y: f64,
    edge_idx_at_source: usize,
    total_source_edges: usize,
    edge_idx_at_target: usize,
    total_target_edges: usize,
) -> (Point, Point) {
    // Calculate line direction and length
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let length = (dx * dx + dy * dy).sqrt();

    // If points are too close, return straight-line control points
    if length < 10.0 {
        let mid_x = (start_x + end_x) / 2.0;
        let mid_y = (start_y + end_y) / 2.0;
        return (Point::new(mid_x, mid_y), Point::new(mid_x, mid_y));
    }

    // Unit vector along the line
    let ux = dx / length;
    let uy = dy / length;

    // Perpendicular unit vector (rotate 90 degrees)
    let perp_x = -uy;
    let perp_y = ux;

    // Calculate perpendicular offset for parallel edges
    // This matches JointJS's adjustVertices algorithm
    let max_edges = total_source_edges.max(total_target_edges);
    let perpendicular_offset = if max_edges <= 1 {
        0.0
    } else {
        // JointJS uses gap=150 and spreads edges perpendicular to the line
        let avg_idx = (edge_idx_at_source + edge_idx_at_target) as f64 / 2.0;
        let center = (max_edges as f64 - 1.0) / 2.0;
        let normalized = avg_idx - center;

        // Offset pattern similar to JointJS: spread edges 40px apart
        let gap = 40.0;
        normalized * gap
    };

    // Control point distance from endpoints (typically 1/3 of the line length)
    // This creates a smooth S-curve
    let ctrl_distance = length / 3.0;

    // Base curve offset (creates the natural curve shape)
    // Smaller curves for shorter distances
    let base_curve_offset = (length * 0.15).min(50.0).max(10.0);

    // First control point: near start, offset in curve direction
    let ctrl1_x = start_x + ux * ctrl_distance + perp_x * (base_curve_offset + perpendicular_offset);
    let ctrl1_y = start_y + uy * ctrl_distance + perp_y * (base_curve_offset + perpendicular_offset);

    // Second control point: near end, offset in same direction for smooth S-curve
    let ctrl2_x = end_x - ux * ctrl_distance + perp_x * (base_curve_offset + perpendicular_offset);
    let ctrl2_y = end_y - uy * ctrl_distance + perp_y * (base_curve_offset + perpendicular_offset);

    (
        Point::new(ctrl1_x, ctrl1_y),
        Point::new(ctrl2_x, ctrl2_y),
    )
}

/// Route a single edge with simple curved path (no parallel edge handling).
/// Useful for cases where edge indices are not needed.
pub fn route_curved_simple(
    source: &LayoutNode,
    target: &LayoutNode,
) -> EdgePath {
    route_curved(source, target, 0, 1, 0, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Position, Size};

    fn create_node(id: &str, x: f64, y: f64, width: f64, height: f64) -> LayoutNode {
        LayoutNode {
            id: id.to_string(),
            position: Position { x, y },
            size: Size { width, height },
            rank: 0,
        }
    }

    #[test]
    fn test_route_curved_single_edge() {
        let source = create_node("a", 0.0, 0.0, 100.0, 50.0);
        let target = create_node("b", 0.0, 200.0, 100.0, 50.0);

        let path = route_curved(&source, &target, 0, 1, 0, 1);

        if let EdgePath::Curved { control_points } = path {
            assert_eq!(control_points.len(), 4, "Cubic bezier needs 4 points");
            // Start should be at bottom of source
            assert!((control_points[0].y - 50.0).abs() < 0.001);
            // End should be at top of target
            assert!((control_points[3].y - 200.0).abs() < 0.001);
            // Control points should be between start and end
            assert!(control_points[1].y > 50.0 && control_points[1].y < 200.0);
            assert!(control_points[2].y > 50.0 && control_points[2].y < 200.0);
        } else {
            panic!("Expected Curved path");
        }
    }

    #[test]
    fn test_route_curved_multiple_edges_spread() {
        let source = create_node("a", 0.0, 0.0, 100.0, 50.0);
        let target = create_node("b", 0.0, 200.0, 100.0, 50.0);

        // Three parallel edges should be spread out
        let path1 = route_curved(&source, &target, 0, 3, 0, 3);
        let path2 = route_curved(&source, &target, 1, 3, 1, 3);
        let path3 = route_curved(&source, &target, 2, 3, 2, 3);

        if let (
            EdgePath::Curved { control_points: cp1 },
            EdgePath::Curved { control_points: cp2 },
            EdgePath::Curved { control_points: cp3 },
        ) = (path1, path2, path3) {
            // Control points should have different X values (spread perpendicular)
            let diff_12 = (cp1[1].x - cp2[1].x).abs();
            let diff_23 = (cp2[1].x - cp3[1].x).abs();

            // Edges should be spread apart
            assert!(diff_12 > 10.0, "Parallel edges should be spread apart");
            assert!(diff_23 > 10.0, "Parallel edges should be spread apart");
        } else {
            panic!("Expected Curved paths");
        }
    }

    #[test]
    fn test_route_curved_horizontal() {
        let source = create_node("a", 0.0, 100.0, 100.0, 50.0);
        let target = create_node("b", 300.0, 100.0, 100.0, 50.0);

        let path = route_curved_simple(&source, &target);

        if let EdgePath::Curved { control_points } = path {
            assert_eq!(control_points.len(), 4);
            // Start should be at right of source
            assert!((control_points[0].x - 100.0).abs() < 0.001);
            // End should be at left of target
            assert!((control_points[3].x - 300.0).abs() < 0.001);
        } else {
            panic!("Expected Curved path");
        }
    }

    #[test]
    fn test_route_curved_diagonal() {
        let source = create_node("a", 0.0, 0.0, 100.0, 50.0);
        let target = create_node("b", 200.0, 200.0, 100.0, 50.0);

        let path = route_curved_simple(&source, &target);

        if let EdgePath::Curved { control_points } = path {
            assert_eq!(control_points.len(), 4);
            // Control points should create a smooth curve
            // between the diagonal endpoints
        } else {
            panic!("Expected Curved path");
        }
    }
}
