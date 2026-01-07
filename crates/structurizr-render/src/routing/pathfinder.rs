//! A* pathfinding for orthogonal edge routing with obstacle avoidance.
//!
//! This module provides grid-based pathfinding that routes edges around
//! diagram elements to avoid collisions and minimize crossings.

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

use crate::layout::{LayoutNode, Point};

/// Grid cell size for pathfinding (smaller = more precise but slower).
const GRID_CELL_SIZE: f64 = 20.0;

/// Padding around obstacles to ensure edges don't touch elements.
const OBSTACLE_PADDING: f64 = 15.0;

/// Grid coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCoord {
    pub x: i32,
    pub y: i32,
}

impl GridCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert from world coordinates to grid coordinates.
    pub fn from_world(x: f64, y: f64) -> Self {
        Self {
            x: (x / GRID_CELL_SIZE).round() as i32,
            y: (y / GRID_CELL_SIZE).round() as i32,
        }
    }

    /// Convert to world coordinates.
    pub fn to_world(&self) -> Point {
        Point::new(
            self.x as f64 * GRID_CELL_SIZE,
            self.y as f64 * GRID_CELL_SIZE,
        )
    }

    /// Manhattan distance to another coordinate.
    pub fn manhattan_distance(&self, other: &GridCoord) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Get orthogonal neighbors (up, down, left, right).
    pub fn neighbors(&self) -> [GridCoord; 4] {
        [
            GridCoord::new(self.x, self.y - 1), // Up
            GridCoord::new(self.x, self.y + 1), // Down
            GridCoord::new(self.x - 1, self.y), // Left
            GridCoord::new(self.x + 1, self.y), // Right
        ]
    }
}

/// A node in the A* search priority queue.
#[derive(Clone)]
struct SearchNode {
    coord: GridCoord,
    g_cost: i32,     // Cost from start
    f_cost: i32,     // g_cost + heuristic
    direction: Option<Direction>, // Direction we came from (for bend penalty)
}

impl Eq for SearchNode {}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.coord == other.coord
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (lower f_cost = higher priority)
        other.f_cost.cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Direction of movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn from_delta(dx: i32, dy: i32) -> Option<Self> {
        match (dx, dy) {
            (0, -1) => Some(Direction::Up),
            (0, 1) => Some(Direction::Down),
            (-1, 0) => Some(Direction::Left),
            (1, 0) => Some(Direction::Right),
            _ => None,
        }
    }
}

/// Obstacle grid for A* pathfinding.
pub struct ObstacleGrid {
    /// Set of blocked grid cells.
    obstacles: HashSet<GridCoord>,
    /// Minimum grid coordinates (for bounds).
    min_x: i32,
    min_y: i32,
    /// Maximum grid coordinates (for bounds).
    max_x: i32,
    max_y: i32,
}

impl ObstacleGrid {
    /// Create a new obstacle grid from layout nodes.
    pub fn from_nodes(nodes: &[LayoutNode], padding: f64) -> Self {
        let mut obstacles = HashSet::new();
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        let total_padding = OBSTACLE_PADDING + padding;

        for node in nodes {
            // Calculate padded bounds
            let left = node.position.x - total_padding;
            let right = node.position.x + node.size.width + total_padding;
            let top = node.position.y - total_padding;
            let bottom = node.position.y + node.size.height + total_padding;

            // Convert to grid coordinates
            let grid_left = (left / GRID_CELL_SIZE).floor() as i32;
            let grid_right = (right / GRID_CELL_SIZE).ceil() as i32;
            let grid_top = (top / GRID_CELL_SIZE).floor() as i32;
            let grid_bottom = (bottom / GRID_CELL_SIZE).ceil() as i32;

            // Mark cells as blocked
            for gx in grid_left..=grid_right {
                for gy in grid_top..=grid_bottom {
                    obstacles.insert(GridCoord::new(gx, gy));
                }
            }

            // Update bounds
            min_x = min_x.min(grid_left - 10);
            min_y = min_y.min(grid_top - 10);
            max_x = max_x.max(grid_right + 10);
            max_y = max_y.max(grid_bottom + 10);
        }

        // Ensure reasonable bounds even for empty/small diagrams
        if min_x == i32::MAX {
            min_x = -10;
            min_y = -10;
            max_x = 100;
            max_y = 100;
        }

        Self {
            obstacles,
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Check if a grid cell is blocked.
    pub fn is_blocked(&self, coord: &GridCoord) -> bool {
        self.obstacles.contains(coord)
    }

    /// Check if a grid cell is within bounds.
    pub fn in_bounds(&self, coord: &GridCoord) -> bool {
        coord.x >= self.min_x
            && coord.x <= self.max_x
            && coord.y >= self.min_y
            && coord.y <= self.max_y
    }

    /// Temporarily unblock cells around source and target for pathfinding.
    pub fn create_clearings(&mut self, source: &LayoutNode, target: &LayoutNode) {
        // Clear cells around connection ports
        let src_center = GridCoord::from_world(
            source.position.x + source.size.width / 2.0,
            source.position.y + source.size.height / 2.0,
        );
        let tgt_center = GridCoord::from_world(
            target.position.x + target.size.width / 2.0,
            target.position.y + target.size.height / 2.0,
        );

        // Don't block the immediate area around centers
        for dx in -2..=2 {
            for dy in -2..=2 {
                self.obstacles.remove(&GridCoord::new(src_center.x + dx, src_center.y + dy));
                self.obstacles.remove(&GridCoord::new(tgt_center.x + dx, tgt_center.y + dy));
            }
        }
    }
}

/// A* pathfinder for orthogonal routing.
pub struct AStarPathfinder {
    grid: ObstacleGrid,
    /// Penalty for changing direction (to minimize bends).
    bend_penalty: i32,
}

impl AStarPathfinder {
    /// Create a new pathfinder with the given obstacle grid.
    pub fn new(grid: ObstacleGrid) -> Self {
        Self {
            grid,
            bend_penalty: 5, // Penalty for each bend to encourage straighter paths
        }
    }

    /// Create pathfinder from layout nodes.
    pub fn from_nodes(nodes: &[LayoutNode]) -> Self {
        let grid = ObstacleGrid::from_nodes(nodes, 0.0);
        Self::new(grid)
    }

    /// Find a path from start to goal using A*.
    ///
    /// Returns a list of waypoints in world coordinates, or None if no path exists.
    pub fn find_path(&self, start: Point, goal: Point) -> Option<Vec<Point>> {
        let start_coord = GridCoord::from_world(start.x, start.y);
        let goal_coord = GridCoord::from_world(goal.x, goal.y);

        // Early exit if start or goal is blocked (shouldn't happen normally)
        // We allow this to let paths start/end inside elements

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<GridCoord, (GridCoord, Option<Direction>)> = HashMap::new();
        let mut g_scores: HashMap<GridCoord, i32> = HashMap::new();

        let start_node = SearchNode {
            coord: start_coord,
            g_cost: 0,
            f_cost: start_coord.manhattan_distance(&goal_coord),
            direction: None,
        };

        open_set.push(start_node);
        g_scores.insert(start_coord, 0);

        let mut iterations = 0;
        const MAX_ITERATIONS: i32 = 50000;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                // Fallback to direct path if search takes too long
                return Some(self.fallback_path(start, goal));
            }

            if current.coord == goal_coord {
                return Some(self.reconstruct_path(&came_from, current.coord, start, goal));
            }

            let current_g = *g_scores.get(&current.coord).unwrap_or(&i32::MAX);
            if current.g_cost > current_g {
                continue; // Skip if we've found a better path
            }

            for neighbor in current.coord.neighbors() {
                // Check bounds
                if !self.grid.in_bounds(&neighbor) {
                    continue;
                }

                // Allow moving to goal even if it's "blocked"
                if self.grid.is_blocked(&neighbor) && neighbor != goal_coord {
                    continue;
                }

                let dx = neighbor.x - current.coord.x;
                let dy = neighbor.y - current.coord.y;
                let new_direction = Direction::from_delta(dx, dy);

                // Calculate movement cost with bend penalty
                let mut move_cost = 1;
                if let (Some(prev_dir), Some(new_dir)) = (current.direction, new_direction) {
                    if prev_dir != new_dir {
                        move_cost += self.bend_penalty;
                    }
                }

                let tentative_g = current_g + move_cost;
                let current_best = *g_scores.get(&neighbor).unwrap_or(&i32::MAX);

                if tentative_g < current_best {
                    came_from.insert(neighbor, (current.coord, current.direction));
                    g_scores.insert(neighbor, tentative_g);

                    let h = neighbor.manhattan_distance(&goal_coord);
                    let neighbor_node = SearchNode {
                        coord: neighbor,
                        g_cost: tentative_g,
                        f_cost: tentative_g + h,
                        direction: new_direction,
                    };
                    open_set.push(neighbor_node);
                }
            }
        }

        // No path found, return fallback
        Some(self.fallback_path(start, goal))
    }

    /// Reconstruct the path from A* search results.
    fn reconstruct_path(
        &self,
        came_from: &HashMap<GridCoord, (GridCoord, Option<Direction>)>,
        goal_coord: GridCoord,
        start: Point,
        goal: Point,
    ) -> Vec<Point> {
        let mut path = Vec::new();
        let mut current = goal_coord;

        // Build path in reverse
        while let Some(&(prev, _)) = came_from.get(&current) {
            path.push(current.to_world());
            current = prev;
        }

        path.reverse();

        // Add exact start and end points
        let mut result = vec![start];
        result.extend(path);
        result.push(goal);

        // Simplify to remove collinear points
        self.simplify_path(result)
    }

    /// Fallback path when A* fails (simple L or Z shape).
    fn fallback_path(&self, start: Point, goal: Point) -> Vec<Point> {
        let mid_y = (start.y + goal.y) / 2.0;
        vec![
            start,
            Point::new(start.x, mid_y),
            Point::new(goal.x, mid_y),
            goal,
        ]
    }

    /// Simplify path by removing collinear points.
    fn simplify_path(&self, path: Vec<Point>) -> Vec<Point> {
        if path.len() <= 2 {
            return path;
        }

        let mut simplified = vec![path[0]];

        for i in 1..path.len() - 1 {
            let prev = simplified.last().unwrap();
            let curr = &path[i];
            let next = &path[i + 1];

            // Check if curr is collinear with prev and next
            let is_horizontal = (prev.y - curr.y).abs() < 0.5 && (curr.y - next.y).abs() < 0.5;
            let is_vertical = (prev.x - curr.x).abs() < 0.5 && (curr.x - next.x).abs() < 0.5;

            if !is_horizontal && !is_vertical {
                simplified.push(*curr);
            }
        }

        simplified.push(*path.last().unwrap());
        simplified
    }
}

/// Route multiple edges with obstacle avoidance.
pub struct ObstacleAwareRouter {
    pathfinder: AStarPathfinder,
}

impl ObstacleAwareRouter {
    /// Create a new router from layout nodes.
    pub fn new(nodes: &[LayoutNode]) -> Self {
        Self {
            pathfinder: AStarPathfinder::from_nodes(nodes),
        }
    }

    /// Find a path between two points, avoiding obstacles.
    pub fn find_path(&self, start: Point, goal: Point) -> Vec<Point> {
        self.pathfinder.find_path(start, goal).unwrap_or_else(|| vec![start, goal])
    }

    /// Find path with specified clearance points.
    ///
    /// The path goes: start -> start_clear -> [A* path] -> goal_clear -> goal
    pub fn find_path_with_clearance(
        &self,
        start: Point,
        start_clearance: Point,
        goal_clearance: Point,
        goal: Point,
    ) -> Vec<Point> {
        // Use A* to find path between clearance points
        let middle_path = self.pathfinder
            .find_path(start_clearance, goal_clearance)
            .unwrap_or_else(|| vec![start_clearance, goal_clearance]);

        // Combine into full path
        let mut full_path = vec![start];

        // Add clearance point if different from start
        if (start_clearance.x - start.x).abs() > 0.5 || (start_clearance.y - start.y).abs() > 0.5 {
            full_path.push(start_clearance);
        }

        // Add middle path (skip first point if it's the same as start_clearance)
        for (i, &p) in middle_path.iter().enumerate() {
            if i == 0 && (p.x - start_clearance.x).abs() < 0.5 && (p.y - start_clearance.y).abs() < 0.5 {
                continue;
            }
            if i == middle_path.len() - 1 && (p.x - goal_clearance.x).abs() < 0.5 && (p.y - goal_clearance.y).abs() < 0.5 {
                continue;
            }
            full_path.push(p);
        }

        // Add goal clearance if different from goal
        if (goal_clearance.x - goal.x).abs() > 0.5 || (goal_clearance.y - goal.y).abs() > 0.5 {
            full_path.push(goal_clearance);
        }

        full_path.push(goal);

        // Final simplification
        self.pathfinder.simplify_path(full_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Position, Size};

    fn make_node(id: &str, x: f64, y: f64) -> LayoutNode {
        LayoutNode {
            id: id.to_string(),
            position: Position { x, y },
            size: Size { width: 100.0, height: 50.0 },
            rank: 0,
        }
    }

    #[test]
    fn test_grid_coord_conversion() {
        let coord = GridCoord::from_world(100.0, 200.0);
        let world = coord.to_world();
        assert!((world.x - 100.0).abs() < GRID_CELL_SIZE);
        assert!((world.y - 200.0).abs() < GRID_CELL_SIZE);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = GridCoord::new(0, 0);
        let b = GridCoord::new(3, 4);
        assert_eq!(a.manhattan_distance(&b), 7);
    }

    #[test]
    fn test_obstacle_grid_creation() {
        let nodes = vec![make_node("a", 100.0, 100.0)];
        let grid = ObstacleGrid::from_nodes(&nodes, 0.0);

        // Center of node should be blocked
        let center = GridCoord::from_world(150.0, 125.0);
        assert!(grid.is_blocked(&center));

        // Far away should not be blocked
        let far = GridCoord::from_world(500.0, 500.0);
        assert!(!grid.is_blocked(&far));
    }

    #[test]
    fn test_pathfinding_no_obstacles() {
        let nodes: Vec<LayoutNode> = vec![];
        let pathfinder = AStarPathfinder::from_nodes(&nodes);

        let start = Point::new(0.0, 0.0);
        let goal = Point::new(100.0, 100.0);

        let path = pathfinder.find_path(start, goal);
        assert!(path.is_some());

        let path = path.unwrap();
        assert!(!path.is_empty());

        // Start and end should be correct
        assert!((path.first().unwrap().x - start.x).abs() < 1.0);
        assert!((path.last().unwrap().x - goal.x).abs() < 1.0);
    }

    #[test]
    fn test_pathfinding_around_obstacle() {
        // Create an obstacle in the middle
        let nodes = vec![make_node("obstacle", 50.0, 50.0)];
        let pathfinder = AStarPathfinder::from_nodes(&nodes);

        let start = Point::new(0.0, 75.0);
        let goal = Point::new(200.0, 75.0);

        let path = pathfinder.find_path(start, goal);
        assert!(path.is_some());

        let path = path.unwrap();

        // Path should go around the obstacle
        assert!(path.len() > 2, "Path should have intermediate waypoints");
    }

    #[test]
    fn test_path_simplification() {
        let nodes: Vec<LayoutNode> = vec![];
        let pathfinder = AStarPathfinder::from_nodes(&nodes);

        // Create a path with collinear points
        let path = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 0.0),
            Point::new(100.0, 0.0), // Collinear - should be removed
            Point::new(100.0, 50.0),
            Point::new(100.0, 100.0), // Collinear - should be removed
        ];

        let simplified = pathfinder.simplify_path(path);
        assert!(simplified.len() < 5, "Collinear points should be removed");
    }
}
