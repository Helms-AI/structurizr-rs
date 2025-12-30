//! Explore diagram module for interactive force-directed graph visualization.
//!
//! This module provides functionality for visualizing the entire workspace as an
//! interactive network diagram, allowing users to explore relationships between
//! all elements (people, systems, containers, components) in a dynamic graph view.

mod data;
mod renderer;

pub use renderer::generate_explore_page_html;

// Re-export data types if needed externally
pub use data::{GraphNode, GraphLink, extract_graph_data};