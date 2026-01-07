//! Collision detection and resolution for diagram layout.
//!
//! This module provides:
//! - Quadtree-based spatial indexing for efficient overlap detection
//! - AABB (Axis-Aligned Bounding Box) collision testing
//! - Label placement optimization to avoid overlaps
//! - Node separation enforcement for clean layouts

use std::collections::HashSet;

use crate::layout::{LayoutNode, Point};

/// Axis-Aligned Bounding Box for collision detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl AABB {
    /// Create a new AABB from corner coordinates.
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
        }
    }

    /// Create AABB from center point and dimensions.
    pub fn from_center(center_x: f64, center_y: f64, width: f64, height: f64) -> Self {
        let half_w = width / 2.0;
        let half_h = height / 2.0;
        Self::new(
            center_x - half_w,
            center_y - half_h,
            center_x + half_w,
            center_y + half_h,
        )
    }

    /// Create AABB from a layout node.
    pub fn from_node(node: &LayoutNode) -> Self {
        Self::new(
            node.position.x,
            node.position.y,
            node.position.x + node.size.width,
            node.position.y + node.size.height,
        )
    }

    /// Create AABB from position and size.
    pub fn from_position(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::new(x, y, x + width, y + height)
    }

    /// Get the width of the bounding box.
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Get the height of the bounding box.
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Get the center point.
    pub fn center(&self) -> Point {
        Point::new(
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// Check if this AABB intersects with another.
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
    }

    /// Check if this AABB contains a point.
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Check if this AABB fully contains another AABB.
    pub fn contains(&self, other: &AABB) -> bool {
        self.min_x <= other.min_x
            && self.max_x >= other.max_x
            && self.min_y <= other.min_y
            && self.max_y >= other.max_y
    }

    /// Expand the AABB by a padding amount on all sides.
    pub fn expand(&self, padding: f64) -> Self {
        Self::new(
            self.min_x - padding,
            self.min_y - padding,
            self.max_x + padding,
            self.max_y + padding,
        )
    }

    /// Calculate the overlap area with another AABB.
    pub fn overlap_area(&self, other: &AABB) -> f64 {
        let overlap_x = (self.max_x.min(other.max_x) - self.min_x.max(other.min_x)).max(0.0);
        let overlap_y = (self.max_y.min(other.max_y) - self.min_y.max(other.min_y)).max(0.0);
        overlap_x * overlap_y
    }

    /// Calculate minimum translation vector to separate from another AABB.
    /// Returns (dx, dy) to move this AABB to resolve overlap.
    pub fn separation_vector(&self, other: &AABB) -> Option<(f64, f64)> {
        if !self.intersects(other) {
            return None;
        }

        // Calculate overlap in each direction
        let left = other.max_x - self.min_x;   // Move right
        let right = self.max_x - other.min_x;  // Move left
        let top = other.max_y - self.min_y;    // Move down
        let bottom = self.max_y - other.min_y; // Move up

        // Find minimum separation
        let min_x = if left < right { left } else { -right };
        let min_y = if top < bottom { top } else { -bottom };

        // Return the smaller movement
        if min_x.abs() < min_y.abs() {
            Some((min_x, 0.0))
        } else {
            Some((0.0, min_y))
        }
    }
}

/// An item stored in the quadtree.
#[derive(Debug, Clone)]
pub struct QuadtreeItem {
    pub id: usize,
    pub bounds: AABB,
}

/// Quadtree for efficient spatial queries.
///
/// Provides O(log n) collision queries instead of O(n²).
pub struct Quadtree {
    bounds: AABB,
    items: Vec<QuadtreeItem>,
    children: Option<Box<[Quadtree; 4]>>,
    max_items: usize,
    max_depth: usize,
    depth: usize,
}

impl Quadtree {
    /// Create a new quadtree covering the given bounds.
    pub fn new(bounds: AABB) -> Self {
        Self::with_config(bounds, 8, 8, 0)
    }

    /// Create a quadtree with custom configuration.
    pub fn with_config(bounds: AABB, max_items: usize, max_depth: usize, depth: usize) -> Self {
        Self {
            bounds,
            items: Vec::new(),
            children: None,
            max_items,
            max_depth,
            depth,
        }
    }

    /// Create a quadtree from layout nodes.
    pub fn from_nodes(nodes: &[LayoutNode]) -> Self {
        if nodes.is_empty() {
            return Self::new(AABB::new(0.0, 0.0, 1000.0, 1000.0));
        }

        // Calculate bounds that contain all nodes with padding
        let padding = 100.0;
        let min_x = nodes.iter().map(|n| n.position.x).fold(f64::INFINITY, f64::min) - padding;
        let min_y = nodes.iter().map(|n| n.position.y).fold(f64::INFINITY, f64::min) - padding;
        let max_x = nodes.iter().map(|n| n.position.x + n.size.width).fold(f64::NEG_INFINITY, f64::max) + padding;
        let max_y = nodes.iter().map(|n| n.position.y + n.size.height).fold(f64::NEG_INFINITY, f64::max) + padding;

        let mut tree = Self::new(AABB::new(min_x, min_y, max_x, max_y));

        for (id, node) in nodes.iter().enumerate() {
            tree.insert(QuadtreeItem {
                id,
                bounds: AABB::from_node(node),
            });
        }

        tree
    }

    /// Insert an item into the quadtree.
    pub fn insert(&mut self, item: QuadtreeItem) {
        // If we have children, try to insert into appropriate child
        if self.children.is_some() {
            let quadrant = self.get_quadrant(&item.bounds);
            if let Some(q) = quadrant {
                if let Some(ref mut children) = self.children {
                    children[q].insert(item);
                    return;
                }
            }
        }

        // Item doesn't fit in a single quadrant or no children yet
        self.items.push(item);

        // Subdivide if needed
        if self.children.is_none()
            && self.items.len() > self.max_items
            && self.depth < self.max_depth
        {
            self.subdivide();
        }
    }

    /// Subdivide this node into four children.
    fn subdivide(&mut self) {
        let mid_x = (self.bounds.min_x + self.bounds.max_x) / 2.0;
        let mid_y = (self.bounds.min_y + self.bounds.max_y) / 2.0;

        let children = [
            // Top-left (NW)
            Quadtree::with_config(
                AABB::new(self.bounds.min_x, self.bounds.min_y, mid_x, mid_y),
                self.max_items,
                self.max_depth,
                self.depth + 1,
            ),
            // Top-right (NE)
            Quadtree::with_config(
                AABB::new(mid_x, self.bounds.min_y, self.bounds.max_x, mid_y),
                self.max_items,
                self.max_depth,
                self.depth + 1,
            ),
            // Bottom-left (SW)
            Quadtree::with_config(
                AABB::new(self.bounds.min_x, mid_y, mid_x, self.bounds.max_y),
                self.max_items,
                self.max_depth,
                self.depth + 1,
            ),
            // Bottom-right (SE)
            Quadtree::with_config(
                AABB::new(mid_x, mid_y, self.bounds.max_x, self.bounds.max_y),
                self.max_items,
                self.max_depth,
                self.depth + 1,
            ),
        ];

        self.children = Some(Box::new(children));

        // Redistribute items to children
        let items = std::mem::take(&mut self.items);
        for item in items {
            self.insert(item);
        }
    }

    /// Get the quadrant index for a bounding box (if it fits entirely in one).
    fn get_quadrant(&self, bounds: &AABB) -> Option<usize> {
        let mid_x = (self.bounds.min_x + self.bounds.max_x) / 2.0;
        let mid_y = (self.bounds.min_y + self.bounds.max_y) / 2.0;

        let in_left = bounds.max_x <= mid_x;
        let in_right = bounds.min_x >= mid_x;
        let in_top = bounds.max_y <= mid_y;
        let in_bottom = bounds.min_y >= mid_y;

        match (in_left, in_right, in_top, in_bottom) {
            (true, false, true, false) => Some(0), // NW
            (false, true, true, false) => Some(1), // NE
            (true, false, false, true) => Some(2), // SW
            (false, true, false, true) => Some(3), // SE
            _ => None, // Spans multiple quadrants
        }
    }

    /// Query for all items that might intersect with the given bounds.
    pub fn query(&self, bounds: &AABB) -> Vec<&QuadtreeItem> {
        let mut results = Vec::new();
        self.query_internal(bounds, &mut results);
        results
    }

    fn query_internal<'a>(&'a self, bounds: &AABB, results: &mut Vec<&'a QuadtreeItem>) {
        // Check if query bounds intersects this node
        if !self.bounds.intersects(bounds) {
            return;
        }

        // Add items from this node that intersect
        for item in &self.items {
            if item.bounds.intersects(bounds) {
                results.push(item);
            }
        }

        // Query children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_internal(bounds, results);
            }
        }
    }

    /// Find all pairs of overlapping items.
    pub fn find_overlaps(&self) -> Vec<(usize, usize)> {
        let mut overlaps = HashSet::new();
        self.find_overlaps_internal(&mut overlaps);
        overlaps.into_iter().collect()
    }

    fn find_overlaps_internal(&self, overlaps: &mut HashSet<(usize, usize)>) {
        // Check items in this node against each other
        for i in 0..self.items.len() {
            for j in (i + 1)..self.items.len() {
                if self.items[i].bounds.intersects(&self.items[j].bounds) {
                    let pair = if self.items[i].id < self.items[j].id {
                        (self.items[i].id, self.items[j].id)
                    } else {
                        (self.items[j].id, self.items[i].id)
                    };
                    overlaps.insert(pair);
                }
            }

            // Check against items in children
            if let Some(ref children) = self.children {
                for child in children.iter() {
                    self.check_item_against_tree(&self.items[i], child, overlaps);
                }
            }
        }

        // Recurse into children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.find_overlaps_internal(overlaps);
            }
        }
    }

    fn check_item_against_tree(
        &self,
        item: &QuadtreeItem,
        tree: &Quadtree,
        overlaps: &mut HashSet<(usize, usize)>,
    ) {
        if !tree.bounds.intersects(&item.bounds) {
            return;
        }

        for other in &tree.items {
            if item.id != other.id && item.bounds.intersects(&other.bounds) {
                let pair = if item.id < other.id {
                    (item.id, other.id)
                } else {
                    (other.id, item.id)
                };
                overlaps.insert(pair);
            }
        }

        if let Some(ref children) = tree.children {
            for child in children.iter() {
                self.check_item_against_tree(item, child, overlaps);
            }
        }
    }
}

/// Configuration for label placement.
#[derive(Debug, Clone)]
pub struct LabelPlacementConfig {
    /// Padding around labels
    pub padding: f64,
    /// Preferred positions to try (in order of preference)
    pub preferred_positions: Vec<LabelPosition>,
    /// Weight for overlap penalty
    pub overlap_weight: f64,
    /// Weight for distance from ideal position
    pub distance_weight: f64,
}

impl Default for LabelPlacementConfig {
    fn default() -> Self {
        Self {
            padding: 5.0,
            preferred_positions: vec![
                LabelPosition::Above,
                LabelPosition::Below,
                LabelPosition::Right,
                LabelPosition::Left,
                LabelPosition::AboveRight,
                LabelPosition::AboveLeft,
                LabelPosition::BelowRight,
                LabelPosition::BelowLeft,
            ],
            overlap_weight: 100.0,
            distance_weight: 1.0,
        }
    }
}

/// Possible positions for a label relative to its anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelPosition {
    Above,
    Below,
    Left,
    Right,
    AboveLeft,
    AboveRight,
    BelowLeft,
    BelowRight,
    Center,
}

impl LabelPosition {
    /// Calculate the offset for this position.
    pub fn offset(&self, label_width: f64, label_height: f64, padding: f64) -> (f64, f64) {
        match self {
            LabelPosition::Above => (-label_width / 2.0, -label_height - padding),
            LabelPosition::Below => (-label_width / 2.0, padding),
            LabelPosition::Left => (-label_width - padding, -label_height / 2.0),
            LabelPosition::Right => (padding, -label_height / 2.0),
            LabelPosition::AboveLeft => (-label_width - padding, -label_height - padding),
            LabelPosition::AboveRight => (padding, -label_height - padding),
            LabelPosition::BelowLeft => (-label_width - padding, padding),
            LabelPosition::BelowRight => (padding, padding),
            LabelPosition::Center => (-label_width / 2.0, -label_height / 2.0),
        }
    }
}

/// A label to be placed.
#[derive(Debug, Clone)]
pub struct Label {
    /// Anchor point (where the label should be near)
    pub anchor: Point,
    /// Label dimensions
    pub width: f64,
    pub height: f64,
    /// Current position (top-left corner)
    pub position: Point,
}

impl Label {
    pub fn new(anchor: Point, width: f64, height: f64) -> Self {
        Self {
            anchor,
            width,
            height,
            position: anchor,
        }
    }

    /// Get the bounding box for the label at its current position.
    pub fn bounds(&self) -> AABB {
        AABB::from_position(self.position.x, self.position.y, self.width, self.height)
    }

    /// Set position based on a label position relative to anchor.
    pub fn set_position(&mut self, pos: LabelPosition, padding: f64) {
        let (dx, dy) = pos.offset(self.width, self.height, padding);
        self.position = Point::new(self.anchor.x + dx, self.anchor.y + dy);
    }
}

/// Label placement optimizer.
pub struct LabelPlacer {
    config: LabelPlacementConfig,
}

impl LabelPlacer {
    pub fn new(config: LabelPlacementConfig) -> Self {
        Self { config }
    }

    /// Find the best position for a label given existing obstacles.
    pub fn place_label(&self, label: &mut Label, obstacles: &[AABB]) -> LabelPosition {
        let mut best_position = LabelPosition::Above;
        let mut best_score = f64::MAX;

        for &pos in &self.config.preferred_positions {
            label.set_position(pos, self.config.padding);
            let score = self.score_placement(label, obstacles, pos);

            if score < best_score {
                best_score = score;
                best_position = pos;
            }
        }

        label.set_position(best_position, self.config.padding);
        best_position
    }

    /// Score a label placement (lower is better).
    fn score_placement(&self, label: &Label, obstacles: &[AABB], position: LabelPosition) -> f64 {
        let bounds = label.bounds();
        let mut score = 0.0;

        // Penalty for overlapping obstacles
        for obstacle in obstacles {
            let overlap = bounds.overlap_area(obstacle);
            if overlap > 0.0 {
                score += overlap * self.config.overlap_weight;
            }
        }

        // Penalty for distance from preferred positions
        let preference_index = self.config.preferred_positions
            .iter()
            .position(|&p| p == position)
            .unwrap_or(self.config.preferred_positions.len());
        score += preference_index as f64 * self.config.distance_weight;

        score
    }

    /// Place multiple labels, considering each other.
    pub fn place_labels(&self, labels: &mut [Label], fixed_obstacles: &[AABB]) {
        // Sort labels by some criteria (e.g., importance or size)
        // Place them one by one, adding each placed label as an obstacle

        let mut all_obstacles: Vec<AABB> = fixed_obstacles.to_vec();

        for label in labels.iter_mut() {
            self.place_label(label, &all_obstacles);
            all_obstacles.push(label.bounds().expand(self.config.padding));
        }
    }
}

/// Node separation enforcer.
pub struct NodeSeparator {
    /// Minimum separation between nodes
    pub min_separation: f64,
    /// Maximum iterations for separation algorithm
    pub max_iterations: usize,
    /// Movement damping factor (0-1)
    pub damping: f64,
}

impl Default for NodeSeparator {
    fn default() -> Self {
        Self {
            min_separation: 20.0,
            max_iterations: 100,
            damping: 0.8, // Higher damping for more aggressive separation
        }
    }
}

impl NodeSeparator {
    pub fn new(min_separation: f64) -> Self {
        Self {
            min_separation,
            ..Default::default()
        }
    }

    /// Separate overlapping nodes by pushing them apart.
    ///
    /// Returns the number of overlaps that were resolved.
    pub fn separate_nodes(&self, nodes: &mut [LayoutNode]) -> usize {
        if nodes.len() < 2 {
            return 0;
        }

        let mut total_resolved = 0;

        for _ in 0..self.max_iterations {
            let overlaps = self.find_overlapping_pairs(nodes);
            if overlaps.is_empty() {
                break;
            }

            for (i, j) in &overlaps {
                self.push_apart(nodes, *i, *j);
            }

            total_resolved += overlaps.len();
        }

        total_resolved
    }

    /// Find all pairs of overlapping nodes.
    fn find_overlapping_pairs(&self, nodes: &[LayoutNode]) -> Vec<(usize, usize)> {
        let mut overlaps = Vec::new();
        let separation = self.min_separation;

        for i in 0..nodes.len() {
            let bounds_i = AABB::from_node(&nodes[i]).expand(separation / 2.0);

            for j in (i + 1)..nodes.len() {
                let bounds_j = AABB::from_node(&nodes[j]).expand(separation / 2.0);

                if bounds_i.intersects(&bounds_j) {
                    overlaps.push((i, j));
                }
            }
        }

        overlaps
    }

    /// Push two overlapping nodes apart.
    fn push_apart(&self, nodes: &mut [LayoutNode], i: usize, j: usize) {
        let bounds_i = AABB::from_node(&nodes[i]).expand(self.min_separation / 2.0);
        let bounds_j = AABB::from_node(&nodes[j]).expand(self.min_separation / 2.0);

        if let Some((dx, dy)) = bounds_i.separation_vector(&bounds_j) {
            // separation_vector returns the vector to move bounds_i away from bounds_j
            // Apply half the separation to each node (in opposite directions)
            let half_dx = dx * self.damping / 2.0;
            let half_dy = dy * self.damping / 2.0;

            // Move node i in the direction of separation
            nodes[i].position.x += half_dx;
            nodes[i].position.y += half_dy;
            // Move node j in the opposite direction
            nodes[j].position.x -= half_dx;
            nodes[j].position.y -= half_dy;
        }
    }

    /// Separate nodes using quadtree for efficiency.
    pub fn separate_nodes_quadtree(&self, nodes: &mut [LayoutNode]) -> usize {
        if nodes.len() < 2 {
            return 0;
        }

        let mut total_resolved = 0;

        for _ in 0..self.max_iterations {
            // Build quadtree with expanded bounds
            let items: Vec<QuadtreeItem> = nodes
                .iter()
                .enumerate()
                .map(|(id, node)| QuadtreeItem {
                    id,
                    bounds: AABB::from_node(node).expand(self.min_separation / 2.0),
                })
                .collect();

            // Calculate bounds for quadtree
            let min_x = items.iter().map(|i| i.bounds.min_x).fold(f64::INFINITY, f64::min);
            let min_y = items.iter().map(|i| i.bounds.min_y).fold(f64::INFINITY, f64::min);
            let max_x = items.iter().map(|i| i.bounds.max_x).fold(f64::NEG_INFINITY, f64::max);
            let max_y = items.iter().map(|i| i.bounds.max_y).fold(f64::NEG_INFINITY, f64::max);

            let mut tree = Quadtree::new(AABB::new(min_x, min_y, max_x, max_y));
            for item in items {
                tree.insert(item);
            }

            let overlaps = tree.find_overlaps();
            if overlaps.is_empty() {
                break;
            }

            for (i, j) in &overlaps {
                self.push_apart(nodes, *i, *j);
            }

            total_resolved += overlaps.len();
        }

        total_resolved
    }
}

/// Check if any nodes in the layout overlap.
pub fn has_overlaps(nodes: &[LayoutNode], min_separation: f64) -> bool {
    for i in 0..nodes.len() {
        let bounds_i = AABB::from_node(&nodes[i]).expand(min_separation / 2.0);
        for j in (i + 1)..nodes.len() {
            let bounds_j = AABB::from_node(&nodes[j]).expand(min_separation / 2.0);
            if bounds_i.intersects(&bounds_j) {
                return true;
            }
        }
    }
    false
}

/// Get all overlapping node pairs.
pub fn find_all_overlaps(nodes: &[LayoutNode], min_separation: f64) -> Vec<(usize, usize)> {
    let separator = NodeSeparator::new(min_separation);
    separator.find_overlapping_pairs(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Position, Size};

    fn make_node(id: &str, x: f64, y: f64, width: f64, height: f64) -> LayoutNode {
        LayoutNode {
            id: id.to_string(),
            position: Position { x, y },
            size: Size { width, height },
            rank: 0,
        }
    }

    #[test]
    fn test_aabb_intersection() {
        let a = AABB::new(0.0, 0.0, 100.0, 100.0);
        let b = AABB::new(50.0, 50.0, 150.0, 150.0);
        let c = AABB::new(200.0, 200.0, 300.0, 300.0);

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
        assert!(!c.intersects(&a));
    }

    #[test]
    fn test_aabb_contains() {
        let outer = AABB::new(0.0, 0.0, 100.0, 100.0);
        let inner = AABB::new(25.0, 25.0, 75.0, 75.0);
        let partial = AABB::new(50.0, 50.0, 150.0, 150.0);

        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
        assert!(!outer.contains(&partial));
    }

    #[test]
    fn test_aabb_separation_vector() {
        let a = AABB::new(0.0, 0.0, 100.0, 100.0);
        let b = AABB::new(80.0, 0.0, 180.0, 100.0);

        let sep = a.separation_vector(&b);
        assert!(sep.is_some());

        let (dx, dy) = sep.unwrap();
        // Should suggest moving horizontally to separate
        assert!(dx.abs() > 0.0 || dy.abs() > 0.0);
    }

    #[test]
    fn test_quadtree_query() {
        let mut tree = Quadtree::new(AABB::new(0.0, 0.0, 1000.0, 1000.0));

        tree.insert(QuadtreeItem {
            id: 0,
            bounds: AABB::new(100.0, 100.0, 200.0, 200.0),
        });
        tree.insert(QuadtreeItem {
            id: 1,
            bounds: AABB::new(500.0, 500.0, 600.0, 600.0),
        });

        // Query that should hit first item only
        let results = tree.query(&AABB::new(50.0, 50.0, 250.0, 250.0));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 0);

        // Query that should hit second item only
        let results = tree.query(&AABB::new(450.0, 450.0, 650.0, 650.0));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);

        // Query that should miss both
        let results = tree.query(&AABB::new(300.0, 300.0, 400.0, 400.0));
        assert!(results.is_empty());
    }

    #[test]
    fn test_quadtree_find_overlaps() {
        let mut tree = Quadtree::new(AABB::new(0.0, 0.0, 1000.0, 1000.0));

        // Two overlapping items
        tree.insert(QuadtreeItem {
            id: 0,
            bounds: AABB::new(100.0, 100.0, 200.0, 200.0),
        });
        tree.insert(QuadtreeItem {
            id: 1,
            bounds: AABB::new(150.0, 150.0, 250.0, 250.0),
        });
        // Non-overlapping item
        tree.insert(QuadtreeItem {
            id: 2,
            bounds: AABB::new(500.0, 500.0, 600.0, 600.0),
        });

        let overlaps = tree.find_overlaps();
        assert_eq!(overlaps.len(), 1);
        assert!(overlaps.contains(&(0, 1)));
    }

    #[test]
    fn test_label_placement() {
        let config = LabelPlacementConfig::default();
        let placer = LabelPlacer::new(config);

        let mut label = Label::new(Point::new(100.0, 100.0), 50.0, 20.0);
        let obstacles = vec![
            AABB::new(90.0, 70.0, 110.0, 95.0), // Obstacle above
        ];

        let position = placer.place_label(&mut label, &obstacles);

        // Should not place above since there's an obstacle
        assert_ne!(position, LabelPosition::Above);
    }

    #[test]
    fn test_node_separation() {
        let mut nodes = vec![
            make_node("a", 0.0, 0.0, 100.0, 50.0),
            make_node("b", 80.0, 0.0, 100.0, 50.0), // Overlapping
        ];

        let separator = NodeSeparator::new(10.0);
        let resolved = separator.separate_nodes(&mut nodes);

        assert!(resolved > 0);

        // Check they no longer overlap
        let bounds_a = AABB::from_node(&nodes[0]).expand(5.0);
        let bounds_b = AABB::from_node(&nodes[1]).expand(5.0);
        assert!(!bounds_a.intersects(&bounds_b));
    }

    #[test]
    fn test_node_separation_quadtree() {
        let mut nodes = vec![
            make_node("a", 0.0, 0.0, 100.0, 50.0),
            make_node("b", 80.0, 0.0, 100.0, 50.0),
            make_node("c", 200.0, 0.0, 100.0, 50.0), // Not overlapping
        ];

        let separator = NodeSeparator::new(10.0);
        let resolved = separator.separate_nodes_quadtree(&mut nodes);

        assert!(resolved > 0);
        assert!(!has_overlaps(&nodes, 10.0));
    }

    #[test]
    fn test_has_overlaps() {
        let nodes = vec![
            make_node("a", 0.0, 0.0, 100.0, 50.0),
            make_node("b", 200.0, 0.0, 100.0, 50.0),
        ];

        assert!(!has_overlaps(&nodes, 10.0));

        let overlapping = vec![
            make_node("a", 0.0, 0.0, 100.0, 50.0),
            make_node("b", 50.0, 0.0, 100.0, 50.0),
        ];

        assert!(has_overlaps(&overlapping, 10.0));
    }
}
