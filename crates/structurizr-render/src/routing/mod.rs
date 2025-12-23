//! Edge routing algorithms for diagram rendering.
//!
//! This module provides different routing strategies for relationship edges:
//! - Direct: Straight lines between elements
//! - Orthogonal: Right-angle paths through channels
//! - Curved: Bezier curves (future)

pub mod direct;
pub mod orthogonal;

use crate::layout::Point;

/// A path for rendering an edge between two elements.
#[derive(Debug, Clone)]
pub enum EdgePath {
    /// Direct straight line from source to target.
    Direct {
        /// Start point (on source element boundary)
        start: Point,
        /// End point (on target element boundary)
        end: Point,
    },
    /// Orthogonal path with right-angle bends.
    Orthogonal {
        /// Waypoints forming the path (including start and end)
        waypoints: Vec<Point>,
    },
    /// Curved Bezier path (future).
    Curved {
        /// Control points for the Bezier curve
        control_points: Vec<Point>,
    },
}

impl EdgePath {
    /// Get the start point of the path.
    pub fn start(&self) -> Point {
        match self {
            EdgePath::Direct { start, .. } => *start,
            EdgePath::Orthogonal { waypoints } => waypoints.first().copied().unwrap_or(Point::new(0.0, 0.0)),
            EdgePath::Curved { control_points } => control_points.first().copied().unwrap_or(Point::new(0.0, 0.0)),
        }
    }

    /// Get the end point of the path.
    pub fn end(&self) -> Point {
        match self {
            EdgePath::Direct { end, .. } => *end,
            EdgePath::Orthogonal { waypoints } => waypoints.last().copied().unwrap_or(Point::new(0.0, 0.0)),
            EdgePath::Curved { control_points } => control_points.last().copied().unwrap_or(Point::new(0.0, 0.0)),
        }
    }

    /// Get total path length.
    pub fn length(&self) -> f64 {
        match self {
            EdgePath::Direct { start, end } => {
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                (dx * dx + dy * dy).sqrt()
            }
            EdgePath::Orthogonal { waypoints } => {
                let mut total = 0.0;
                for i in 1..waypoints.len() {
                    let dx = waypoints[i].x - waypoints[i - 1].x;
                    let dy = waypoints[i].y - waypoints[i - 1].y;
                    total += (dx * dx + dy * dy).sqrt();
                }
                total
            }
            EdgePath::Curved { control_points } => {
                // Approximate length for curves
                let mut total = 0.0;
                for i in 1..control_points.len() {
                    let dx = control_points[i].x - control_points[i - 1].x;
                    let dy = control_points[i].y - control_points[i - 1].y;
                    total += (dx * dx + dy * dy).sqrt();
                }
                total
            }
        }
    }

    /// Convert path to SVG path data string.
    pub fn to_svg_path(&self) -> String {
        match self {
            EdgePath::Direct { start, end } => {
                format!("M {:.1} {:.1} L {:.1} {:.1}", start.x, start.y, end.x, end.y)
            }
            EdgePath::Orthogonal { waypoints } => {
                if waypoints.is_empty() {
                    return String::new();
                }
                let mut path = format!("M {:.1} {:.1}", waypoints[0].x, waypoints[0].y);
                for wp in &waypoints[1..] {
                    path.push_str(&format!(" L {:.1} {:.1}", wp.x, wp.y));
                }
                path
            }
            EdgePath::Curved { control_points } => {
                if control_points.len() < 2 {
                    return String::new();
                }
                // Simple quadratic Bezier
                let mut path = format!("M {:.1} {:.1}", control_points[0].x, control_points[0].y);
                if control_points.len() == 3 {
                    path.push_str(&format!(
                        " Q {:.1} {:.1} {:.1} {:.1}",
                        control_points[1].x, control_points[1].y,
                        control_points[2].x, control_points[2].y
                    ));
                } else {
                    for cp in &control_points[1..] {
                        path.push_str(&format!(" L {:.1} {:.1}", cp.x, cp.y));
                    }
                }
                path
            }
        }
    }

    /// Find the best position for a label along this path.
    ///
    /// Returns (x, y, angle) where angle is the rotation for the label.
    pub fn label_position(&self) -> (f64, f64, f64) {
        match self {
            EdgePath::Direct { start, end } => {
                let mid_x = (start.x + end.x) / 2.0;
                let mid_y = (start.y + end.y) / 2.0;
                let angle = calculate_angle(start, end);
                (mid_x, mid_y, angle)
            }
            EdgePath::Orthogonal { waypoints } => {
                find_orthogonal_label_position(waypoints)
            }
            EdgePath::Curved { control_points } => {
                if control_points.len() < 2 {
                    return (0.0, 0.0, 0.0);
                }
                let mid_idx = control_points.len() / 2;
                let point = &control_points[mid_idx];
                (point.x, point.y, 0.0)
            }
        }
    }
}

/// Calculate angle between two points (in degrees).
fn calculate_angle(start: &Point, end: &Point) -> f64 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let mut angle = dy.atan2(dx).to_degrees();

    // Keep angle between -90 and 90 so text is never upside down
    if angle > 90.0 {
        angle -= 180.0;
    } else if angle < -90.0 {
        angle += 180.0;
    }

    angle
}

/// Find the best position for a label on an orthogonal path.
fn find_orthogonal_label_position(waypoints: &[Point]) -> (f64, f64, f64) {
    if waypoints.len() < 2 {
        return (0.0, 0.0, 0.0);
    }

    // Find the longest segment
    let mut longest_idx = 0;
    let mut longest_len = 0.0;

    for i in 0..waypoints.len() - 1 {
        let dx = waypoints[i + 1].x - waypoints[i].x;
        let dy = waypoints[i + 1].y - waypoints[i].y;
        let len = (dx * dx + dy * dy).sqrt();

        if len > longest_len {
            longest_len = len;
            longest_idx = i;
        }
    }

    // Place label at midpoint of longest segment
    let start = &waypoints[longest_idx];
    let end = &waypoints[longest_idx + 1];

    let mid_x = (start.x + end.x) / 2.0;
    let mid_y = (start.y + end.y) / 2.0;

    // Offset label slightly from the line
    let offset = 12.0;
    let is_horizontal = (end.y - start.y).abs() < 0.1;

    let (label_x, label_y) = if is_horizontal {
        (mid_x, mid_y - offset)
    } else {
        (mid_x + offset, mid_y)
    };

    (label_x, label_y, 0.0)
}

/// A routed edge with its path.
#[derive(Debug, Clone)]
pub struct RoutedEdge {
    pub source_id: String,
    pub target_id: String,
    pub path: EdgePath,
}

/// Which side of an element a port is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// A connection port on an element.
#[derive(Debug, Clone, Copy)]
pub struct Port {
    pub side: PortSide,
    pub x: f64,
    pub y: f64,
}

impl Port {
    pub fn new(side: PortSide, x: f64, y: f64) -> Self {
        Self { side, x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_path_length() {
        let path = EdgePath::Direct {
            start: Point::new(0.0, 0.0),
            end: Point::new(3.0, 4.0),
        };
        assert!((path.length() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_orthogonal_path_length() {
        let path = EdgePath::Orthogonal {
            waypoints: vec![
                Point::new(0.0, 0.0),
                Point::new(100.0, 0.0),
                Point::new(100.0, 50.0),
            ],
        };
        assert!((path.length() - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_direct_svg_path() {
        let path = EdgePath::Direct {
            start: Point::new(10.0, 20.0),
            end: Point::new(100.0, 200.0),
        };
        let svg = path.to_svg_path();
        assert!(svg.contains("M 10.0 20.0"));
        assert!(svg.contains("L 100.0 200.0"));
    }

    #[test]
    fn test_orthogonal_svg_path() {
        let path = EdgePath::Orthogonal {
            waypoints: vec![
                Point::new(0.0, 0.0),
                Point::new(50.0, 0.0),
                Point::new(50.0, 100.0),
            ],
        };
        let svg = path.to_svg_path();
        assert!(svg.starts_with("M 0.0 0.0"));
        assert!(svg.contains("L 50.0 0.0"));
        assert!(svg.contains("L 50.0 100.0"));
    }

    #[test]
    fn test_label_position_horizontal() {
        let path = EdgePath::Orthogonal {
            waypoints: vec![
                Point::new(0.0, 0.0),
                Point::new(100.0, 0.0),
            ],
        };
        let (x, y, _angle) = path.label_position();
        assert!((x - 50.0).abs() < 0.001);
        // Y should be offset above the line
        assert!(y < 0.0);
    }
}
