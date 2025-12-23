//! # structurizr-render
//!
//! SVG diagram rendering for Structurizr workspaces.
//!
//! This crate provides rendering capabilities to generate SVG diagrams
//! from Structurizr views.
//!
//! ## Features
//!
//! - **Style Resolution**: Automatically resolves styles from workspace definitions
//!   with proper tag matching and cascading.
//! - **Shape Rendering**: Supports all C4/Structurizr shapes including Person,
//!   Cylinder (database), Hexagon, WebBrowser, and more.
//! - **Layout Algorithms**: Grid-based auto-layout with configurable directions,
//!   plus advanced Sugiyama-style layout with crossing minimization.
//! - **Edge Routing**: Direct (straight line) and orthogonal (right-angle) routing.

pub mod error;
pub mod layout;
pub mod routing;
pub mod shapes;
pub mod style_resolver;
pub mod sugiyama;
pub mod svg;

pub use error::{Error, Result};
pub use layout::{GridLayout, LayoutEdge, LayoutNode, Point, Position, Size};
pub use routing::{EdgePath, Port, PortSide, RoutedEdge};
pub use shapes::{render_shape, Bounds};
pub use style_resolver::{ElementKind, ResolvedElementStyle, ResolvedRelationshipStyle, StyleResolver};
pub use svg::SvgRenderer;
