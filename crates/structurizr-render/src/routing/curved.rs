//! Curved (Bezier) edge routing.
//!
//! This module computes smooth curved paths between elements using
//! quadratic Bezier curves. The curve shape varies based on edge index
//! to help distinguish parallel edges.

use crate::layout::{LayoutNode, Point};

use super::direct::line_rect_intersection;
use super::EdgePath;

/// Route an edge as a smooth Bezier curve.
///
/// The curve is created with a control point offset perpendicular to the
/// straight-line path. The offset amount varies based on the edge index
/// to help distinguish multiple edges going to the same target.
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

    // Calculate the control point for the Bezier curve
    let control = calculate_control_point(
        start_x, start_y,
        end_x, end_y,
        edge_idx_at_source,
        total_source_edges,
        edge_idx_at_target,
        total_target_edges,
    );

    // Return curved path with 3 points: start, control, end
    EdgePath::Curved {
        control_points: vec![
            Point::new(start_x, start_y),
            control,
            Point::new(end_x, end_y),
        ],
    }
}

/// Calculate the control point for a quadratic Bezier curve.
///
/// The control point is placed at the midpoint between start and end,
/// then offset perpendicular to the line. The offset varies based on
/// edge indices to separate parallel edges.
fn calculate_control_point(
    start_x: f64, start_y: f64,
    end_x: f64, end_y: f64,
    edge_idx_at_source: usize,
    total_source_edges: usize,
    edge_idx_at_target: usize,
    total_target_edges: usize,
) -> Point {
    // Midpoint between start and end
    let mid_x = (start_x + end_x) / 2.0;
    let mid_y = (start_y + end_y) / 2.0;

    // Calculate line direction
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let length = (dx * dx + dy * dy).sqrt();

    // If points are too close, return midpoint as control
    if length < 1.0 {
        return Point::new(mid_x, mid_y);
    }

    // Perpendicular unit vector (rotate 90 degrees)
    let perp_x = -dy / length;
    let perp_y = dx / length;

    // Calculate curve offset based on edge indices
    // Base offset is 20px, increases for edges with higher indices
    // This helps separate parallel edges visually
    let max_edges = total_source_edges.max(total_target_edges);

    let base_offset = if max_edges <= 1 {
        // Single edge - minimal curve (almost straight)
        15.0
    } else {
        // Multiple edges - vary the curve amount
        // Center index at 0, spread others out
        let avg_idx = (edge_idx_at_source + edge_idx_at_target) as f64 / 2.0;
        let center = (max_edges as f64 - 1.0) / 2.0;
        let normalized = (avg_idx - center) / center.max(1.0);

        // Offset ranges from -40 to +40 based on position
        20.0 + normalized * 30.0
    };

    // Apply perpendicular offset to midpoint
    let control_x = mid_x + perp_x * base_offset;
    let control_y = mid_y + perp_y * base_offset;

    Point::new(control_x, control_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Position, Size};

    #[test]
    fn test_route_curved_single_edge() {
        let source = LayoutNode {
            id: "a".to_string(),
            position: Position { x: 0.0, y: 0.0 },
            size: Size { width: 100.0, height: 50.0 },
            rank: 0,
        };
        let target = LayoutNode {
            id: "b".to_string(),
            position: Position { x: 0.0, y: 200.0 },
            size: Size { width: 100.0, height: 50.0 },
            rank: 1,
        };

        let path = route_curved(&source, &target, 0, 1, 0, 1);

        if let EdgePath::Curved { control_points } = path {
            assert_eq!(control_points.len(), 3);
            // Start should be at bottom of source
            assert!((control_points[0].y - 50.0).abs() < 0.001);
            // End should be at top of target
            assert!((control_points[2].y - 200.0).abs() < 0.001);
            // Control point should be between start and end
            assert!(control_points[1].y > 50.0 && control_points[1].y < 200.0);
        } else {
            panic!("Expected Curved path");
        }
    }

    #[test]
    fn test_route_curved_multiple_edges() {
        let source = LayoutNode {
            id: "a".to_string(),
            position: Position { x: 0.0, y: 0.0 },
            size: Size { width: 100.0, height: 50.0 },
            rank: 0,
        };
        let target = LayoutNode {
            id: "b".to_string(),
            position: Position { x: 0.0, y: 200.0 },
            size: Size { width: 100.0, height: 50.0 },
            rank: 1,
        };

        // Two edges with different indices should have different control points
        let path1 = route_curved(&source, &target, 0, 2, 0, 2);
        let path2 = route_curved(&source, &target, 1, 2, 1, 2);

        if let (EdgePath::Curved { control_points: cp1 }, EdgePath::Curved { control_points: cp2 }) = (path1, path2) {
            // Control points should be different
            let diff_x = (cp1[1].x - cp2[1].x).abs();
            let diff_y = (cp1[1].y - cp2[1].y).abs();
            assert!(diff_x > 1.0 || diff_y > 1.0, "Control points should differ");
        } else {
            panic!("Expected Curved paths");
        }
    }
}
