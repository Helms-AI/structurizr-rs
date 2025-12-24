//! Abstract Syntax Tree (AST) for the Structurizr DSL.
//!
//! This module defines the intermediate representation produced by the parser
//! before it is converted into a `Workspace`.

use std::collections::HashMap;

/// A span in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// A node in the AST with span information.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

/// Root of the AST - a workspace definition.
#[derive(Debug, Clone)]
pub struct WorkspaceNode {
    pub name: Option<String>,
    pub description: Option<String>,
    pub extends: Option<String>,
    pub model: Option<ModelNode>,
    pub views: Option<ViewsNode>,
    pub properties: HashMap<String, String>,
    pub directives: Vec<Directive>,
}

/// A directive like !identifiers, !impliedRelationships, !const, etc.
#[derive(Debug, Clone)]
pub enum Directive {
    Identifiers(IdentifierScope),
    ImpliedRelationships(bool),
    Const { name: String, value: String },
    Include(String),
    Docs(String),
    Adrs(String),
}

/// Identifier scoping mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IdentifierScope {
    #[default]
    Flat,
    Hierarchical,
}

/// The model block containing all elements.
#[derive(Debug, Clone, Default)]
pub struct ModelNode {
    pub elements: Vec<ElementNode>,
    pub relationships: Vec<RelationshipNode>,
    pub groups: Vec<GroupNode>,
}

/// An element definition (person, softwareSystem, container, component).
#[derive(Debug, Clone)]
pub struct ElementNode {
    pub identifier: Option<String>,
    pub kind: ElementKind,
    pub name: String,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
    pub children: Vec<ElementNode>,
    pub relationships: Vec<RelationshipNode>,
}

/// Type of element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Person,
    SoftwareSystem,
    Container,
    Component,
    DeploymentNode,
    InfrastructureNode,
    ContainerInstance,
    SoftwareSystemInstance,
}

/// A relationship definition.
#[derive(Debug, Clone)]
pub struct RelationshipNode {
    pub source: String,
    pub destination: String,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// A group of elements.
#[derive(Debug, Clone)]
pub struct GroupNode {
    pub name: String,
    pub elements: Vec<ElementNode>,
    pub groups: Vec<GroupNode>,
}

/// The views block containing all view definitions.
#[derive(Debug, Clone, Default)]
pub struct ViewsNode {
    pub system_landscape: Vec<SystemLandscapeViewNode>,
    pub system_context: Vec<SystemContextViewNode>,
    pub container: Vec<ContainerViewNode>,
    pub component: Vec<ComponentViewNode>,
    pub dynamic: Vec<DynamicViewNode>,
    pub deployment: Vec<DeploymentViewNode>,
    pub filtered: Vec<FilteredViewNode>,
    pub custom: Vec<CustomViewNode>,
    pub styles: Option<StylesNode>,
    pub themes: Vec<String>,
}

/// Base view properties shared by all view types.
#[derive(Debug, Clone, Default)]
pub struct ViewPropertiesNode {
    pub key: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub include: Vec<IncludeExclude>,
    pub exclude: Vec<IncludeExclude>,
    pub auto_layout: Option<AutoLayoutNode>,
    pub properties: HashMap<String, String>,
    /// Background color for the view (e.g., "#1a1a1a" for dark mode).
    pub background: Option<String>,
}

/// Include or exclude directive in a view.
#[derive(Debug, Clone)]
pub enum IncludeExclude {
    All,
    Identifier(String),
    Expression(String),
}

/// Auto-layout configuration.
#[derive(Debug, Clone)]
pub struct AutoLayoutNode {
    pub direction: AutoLayoutDirection,
    pub rank_separation: Option<u32>,
    pub node_separation: Option<u32>,
}

/// Auto-layout direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoLayoutDirection {
    #[default]
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

/// System landscape view.
#[derive(Debug, Clone)]
pub struct SystemLandscapeViewNode {
    pub properties: ViewPropertiesNode,
}

/// System context view.
#[derive(Debug, Clone)]
pub struct SystemContextViewNode {
    pub software_system: String,
    pub properties: ViewPropertiesNode,
}

/// Container view.
#[derive(Debug, Clone)]
pub struct ContainerViewNode {
    pub software_system: String,
    pub properties: ViewPropertiesNode,
}

/// Component view.
#[derive(Debug, Clone)]
pub struct ComponentViewNode {
    pub container: String,
    pub properties: ViewPropertiesNode,
}

/// Dynamic view.
#[derive(Debug, Clone)]
pub struct DynamicViewNode {
    pub scope: Option<String>,
    pub properties: ViewPropertiesNode,
    pub steps: Vec<DynamicStepNode>,
}

/// A step in a dynamic view.
#[derive(Debug, Clone)]
pub struct DynamicStepNode {
    pub source: String,
    pub destination: String,
    pub description: Option<String>,
}

/// Deployment view.
#[derive(Debug, Clone)]
pub struct DeploymentViewNode {
    pub scope: Option<String>,
    pub environment: String,
    pub properties: ViewPropertiesNode,
}

/// Filtered view.
#[derive(Debug, Clone)]
pub struct FilteredViewNode {
    pub base_view: String,
    pub mode: FilterMode,
    pub tags: Vec<String>,
    pub properties: ViewPropertiesNode,
}

/// Filter mode for filtered views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    #[default]
    Include,
    Exclude,
}

/// Custom view.
#[derive(Debug, Clone)]
pub struct CustomViewNode {
    pub properties: ViewPropertiesNode,
}

/// Styles block.
#[derive(Debug, Clone, Default)]
pub struct StylesNode {
    pub elements: Vec<ElementStyleNode>,
    pub relationships: Vec<RelationshipStyleNode>,
}

/// Element style definition.
#[derive(Debug, Clone)]
pub struct ElementStyleNode {
    pub tag: String,
    pub shape: Option<String>,
    pub icon: Option<String>,
    pub icon_position: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub background: Option<String>,
    pub color: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: Option<u32>,
    pub font_size: Option<u32>,
    pub border: Option<String>,
    pub opacity: Option<u32>,
    pub metadata: Option<bool>,
    pub description: Option<bool>,
}

/// Relationship style definition.
#[derive(Debug, Clone)]
pub struct RelationshipStyleNode {
    pub tag: String,
    pub thickness: Option<u32>,
    pub color: Option<String>,
    pub style: Option<String>,
    pub routing: Option<String>,
    pub font_size: Option<u32>,
    pub width: Option<u32>,
    pub position: Option<u32>,
    pub opacity: Option<u32>,
}
