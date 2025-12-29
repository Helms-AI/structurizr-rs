# Layout Algorithms Implementation

This document details the layout algorithms used in structurizr-rs for positioning elements in diagrams. The system supports three primary layout strategies, with Sugiyama being the most sophisticated.

## Table of Contents

1. [Overview](#overview)
2. [Grid Layout](#grid-layout)
3. [Adaptive Layout](#adaptive-layout)
4. [Sugiyama Hierarchical Layout](#sugiyama-hierarchical-layout)
5. [Layout Direction Support](#layout-direction-support)
6. [Performance Characteristics](#performance-characteristics)
7. [Implementation Details](#implementation-details)

## Overview

The layout system in structurizr-rs provides automatic positioning of elements and relationships in C4 diagrams. Located in `crates/structurizr-render/src/layout.rs` and `crates/structurizr-render/src/sugiyama/`, the system offers progressively sophisticated algorithms.

### Layout Algorithm Hierarchy

```
GridLayout (Base)
    ├── Basic Grid Layout (simple rank-based)
    ├── Adaptive Layout (responsive spacing)
    └── Sugiyama Layout (hierarchical optimization)
```

## Grid Layout

The base layout algorithm provides simple, predictable positioning based on graph topology.

### Configuration

```rust
pub struct GridLayout {
    horizontal_spacing: i32,  // Default: 150px
    vertical_spacing: i32,    // Default: 150px
    element_width: i32,       // Default: 400px
    element_height: i32,      // Default: 250px
}
```

### Algorithm

1. **Topological Sort**: Order nodes based on dependencies
2. **Rank Assignment**: Group nodes into horizontal or vertical ranks
3. **Centering**: Center each rank in the available space
4. **Direction Application**: Apply layout direction transformations

### Implementation

```rust
pub fn layout_with_spacing(
    &self,
    mut nodes: Vec<LayoutNode>,
    edges: Vec<LayoutEdge>,
    direction: Direction,
    h_spacing: i32,
    v_spacing: i32,
) -> Vec<LayoutNode> {
    // Build adjacency list for topological sort
    let adj_list = build_adjacency_list(&edges);

    // Assign ranks via topological traversal
    let ranks = compute_ranks(&nodes, &adj_list);

    // Group nodes by rank
    let mut rank_groups: HashMap<i32, Vec<&mut LayoutNode>> = HashMap::new();
    for node in &mut nodes {
        let rank = ranks.get(&node.id).copied().unwrap_or(0);
        rank_groups.entry(rank).push(node);
    }

    // Position each rank
    for (rank, nodes_in_rank) in rank_groups {
        let rank_width = nodes_in_rank.len() as i32 * (self.element_width + h_spacing);
        let start_x = -(rank_width / 2) + (self.element_width / 2);

        for (i, node) in nodes_in_rank.iter_mut().enumerate() {
            node.x = start_x + (i as i32 * (self.element_width + h_spacing));
            node.y = rank * (self.element_height + v_spacing);
        }
    }

    apply_direction(nodes, direction)
}
```

### Characteristics

- **Complexity**: O(V + E) for topological sort
- **Predictability**: Deterministic results
- **Quality**: Basic, may have edge crossings
- **Use Case**: Simple diagrams with few relationships

## Adaptive Layout

Adaptive layout dynamically adjusts spacing based on diagram complexity and element count.

### Algorithm

1. **Complexity Analysis**: Assess diagram structure
2. **Spacing Calculation**: Compute optimal spacing
3. **Viewport Fitting**: Ensure diagram fits target viewport
4. **Fallback**: Use Sugiyama for complex diagrams

### Spacing Calculation

```rust
pub fn layout_adaptive(
    &self,
    nodes: Vec<LayoutNode>,
    edges: Vec<LayoutEdge>,
    direction: Direction,
) -> Vec<LayoutNode> {
    let element_count = nodes.len();

    // Single element special case
    if element_count == 1 {
        let mut single = nodes;
        single[0].x = 0;
        single[0].y = 0;
        return single;
    }

    // Calculate adaptive spacing
    let (h_spacing, v_spacing) = calculate_adaptive_spacing(element_count);

    // For complex diagrams, use Sugiyama
    if element_count > 8 || edges.len() > element_count * 2 {
        return self.layout_sugiyama(nodes, edges, direction);
    }

    // Use grid layout with calculated spacing
    self.layout_with_spacing(nodes, edges, direction, h_spacing, v_spacing)
}

fn calculate_adaptive_spacing(element_count: usize) -> (i32, i32) {
    const TARGET_WIDTH: i32 = 2400;
    const TARGET_HEIGHT: i32 = 1800;
    const MIN_SPACING: i32 = 150;
    const MAX_SPACING: i32 = 300;
    const PADDING: i32 = 80;

    // Estimate grid dimensions
    let cols = (element_count as f32).sqrt().ceil() as i32;
    let rows = ((element_count as f32) / (cols as f32)).ceil() as i32;

    // Calculate spacing to fit viewport
    let h_spacing = ((TARGET_WIDTH - 2 * PADDING) / cols - 400).clamp(MIN_SPACING, MAX_SPACING);
    let v_spacing = ((TARGET_HEIGHT - 2 * PADDING) / rows - 250).clamp(MIN_SPACING, MAX_SPACING);

    (h_spacing, v_spacing)
}
```

### Characteristics

- **Complexity**: O(V + E) for simple cases, O(V²) for Sugiyama fallback
- **Responsiveness**: Adapts to diagram size
- **Quality**: Better than basic grid, uses Sugiyama for complex diagrams
- **Use Case**: General-purpose layout

## Sugiyama Hierarchical Layout

The Sugiyama algorithm (also known as layered graph drawing) provides high-quality hierarchical layouts with minimal edge crossings.

### Overview

Located in `crates/structurizr-render/src/sugiyama/`, this implementation follows the classic five-phase approach:

1. **Cycle Removal**: Convert to DAG
2. **Rank Assignment**: Assign layers
3. **Vertex Ordering**: Minimize crossings
4. **Coordinate Assignment**: Position nodes
5. **Edge Routing**: Draw edges

### Configuration

```rust
pub struct SugiyamaConfig {
    pub direction: Direction,
    pub rank_separation: i32,      // Default: 150px
    pub node_separation: i32,      // Default: 150px
    pub max_iterations: usize,     // Default: 24
    pub local_refinement: bool,    // Default: true
    pub connectivity_ordering: bool, // Default: true
    pub force_directed_refinement: bool, // Default: true
    pub force_iterations: usize,   // Default: 10
    pub element_width: i32,        // Default: 400px
    pub element_height: i32,       // Default: 250px
}
```

### Phase 1: Cycle Removal

Converts cyclic graphs to directed acyclic graphs (DAGs).

```rust
// crates/structurizr-render/src/sugiyama/cycle_removal.rs
pub fn remove_cycles(edges: &[LayoutEdge]) -> (Vec<LayoutEdge>, Vec<usize>) {
    let mut dag_edges = Vec::new();
    let mut reversed_indices = Vec::new();

    // Build strongly connected components
    let sccs = find_strongly_connected_components(edges);

    for (i, edge) in edges.iter().enumerate() {
        if creates_cycle(&dag_edges, edge) {
            // Reverse edge to break cycle
            dag_edges.push(LayoutEdge {
                source: edge.target.clone(),
                target: edge.source.clone(),
            });
            reversed_indices.push(i);
        } else {
            dag_edges.push(edge.clone());
        }
    }

    (dag_edges, reversed_indices)
}
```

### Phase 2: Rank Assignment

Assigns nodes to hierarchical layers.

```rust
// crates/structurizr-render/src/sugiyama/ranking.rs
pub fn assign_ranks(nodes: &[LayoutNode], edges: &[LayoutEdge]) -> HashMap<String, usize> {
    let mut ranks = HashMap::new();

    // Find sources (nodes with no incoming edges)
    let sources = find_source_nodes(nodes, edges);

    // BFS from sources
    let mut queue = VecDeque::new();
    for source in sources {
        ranks.insert(source.clone(), 0);
        queue.push_back(source);
    }

    while let Some(node_id) = queue.pop_front() {
        let current_rank = ranks[&node_id];

        // Process outgoing edges
        for edge in edges.iter().filter(|e| e.source == node_id) {
            let target_rank = current_rank + 1;
            ranks.entry(edge.target.clone())
                .and_modify(|r| *r = (*r).max(target_rank))
                .or_insert(target_rank);

            if !queue.contains(&edge.target) {
                queue.push_back(edge.target.clone());
            }
        }
    }

    ranks
}
```

### Phase 3: Crossing Minimization

Reorders nodes within ranks to minimize edge crossings using the barycentric method.

```rust
// crates/structurizr-render/src/sugiyama/ordering.rs
pub fn minimize_crossings(
    nodes: &mut [LayoutNode],
    edges: &[LayoutEdge],
    config: &SugiyamaConfig,
) {
    let mut best_ordering = nodes.to_vec();
    let mut best_crossings = count_crossings(nodes, edges);

    for iteration in 0..config.max_iterations {
        // Sweep down
        sweep_layer(nodes, edges, Direction::Down);

        // Sweep up
        sweep_layer(nodes, edges, Direction::Up);

        let current_crossings = count_crossings(nodes, edges);
        if current_crossings < best_crossings {
            best_crossings = current_crossings;
            best_ordering = nodes.to_vec();
        }

        // Early termination if no crossings
        if current_crossings == 0 {
            break;
        }
    }

    // Local refinement: 2-opt swaps
    if config.local_refinement {
        local_refinement(&mut best_ordering, edges);
    }

    *nodes = best_ordering;
}

fn sweep_layer(nodes: &mut [LayoutNode], edges: &[LayoutEdge], direction: Direction) {
    // Group nodes by rank
    let mut ranks: HashMap<usize, Vec<&mut LayoutNode>> = HashMap::new();

    // Sort each rank by barycenter
    for (rank, nodes_in_rank) in ranks {
        nodes_in_rank.sort_by_key(|node| {
            calculate_barycenter(&node.id, edges, direction)
        });
    }
}

fn calculate_barycenter(
    node_id: &str,
    edges: &[LayoutEdge],
    direction: Direction,
) -> i32 {
    let connected = match direction {
        Direction::Down => edges.iter()
            .filter(|e| e.target == node_id)
            .map(|e| &e.source),
        Direction::Up => edges.iter()
            .filter(|e| e.source == node_id)
            .map(|e| &e.target),
    };

    // Average position of connected nodes
    let positions: Vec<i32> = connected.map(|id| get_position(id)).collect();
    if positions.is_empty() {
        0
    } else {
        positions.iter().sum::<i32>() / positions.len() as i32
    }
}
```

### Phase 4: Coordinate Assignment

Assigns final x,y coordinates to nodes.

```rust
// crates/structurizr-render/src/sugiyama/positioning.rs
pub fn assign_coordinates(
    nodes: &mut [LayoutNode],
    config: &SugiyamaConfig,
) {
    // Group by rank
    let mut ranks: BTreeMap<usize, Vec<&mut LayoutNode>> = BTreeMap::new();

    for (rank, mut nodes_in_rank) in ranks {
        // Sort by order within rank
        nodes_in_rank.sort_by_key(|n| n.order);

        // Assign x coordinates with spacing
        let total_width = nodes_in_rank.len() as i32
            * (config.element_width + config.node_separation);
        let start_x = -(total_width / 2) + (config.element_width / 2);

        for (i, node) in nodes_in_rank.iter_mut().enumerate() {
            node.x = start_x + (i as i32 * (config.element_width + config.node_separation));
            node.y = rank as i32 * (config.element_height + config.rank_separation);
        }
    }

    // Apply force-directed refinement
    if config.force_directed_refinement {
        apply_forces(nodes, config);
    }
}

fn apply_forces(nodes: &mut [LayoutNode], config: &SugiyamaConfig) {
    const SPRING_CONSTANT: f32 = 0.1;
    const REPULSION_CONSTANT: f32 = 1000.0;
    const DAMPING: f32 = 0.95;

    for _ in 0..config.force_iterations {
        let mut forces: HashMap<String, (f32, f32)> = HashMap::new();

        // Calculate repulsive forces between nodes in same rank
        for node1 in nodes.iter() {
            for node2 in nodes.iter() {
                if node1.id != node2.id && node1.rank == node2.rank {
                    let dx = node2.x as f32 - node1.x as f32;
                    let distance = dx.abs().max(1.0);
                    let force = REPULSION_CONSTANT / (distance * distance);

                    let fx = if dx > 0.0 { -force } else { force };
                    forces.entry(node1.id.clone())
                        .and_modify(|(fx_sum, _)| *fx_sum += fx)
                        .or_insert((fx, 0.0));
                }
            }
        }

        // Apply forces with damping
        for node in nodes.iter_mut() {
            if let Some((fx, _)) = forces.get(&node.id) {
                node.x += (fx * DAMPING) as i32;
            }
        }
    }
}
```

### Phase 5: Edge Routing

Routes edges between positioned nodes (handled by the routing module).

### Normalization

After Sugiyama layout, positions are normalized to start from (0,0):

```rust
fn normalize_positions(nodes: &mut [LayoutNode]) {
    if nodes.is_empty() {
        return;
    }

    // Find minimum coordinates
    let min_x = nodes.iter().map(|n| n.x).min().unwrap_or(0);
    let min_y = nodes.iter().map(|n| n.y).min().unwrap_or(0);

    // Shift all nodes
    for node in nodes {
        node.x = (node.x - min_x) + 100; // Add padding
        node.y = (node.y - min_y) + 100;
    }
}
```

## Layout Direction Support

All layout algorithms support four directions:

```rust
pub enum Direction {
    TopBottom,    // Default
    BottomTop,
    LeftRight,
    RightLeft,
}

fn apply_direction(mut nodes: Vec<LayoutNode>, direction: Direction) -> Vec<LayoutNode> {
    match direction {
        Direction::TopBottom => nodes, // No transformation needed
        Direction::BottomTop => {
            let max_y = nodes.iter().map(|n| n.y).max().unwrap_or(0);
            for node in &mut nodes {
                node.y = max_y - node.y;
            }
            nodes
        }
        Direction::LeftRight => {
            for node in &mut nodes {
                std::mem::swap(&mut node.x, &mut node.y);
            }
            nodes
        }
        Direction::RightLeft => {
            let max_x = nodes.iter().map(|n| n.x).max().unwrap_or(0);
            for node in &mut nodes {
                std::mem::swap(&mut node.x, &mut node.y);
                node.x = max_x - node.x;
            }
            nodes
        }
    }
}
```

## Performance Characteristics

### Comparison Table

| Algorithm | Time Complexity | Space Complexity | Quality | Use Case |
|-----------|----------------|------------------|---------|----------|
| Grid Layout | O(V + E) | O(V) | Basic | Simple diagrams |
| Adaptive | O(V + E) to O(V²) | O(V) | Good | General purpose |
| Sugiyama | O(V² × iterations) | O(V + E) | Excellent | Complex hierarchies |

### Optimization Strategies

1. **Early Termination**: Stop if no crossings detected
2. **Incremental Updates**: Reuse previous layout as starting point
3. **Parallel Processing**: Independent rank processing
4. **Caching**: Store computed barycenters
5. **Heuristics**: Use connectivity for initial ordering

## Implementation Details

### Integration with Rendering

The layout system integrates with the SVG renderer through:

```rust
// In svg.rs
pub fn render_with_layout(&self, workspace: &Workspace, view: &View) -> String {
    // Extract nodes and edges
    let (nodes, edges) = extract_graph(workspace, view);

    // Apply layout
    let layout = GridLayout::default();
    let positioned_nodes = layout.layout_adaptive(nodes, edges, Direction::TopBottom);

    // Check for explicit positions
    let explicit_positions = extract_explicit_positions(&view.properties);
    apply_explicit_positions(&mut positioned_nodes, &explicit_positions);

    // Render SVG
    self.render_nodes_and_edges(positioned_nodes, edges)
}
```

### Position Persistence

Layout results can be overridden by explicit positions:

```rust
fn apply_explicit_positions(
    nodes: &mut [LayoutNode],
    positions: &HashMap<String, (i32, i32)>,
) {
    for node in nodes {
        if let Some(&(x, y)) = positions.get(&node.id) {
            node.x = x;
            node.y = y;
        }
    }
}
```

### Error Handling

Layout algorithms handle edge cases gracefully:

- Empty graphs: Return empty layout
- Single node: Center at origin
- Disconnected components: Layout separately, then combine
- Cycles: Detect and break via edge reversal

## Future Enhancements

### Planned Improvements

1. **Orthogonal Layout**: Grid-based routing for cleaner edges
2. **Force-Directed Layout**: Physics-based for organic appearance
3. **Constraint-Based Layout**: User-defined positioning rules
4. **Incremental Layout**: Efficient updates for dynamic diagrams
5. **Layout Metrics**: Crossing count, edge length, aspect ratio

### Extension Points

```rust
// Custom layout algorithm trait
pub trait LayoutAlgorithm {
    fn layout(
        &self,
        nodes: Vec<LayoutNode>,
        edges: Vec<LayoutEdge>,
        config: &LayoutConfig,
    ) -> LayoutResult;
}

// Register custom algorithm
layout_registry.register("custom", Box::new(CustomLayout));
```

## Related Documentation

- [SVG Rendering Pipeline](svg-rendering-pipeline.md) - Main rendering system
- [Edge Routing](edge-routing.md) - How edges are drawn between nodes
- [Coordinate Systems](coordinate-systems.md) - Positioning and transformations
- [Drag-and-Drop Implementation](drag-drop-implementation.md) - Manual positioning