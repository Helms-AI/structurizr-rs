//! Prelude module for convenient imports.
//!
//! This module re-exports commonly used types for convenient importing:
//! ```rust
//! use structurizr_core::prelude::*;
//! ```

pub use crate::error::Error;
pub use crate::model::{
    Component, Container, ContainerInstance, CustomElement, DeploymentNode, ElementId, ElementProperties,
    ElementRef, Group, HealthCheck, InfrastructureNode, InteractionStyle, Location, Model, Person,
    Relationship, SoftwareSystem, SoftwareSystemInstance,
};
pub use crate::style::{Border, ElementStyle, IconPosition, LineStyle, RelationshipStyle, Routing, Shape, Styles};
pub use crate::view::{
    AutoLayout, AutoLayoutDirection, ComponentView, ContainerView, CustomView, DeploymentView,
    DynamicView, DynamicViewStep, ElementPosition, ElementView, FilterMode, FilteredView,
    ImageView, RelationshipView, SystemContextView, SystemLandscapeView, Vertex, ViewProperties,
    Views,
};
pub use crate::workspace::{
    UserRole, Workspace, WorkspaceConfiguration, WorkspaceScope, WorkspaceUser, WorkspaceVisibility,
};
