//! Layered graph data structure for Sugiyama layout.

use std::collections::HashMap;

use super::{EdgeRoute, LayoutEdge, LayoutNode, Point, Position, Size, SugiyamaConfig};

/// A node in the layered graph.
#[derive(Debug, Clone)]
pub struct LayeredNode {
    /// Original element ID (or generated ID for dummy nodes)
    pub id: String,
    /// Internal index in the nodes vector
    pub index: usize,
    /// Layer (rank) assignment
    pub layer: usize,
    /// Position within the layer (for ordering)
    pub position_in_layer: usize,
    /// X coordinate (assigned during positioning)
    pub x: f64,
    /// Y coordinate (assigned during positioning)
    pub y: f64,
    /// Whether this is a dummy node for long edge routing
    pub is_dummy: bool,
    /// For dummy nodes: the original edge index this belongs to
    pub original_edge_index: Option<usize>,
}

impl LayeredNode {
    /// Create a new real (non-dummy) node.
    pub fn new(id: String, index: usize) -> Self {
        Self {
            id,
            index,
            layer: 0,
            position_in_layer: 0,
            x: 0.0,
            y: 0.0,
            is_dummy: false,
            original_edge_index: None,
        }
    }

    /// Create a dummy node for long edge routing.
    pub fn dummy(index: usize, layer: usize, original_edge_index: usize) -> Self {
        Self {
            id: format!("__dummy_{}_{}", original_edge_index, layer),
            index,
            layer,
            position_in_layer: 0,
            x: 0.0,
            y: 0.0,
            is_dummy: true,
            original_edge_index: Some(original_edge_index),
        }
    }
}

/// An edge in the layered graph.
#[derive(Debug, Clone)]
pub struct LayeredEdge {
    /// Source node index
    pub source: usize,
    /// Target node index
    pub target: usize,
    /// Whether this edge was reversed for DAG conversion
    pub reversed: bool,
    /// Original edge index (for tracking through dummy node splitting)
    pub original_index: usize,
}

impl LayeredEdge {
    pub fn new(source: usize, target: usize, original_index: usize) -> Self {
        Self {
            source,
            target,
            reversed: false,
            original_index,
        }
    }
}

/// The main layered graph structure for Sugiyama layout.
#[derive(Debug, Clone)]
pub struct LayeredGraph {
    /// All nodes (including dummy nodes after insertion)
    pub nodes: Vec<LayeredNode>,
    /// All edges (including split edges after dummy insertion)
    pub edges: Vec<LayeredEdge>,
    /// Map from original ID to node index
    pub id_to_index: HashMap<String, usize>,
    /// Nodes grouped by layer: layers[layer_index] = [node_indices]
    pub layers: Vec<Vec<usize>>,
    /// Original edges (before any modifications)
    pub original_edges: Vec<LayoutEdge>,
    /// Number of original (non-dummy) nodes
    pub original_node_count: usize,
}

impl LayeredGraph {
    /// Create a new layered graph from input nodes and edges.
    pub fn from_input(node_ids: &[String], edges: &[LayoutEdge]) -> Self {
        let mut nodes = Vec::with_capacity(node_ids.len());
        let mut id_to_index = HashMap::with_capacity(node_ids.len());

        for (i, id) in node_ids.iter().enumerate() {
            nodes.push(LayeredNode::new(id.clone(), i));
            id_to_index.insert(id.clone(), i);
        }

        let layered_edges: Vec<_> = edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                let source = id_to_index.get(&e.source)?;
                let target = id_to_index.get(&e.target)?;
                Some(LayeredEdge::new(*source, *target, i))
            })
            .collect();

        Self {
            nodes,
            edges: layered_edges,
            id_to_index,
            layers: Vec::new(),
            original_edges: edges.to_vec(),
            original_node_count: node_ids.len(),
        }
    }

    /// Get the number of nodes (including dummy nodes).
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of layers.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get the maximum layer index.
    pub fn max_layer(&self) -> usize {
        self.nodes.iter().map(|n| n.layer).max().unwrap_or(0)
    }

    /// Build layers from node layer assignments.
    pub fn build_layers(&mut self) {
        let max_layer = self.max_layer();
        self.layers = vec![Vec::new(); max_layer + 1];

        for node in &self.nodes {
            self.layers[node.layer].push(node.index);
        }

        // Initialize position_in_layer
        for (_layer_idx, layer) in self.layers.iter().enumerate() {
            for (pos, &node_idx) in layer.iter().enumerate() {
                self.nodes[node_idx].position_in_layer = pos;
            }
        }
    }

    /// Get incoming edges for a node.
    pub fn incoming_edges(&self, node_idx: usize) -> Vec<&LayeredEdge> {
        self.edges.iter().filter(|e| e.target == node_idx).collect()
    }

    /// Get outgoing edges for a node.
    pub fn outgoing_edges(&self, node_idx: usize) -> Vec<&LayeredEdge> {
        self.edges.iter().filter(|e| e.source == node_idx).collect()
    }

    /// Get predecessor node indices.
    pub fn predecessors(&self, node_idx: usize) -> Vec<usize> {
        self.incoming_edges(node_idx)
            .iter()
            .map(|e| e.source)
            .collect()
    }

    /// Get successor node indices.
    pub fn successors(&self, node_idx: usize) -> Vec<usize> {
        self.outgoing_edges(node_idx)
            .iter()
            .map(|e| e.target)
            .collect()
    }

    /// Update layer assignments from the layers vector.
    pub fn sync_layers_to_nodes(&mut self) {
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            for (pos, &node_idx) in layer.iter().enumerate() {
                self.nodes[node_idx].layer = layer_idx;
                self.nodes[node_idx].position_in_layer = pos;
            }
        }
    }

    /// Convert to output LayoutNode format.
    pub fn to_layout_nodes(
        &self,
        size_map: &HashMap<&str, Size>,
        config: &SugiyamaConfig,
    ) -> Vec<LayoutNode> {
        self.nodes
            .iter()
            .filter(|n| !n.is_dummy)
            .map(|n| {
                let size = size_map
                    .get(n.id.as_str())
                    .copied()
                    .unwrap_or(Size {
                        width: config.default_width,
                        height: config.default_height,
                    });

                LayoutNode {
                    id: n.id.clone(),
                    position: Position { x: n.x, y: n.y },
                    size,
                    rank: n.layer,
                }
            })
            .collect()
    }

    /// Compute edge routes from the positioned graph.
    ///
    /// This extracts waypoints through dummy nodes to create routed edges.
    pub fn compute_edge_routes(&self, reversed_edges: &[usize]) -> Vec<EdgeRoute> {
        let mut routes = Vec::with_capacity(self.original_edges.len());

        for (edge_idx, original_edge) in self.original_edges.iter().enumerate() {
            let is_reversed = reversed_edges.contains(&edge_idx);

            // Collect all waypoints for this edge (through dummy nodes)
            let mut waypoints = Vec::new();

            // Find the path through the graph for this edge
            let source_idx = self.id_to_index.get(&original_edge.source);
            let target_idx = self.id_to_index.get(&original_edge.target);

            if let (Some(&src), Some(&tgt)) = (source_idx, target_idx) {
                // Start point
                let src_node = &self.nodes[src];
                waypoints.push(Point::new(src_node.x, src_node.y));

                // Find dummy nodes for this edge
                let mut dummy_points: Vec<_> = self
                    .nodes
                    .iter()
                    .filter(|n| n.is_dummy && n.original_edge_index == Some(edge_idx))
                    .map(|n| (n.layer, Point::new(n.x, n.y)))
                    .collect();

                // Sort by layer
                dummy_points.sort_by_key(|(layer, _)| *layer);

                // Add dummy waypoints
                for (_, point) in dummy_points {
                    waypoints.push(point);
                }

                // End point
                let tgt_node = &self.nodes[tgt];
                waypoints.push(Point::new(tgt_node.x, tgt_node.y));

                // If edge was reversed, flip the waypoints
                if is_reversed {
                    waypoints.reverse();
                }
            }

            routes.push(EdgeRoute {
                source_id: original_edge.source.clone(),
                target_id: original_edge.target.clone(),
                waypoints,
            });
        }

        routes
    }

    /// Add a dummy node and return its index.
    pub fn add_dummy_node(&mut self, layer: usize, original_edge_index: usize) -> usize {
        let index = self.nodes.len();
        let dummy = LayeredNode::dummy(index, layer, original_edge_index);
        self.nodes.push(dummy);
        index
    }

    /// Add an edge between two node indices.
    pub fn add_edge(&mut self, source: usize, target: usize, original_index: usize) {
        self.edges.push(LayeredEdge::new(source, target, original_index));
    }

    /// Remove an edge by source and target indices.
    pub fn remove_edge(&mut self, source: usize, target: usize) {
        self.edges.retain(|e| e.source != source || e.target != target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_input() {
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
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

        let graph = LayeredGraph::from_input(&nodes, &edges);

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(graph.id_to_index.get("a"), Some(&0));
        assert_eq!(graph.id_to_index.get("b"), Some(&1));
        assert_eq!(graph.id_to_index.get("c"), Some(&2));
    }

    #[test]
    fn test_predecessors_successors() {
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
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

        let graph = LayeredGraph::from_input(&nodes, &edges);

        assert_eq!(graph.predecessors(0), Vec::<usize>::new()); // a has no predecessors
        assert_eq!(graph.predecessors(1), vec![0]); // b's predecessor is a
        assert_eq!(graph.predecessors(2), vec![1]); // c's predecessor is b

        assert_eq!(graph.successors(0), vec![1]); // a's successor is b
        assert_eq!(graph.successors(1), vec![2]); // b's successor is c
        assert_eq!(graph.successors(2), Vec::<usize>::new()); // c has no successors
    }

    #[test]
    fn test_add_dummy_node() {
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![LayoutEdge {
            source: "a".to_string(),
            target: "b".to_string(),
        }];

        let mut graph = LayeredGraph::from_input(&nodes, &edges);
        assert_eq!(graph.node_count(), 2);

        let dummy_idx = graph.add_dummy_node(1, 0);
        assert_eq!(dummy_idx, 2);
        assert_eq!(graph.node_count(), 3);
        assert!(graph.nodes[dummy_idx].is_dummy);
        assert_eq!(graph.nodes[dummy_idx].original_edge_index, Some(0));
    }
}
