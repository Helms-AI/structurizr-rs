//! Structurizr Core - C4 model types and workspace structure.
//!
//! This crate provides the core data structures for representing software
//! architecture using the C4 model.

pub mod error;
pub mod model;
pub mod prelude;
pub mod style;
pub mod theme;
pub mod view;
pub mod workspace;

#[cfg(test)]
mod perspective_tests;

pub use error::Error;
pub use model::{
    Component, Container, ContainerInstance, DeploymentNode, ElementId, ElementProperties,
    ElementRef, Group, InfrastructureNode, Model, Person, Relationship, SoftwareSystem,
    SoftwareSystemInstance,
};
pub use style::{Border, ElementStyle, RelationshipStyle, Shape, Styles};
pub use theme::{apply_themes, clear_theme_cache, fetch_theme, Theme};
pub use view::{
    AutoLayout, ComponentView, ContainerView, DeploymentView, DynamicView, FilteredView,
    SystemContextView, SystemLandscapeView, ViewProperties, Views,
};
pub use workspace::{Perspective, Workspace};
