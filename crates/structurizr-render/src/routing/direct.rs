//! Direct (straight-line) edge routing.
//!
//! This module computes straight-line paths between elements,
//! finding the intersection points with element boundaries.

use crate::layout::{LayoutNode, Point};

use super::{EdgePath, RoutedEdge};

/// Route edges as direct straight lines.
pub fn route_direct(
    source: &LayoutNode,
    target: &LayoutNode,
) -> EdgePath {
    // Calculate center points
    let src_cx = source.position.x + source.size.width / 2.0;
    let src_cy = source.position.y + source.size.height / 2.0;
    let tgt_cx = target.position.x + target.size.width / 2.0;
    let tgt_cy = target.position.y + target.size.height / 2.0;

    // Find intersection points with element boundaries
    let start = line_rect_intersection(
        tgt_cx,
        tgt_cy,
        src_cx,
        src_cy,
        source.position.x,
        source.position.y,
        source.size.width,
        source.size.height,
    );

    let end = line_rect_intersection(
        src_cx,
        src_cy,
        tgt_cx,
        tgt_cy,
        target.position.x,
        target.position.y,
        target.size.width,
        target.size.height,
    );

    EdgePath::Direct {
        start: Point::new(start.0, start.1),
        end: Point::new(end.0, end.1),
    }
}

/// Route multiple edges as direct lines.
pub fn route_all_direct(
    nodes: &[LayoutNode],
    edges: &[(String, String)],
) -> Vec<RoutedEdge> {
    let node_map: std::collections::HashMap<&str, &LayoutNode> = nodes
        .iter()
        .map(|n| (n.id.as_str(), n))
        .collect();

    edges
        .iter()
        .filter_map(|(src_id, tgt_id)| {
            let source = node_map.get(src_id.as_str())?;
            let target = node_map.get(tgt_id.as_str())?;

            Some(RoutedEdge {
                source_id: src_id.clone(),
                target_id: tgt_id.clone(),
                path: route_direct(source, target),
            })
        })
        .collect()
}

/// Find the intersection point between a line and a rectangle boundary.
///
/// The line goes from (x1, y1) to (x2, y2).
/// The rectangle has top-left corner (rx, ry) with dimensions (rw, rh).
///
/// Returns the point where the line crosses the rectangle boundary,
/// starting from (x1, y1) and going toward (x2, y2).
pub fn line_rect_intersection(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    rx: f64,
    ry: f64,
    rw: f64,
    rh: f64,
) -> (f64, f64) {
    let dx = x2 - x1;
    let dy = y2 - y1;

    // If the line is essentially a point, return the center
    if dx.abs() < 0.001 && dy.abs() < 0.001 {
        return (x1, y1);
    }

    let mut t_min = f64::MAX;
    let mut result = (x1, y1);

    // Check intersection with each edge of the rectangle

    // Left edge (x = rx)
    if dx.abs() > 0.001 {
        let t = (rx - x1) / dx;
        if t > 0.0 && t < t_min {
            let y = y1 + t * dy;
            if y >= ry && y <= ry + rh {
                t_min = t;
                result = (rx, y);
            }
        }
    }

    // Right edge (x = rx + rw)
    if dx.abs() > 0.001 {
        let t = (rx + rw - x1) / dx;
        if t > 0.0 && t < t_min {
            let y = y1 + t * dy;
            if y >= ry && y <= ry + rh {
                t_min = t;
                result = (rx + rw, y);
            }
        }
    }

    // Top edge (y = ry)
    if dy.abs() > 0.001 {
        let t = (ry - y1) / dy;
        if t > 0.0 && t < t_min {
            let x = x1 + t * dx;
            if x >= rx && x <= rx + rw {
                t_min = t;
                result = (x, ry);
            }
        }
    }

    // Bottom edge (y = ry + rh)
    if dy.abs() > 0.001 {
        let t = (ry + rh - y1) / dy;
        if t > 0.0 && t < t_min {
            let x = x1 + t * dx;
            if x >= rx && x <= rx + rw {
                // t_min not updated here since this is the last edge check
                result = (x, ry + rh);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Position, Size};

    #[test]
    fn test_line_rect_intersection_from_above() {
        // Line from above the rectangle going down
        let (x, y) = line_rect_intersection(
            50.0, -50.0,  // start above
            50.0, 150.0,  // end below
            0.0, 0.0,     // rect at origin
            100.0, 100.0, // 100x100 rect
        );

        // Should intersect at top edge
        assert!((y - 0.0).abs() < 0.001, "Expected y=0, got y={}", y);
        assert!((x - 50.0).abs() < 0.001, "Expected x=50, got x={}", x);
    }

    #[test]
    fn test_line_rect_intersection_from_left() {
        let (x, y) = line_rect_intersection(
            -50.0, 50.0,  // start left
            150.0, 50.0,  // end right
            0.0, 0.0,     // rect at origin
            100.0, 100.0, // 100x100 rect
        );

        // Should intersect at left edge
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_line_rect_intersection_diagonal() {
        let (x, y) = line_rect_intersection(
            -50.0, -50.0, // start top-left outside
            150.0, 150.0, // end bottom-right outside
            0.0, 0.0,     // rect at origin
            100.0, 100.0, // 100x100 rect
        );

        // Should intersect at corner (0, 0)
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_route_direct() {
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

        let path = route_direct(&source, &target);

        if let EdgePath::Direct { start, end } = path {
            // Start should be at bottom of source
            assert!((start.y - 50.0).abs() < 0.001);
            // End should be at top of target
            assert!((end.y - 200.0).abs() < 0.001);
        } else {
            panic!("Expected Direct path");
        }
    }
}
