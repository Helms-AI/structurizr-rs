# ADR 005: Auto-Layout Algorithm

## Status

Accepted

## Context

When users don't specify explicit coordinates for elements, we need to automatically position them in a readable layout. Requirements:

1. Minimize edge crossings
2. Show hierarchical relationships clearly
3. Support different layout directions
4. Handle various diagram sizes

Several algorithms were considered:

1. **Grid layout** - Simple row/column arrangement
2. **Force-directed** - Physics simulation
3. **Sugiyama algorithm** - Hierarchical layered layout
4. **Orthogonal layout** - Right-angle edges only
5. **Circular layout** - Elements in a circle

## Decision

We chose a **hybrid approach** combining grid layout for simple cases and the Sugiyama algorithm for hierarchical diagrams.

### Grid Layout (Default)

For small diagrams or when relationships don't suggest hierarchy:

```rust
pub struct GridLayout {
    columns: usize,
    spacing: f64,
}

impl GridLayout {
    pub fn compute(&self, elements: &[ElementId]) -> Positions {
        elements.iter().enumerate().map(|(i, id)| {
            let row = i / self.columns;
            let col = i % self.columns;
            (*id, Position {
                x: col as f64 * self.spacing,
                y: row as f64 * self.spacing,
            })
        }).collect()
    }
}
```

### Sugiyama Algorithm (Hierarchical)

For diagrams with clear dependency relationships:

```
Phase 1: Cycle Removal
Phase 2: Layer Assignment
Phase 3: Crossing Reduction
Phase 4: Coordinate Assignment
```

## Consequences

### Positive

- **Predictable**: Same input produces same output
- **Hierarchical**: Shows dependencies clearly
- **Configurable**: Direction and spacing options
- **Fast**: Polynomial time complexity

### Negative

- **Not optimal**: May not minimize all crossings
- **Limited flexibility**: Fixed layout styles
- **Large diagrams**: Can become crowded

### Neutral

- Different from Structurizr Java's layout
- Standard algorithm in graph visualization

## Implementation Details

### Sugiyama Phase 1: Cycle Removal

Temporarily reverse edges to make the graph acyclic:

```rust
fn remove_cycles(graph: &mut Graph) -> Vec<EdgeId> {
    let mut reversed = Vec::new();
    // DFS to find back edges
    for edge in find_back_edges(graph) {
        graph.reverse_edge(edge);
        reversed.push(edge);
    }
    reversed
}
```

### Sugiyama Phase 2: Layer Assignment

Assign nodes to horizontal layers:

```rust
fn assign_layers(graph: &Graph) -> Vec<Vec<NodeId>> {
    let mut layers = Vec::new();
    let mut remaining: HashSet<_> = graph.nodes().collect();

    while !remaining.is_empty() {
        // Nodes with no incoming edges from remaining
        let layer: Vec<_> = remaining.iter()
            .filter(|n| graph.incoming(n)
                .all(|p| !remaining.contains(&p)))
            .cloned()
            .collect();

        for node in &layer {
            remaining.remove(node);
        }
        layers.push(layer);
    }
    layers
}
```

### Sugiyama Phase 3: Crossing Reduction

Minimize edge crossings within layers using barycenter method:

```rust
fn reduce_crossings(layers: &mut Vec<Vec<NodeId>>, graph: &Graph) {
    for _ in 0..MAX_ITERATIONS {
        // Forward pass
        for i in 1..layers.len() {
            sort_by_barycenter(&mut layers[i], &layers[i-1], graph);
        }
        // Backward pass
        for i in (0..layers.len()-1).rev() {
            sort_by_barycenter(&mut layers[i], &layers[i+1], graph);
        }
    }
}

fn barycenter(node: NodeId, adjacent_layer: &[NodeId], graph: &Graph) -> f64 {
    let positions: Vec<_> = graph.neighbors(node)
        .filter_map(|n| adjacent_layer.iter().position(|&x| x == n))
        .collect();

    if positions.is_empty() {
        0.0
    } else {
        positions.iter().sum::<usize>() as f64 / positions.len() as f64
    }
}
```

### Sugiyama Phase 4: Coordinate Assignment

Compute final x/y coordinates:

```rust
fn assign_coordinates(layers: &[Vec<NodeId>], direction: Direction) -> Positions {
    let mut positions = HashMap::new();

    for (layer_idx, layer) in layers.iter().enumerate() {
        for (node_idx, node) in layer.iter().enumerate() {
            let (x, y) = match direction {
                Direction::TopToBottom => (
                    node_idx as f64 * NODE_SPACING,
                    layer_idx as f64 * LAYER_SPACING,
                ),
                Direction::LeftToRight => (
                    layer_idx as f64 * LAYER_SPACING,
                    node_idx as f64 * NODE_SPACING,
                ),
                // ... other directions
            };
            positions.insert(*node, Position { x, y });
        }
    }
    positions
}
```

### Layout Directions

```rust
pub enum LayoutDirection {
    TopToBottom,  // tb (default)
    BottomToTop,  // bt
    LeftToRight,  // lr
    RightToLeft,  // rl
}
```

## DSL Usage

```dsl
views {
    container system "Containers" {
        include *
        autoLayout           // Default: top to bottom
        autoLayout lr        // Left to right
        autoLayout tb 300 200 // With rank/node separation
    }
}
```

## Alternatives Considered

### Force-Directed Layout

**Pros**: Good for dense graphs, natural-looking
**Cons**: Non-deterministic, can oscillate, slow

### Orthogonal Layout

**Pros**: Clean right-angle edges
**Cons**: Complex implementation, can be sparse

### Manual Layout Only

**Pros**: Full user control
**Cons**: Tedious for large diagrams

## References

- [Sugiyama et al., "Methods for Visual Understanding of Hierarchical System Structures"](https://ieeexplore.ieee.org/document/4308636)
- [Graph Drawing: Algorithms for the Visualization of Graphs](https://www.graphdrawing.org/)
