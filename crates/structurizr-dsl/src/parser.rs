//! Parser for the Structurizr DSL.
//!
//! Parses tokens into an AST and then converts to a Workspace.

use std::collections::HashMap;

use structurizr_core::prelude::*;

use crate::ast::*;
use crate::error::{Error, ParseError, Result};
use crate::lexer::{Lexer, Token, TokenKind};

/// Parse a DSL string into a Workspace.
pub fn parse(input: &str) -> Result<Workspace> {
    parse_with_base_path(input, None)
}

/// Parse a DSL string into a Workspace with a base path for resolving includes.
pub fn parse_with_base_path(input: &str, base_path: Option<&std::path::Path>) -> Result<Workspace> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut ast = parser.parse_workspace()?;

    // Process !include directives
    if let Some(base) = base_path {
        ast = process_includes(ast, base)?;
    }

    let workspace = build_workspace(ast)?;
    Ok(workspace)
}

/// Parse a DSL string into an AST (for testing/debugging).
pub fn parse_to_ast(input: &str) -> Result<WorkspaceNode> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_workspace()
}

/// Parser state.
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn current_kind(&self) -> Option<&TokenKind> {
        self.current().map(|t| &t.kind)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }

    fn skip_newlines_and_comments(&mut self) {
        while let Some(kind) = self.current_kind() {
            match kind {
                TokenKind::Newline | TokenKind::Comment => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<&Token> {
        self.skip_newlines_and_comments();
        match self.current() {
            Some(token) if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) => {
                self.position += 1;
                Ok(&self.tokens[self.position - 1])
            }
            Some(token) => Err(Error::Parse(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, token.kind),
                location: Some(token.location),
            })),
            None => Err(Error::Parse(ParseError {
                message: format!("Expected {:?}, found end of input", expected),
                location: None,
            })),
        }
    }

    fn expect_string(&mut self) -> Result<String> {
        self.skip_newlines_and_comments();
        match self.current_kind().cloned() {
            Some(TokenKind::String(s)) => {
                self.advance();
                Ok(s)
            }
            Some(kind) => {
                let loc = self.current().map(|t| t.location);
                Err(Error::Parse(ParseError {
                    message: format!("Expected string, found {:?}", kind),
                    location: loc,
                }))
            }
            None => Err(Error::Parse(ParseError {
                message: "Expected string, found end of input".to_string(),
                location: None,
            })),
        }
    }

    fn expect_identifier(&mut self) -> Result<String> {
        self.skip_newlines_and_comments();
        match self.current_kind().cloned() {
            Some(TokenKind::Identifier(s)) => {
                self.advance();
                Ok(s)
            }
            Some(kind) => {
                let loc = self.current().map(|t| t.location);
                Err(Error::Parse(ParseError {
                    message: format!("Expected identifier, found {:?}", kind),
                    location: loc,
                }))
            }
            None => Err(Error::Parse(ParseError {
                message: "Expected identifier, found end of input".to_string(),
                location: None,
            })),
        }
    }

    /// Expects an identifier or a keyword that can be used as a shape name (e.g., Person).
    fn expect_identifier_or_shape_keyword(&mut self) -> Result<String> {
        self.skip_newlines_and_comments();
        match self.current_kind().cloned() {
            Some(TokenKind::Identifier(s)) => {
                self.advance();
                Ok(s)
            }
            // Shape keywords that are also DSL keywords
            Some(TokenKind::Person) => {
                self.advance();
                Ok("Person".to_string())
            }
            Some(TokenKind::Component) => {
                self.advance();
                Ok("Component".to_string())
            }
            Some(kind) => {
                let loc = self.current().map(|t| t.location);
                Err(Error::Parse(ParseError {
                    message: format!("Expected shape name, found {:?}", kind),
                    location: loc,
                }))
            }
            None => Err(Error::Parse(ParseError {
                message: "Expected shape name, found end of input".to_string(),
                location: None,
            })),
        }
    }

    fn maybe_string(&mut self) -> Option<String> {
        self.skip_newlines_and_comments();
        match self.current_kind().cloned() {
            Some(TokenKind::String(s)) => {
                self.advance();
                Some(s)
            }
            _ => None,
        }
    }

    /// Parse an optional string or number (used for instances which can be "1", "2-10", or just 5)
    fn maybe_string_or_number(&mut self) -> Option<String> {
        self.skip_newlines_and_comments();
        match self.current_kind().cloned() {
            Some(TokenKind::String(s)) => {
                self.advance();
                Some(s)
            }
            Some(TokenKind::Number(n)) => {
                self.advance();
                Some(n.to_string())
            }
            _ => None,
        }
    }

    fn check(&self, expected: &TokenKind) -> bool {
        match self.current_kind() {
            Some(kind) => std::mem::discriminant(kind) == std::mem::discriminant(expected),
            None => false,
        }
    }

    pub fn parse_workspace(&mut self) -> Result<WorkspaceNode> {
        self.skip_newlines_and_comments();

        // Check for optional "workspace" keyword
        let has_workspace_keyword = matches!(self.current_kind(), Some(TokenKind::Workspace));
        if has_workspace_keyword {
            self.advance();
        }

        // Optional name and description
        let name = self.maybe_string();
        let description = self.maybe_string();

        // Check for "extends"
        let extends = if matches!(self.current_kind(), Some(TokenKind::Extends)) {
            self.advance();
            Some(self.expect_string()?)
        } else {
            None
        };

        self.skip_newlines_and_comments();

        // Opening brace (optional for top-level)
        let has_brace = self.check(&TokenKind::OpenBrace);
        if has_brace {
            self.advance();
        }

        let mut workspace = WorkspaceNode {
            name,
            description,
            extends,
            model: None,
            views: None,
            properties: HashMap::new(),
            directives: Vec::new(),
        };

        // Parse workspace contents
        loop {
            self.skip_newlines_and_comments();

            match self.current_kind() {
                Some(TokenKind::CloseBrace) if has_brace => {
                    self.advance();
                    break;
                }
                Some(TokenKind::Eof) => break,
                Some(TokenKind::Model) => {
                    self.advance();
                    workspace.model = Some(self.parse_model()?);
                }
                Some(TokenKind::Views) => {
                    self.advance();
                    workspace.views = Some(self.parse_views()?);
                }
                Some(TokenKind::Properties) => {
                    self.advance();
                    workspace.properties = self.parse_properties_block()?;
                }
                Some(TokenKind::DirectiveIdentifiers) => {
                    self.advance();
                    let scope = self.expect_identifier()?;
                    let scope = if scope.to_lowercase() == "hierarchical" {
                        IdentifierScope::Hierarchical
                    } else {
                        IdentifierScope::Flat
                    };
                    workspace.directives.push(Directive::Identifiers(scope));
                }
                Some(TokenKind::DirectiveImpliedRelationships) => {
                    self.advance();
                    let enabled = self.expect_identifier()?;
                    let enabled = enabled.to_lowercase() != "false" && enabled.to_lowercase() != "off";
                    workspace.directives.push(Directive::ImpliedRelationships(enabled));
                }
                Some(TokenKind::DirectiveConst) => {
                    self.advance();
                    let name = self.expect_identifier()?;
                    let value = self.expect_string()?;
                    workspace.directives.push(Directive::Const { name, value });
                }
                Some(TokenKind::DirectiveInclude) => {
                    self.advance();
                    let path = self.expect_string()?;
                    workspace.directives.push(Directive::Include(path));
                }
                Some(TokenKind::DirectiveDocs) => {
                    self.advance();
                    let path = self.expect_string()?;
                    workspace.directives.push(Directive::Docs(path));
                }
                Some(TokenKind::DirectiveAdrs) => {
                    self.advance();
                    let path = self.expect_string()?;
                    workspace.directives.push(Directive::Adrs(path));
                }
                Some(kind) => {
                    return Err(Error::Parse(ParseError {
                        message: format!("Unexpected token in workspace: {:?}", kind),
                        location: self.current().map(|t| t.location),
                    }));
                }
                None => break,
            }
        }

        Ok(workspace)
    }

    fn parse_model(&mut self) -> Result<ModelNode> {
        self.skip_newlines_and_comments();
        self.expect(TokenKind::OpenBrace)?;

        let mut model = ModelNode::default();

        loop {
            self.skip_newlines_and_comments();

            match self.current_kind().cloned() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::Person) => {
                    self.advance();
                    let element = self.parse_element(ElementKind::Person, None)?;
                    model.elements.push(element);
                }
                Some(TokenKind::SoftwareSystem) => {
                    self.advance();
                    let element = self.parse_element(ElementKind::SoftwareSystem, None)?;
                    model.elements.push(element);
                }
                Some(TokenKind::DeploymentNode) => {
                    self.advance();
                    let element = self.parse_element(ElementKind::DeploymentNode, None)?;
                    model.elements.push(element);
                }
                Some(TokenKind::DeploymentEnvironment) => {
                    self.advance();
                    let env = self.parse_deployment_environment()?;
                    model.deployment_environments.push(env);
                }
                Some(TokenKind::Group) => {
                    self.advance();
                    let group = self.parse_group()?;
                    model.groups.push(group);
                }
                Some(TokenKind::Identifier(id)) => {
                    // Could be: identifier = element, or identifier -> target
                    let id = id.clone();
                    self.advance();

                    self.skip_newlines_and_comments();

                    if self.check(&TokenKind::Equals) {
                        self.advance();
                        self.skip_newlines_and_comments();

                        match self.current_kind() {
                            Some(TokenKind::Person) => {
                                self.advance();
                                let element = self.parse_element(ElementKind::Person, Some(id))?;
                                model.elements.push(element);
                            }
                            Some(TokenKind::SoftwareSystem) => {
                                self.advance();
                                let element = self.parse_element(ElementKind::SoftwareSystem, Some(id))?;
                                model.elements.push(element);
                            }
                            Some(TokenKind::DeploymentNode) => {
                                self.advance();
                                let element = self.parse_element(ElementKind::DeploymentNode, Some(id))?;
                                model.elements.push(element);
                            }
                            _ => {
                                return Err(Error::Parse(ParseError {
                                    message: "Expected element type after '='".to_string(),
                                    location: self.current().map(|t| t.location),
                                }));
                            }
                        }
                    } else if self.check(&TokenKind::Arrow) {
                        self.advance();
                        let rel = self.parse_relationship(id)?;
                        model.relationships.push(rel);
                    } else {
                        return Err(Error::Parse(ParseError {
                            message: "Expected '=' or '->' after identifier".to_string(),
                            location: self.current().map(|t| t.location),
                        }));
                    }
                }
                Some(kind) => {
                    return Err(Error::Parse(ParseError {
                        message: format!("Unexpected token in model: {:?}", kind),
                        location: self.current().map(|t| t.location),
                    }));
                }
                None => {
                    return Err(Error::Parse(ParseError {
                        message: "Unexpected end of input in model".to_string(),
                        location: None,
                    }));
                }
            }
        }

        Ok(model)
    }

    fn parse_element(&mut self, kind: ElementKind, identifier: Option<String>) -> Result<ElementNode> {
        self.skip_newlines_and_comments();

        // ContainerInstance and SoftwareSystemInstance use an identifier reference, not a string
        let name = if matches!(kind, ElementKind::ContainerInstance | ElementKind::SoftwareSystemInstance) {
            self.expect_identifier()?
        } else {
            self.expect_string()?
        };
        let description = self.maybe_string();

        // Technology is the 3rd string for Container/Component
        // For DeploymentNode, it's also the 3rd string (followed by tags and instances)
        let technology = if matches!(kind, ElementKind::Container | ElementKind::Component | ElementKind::DeploymentNode) {
            self.maybe_string()
        } else {
            None
        };

        // For DeploymentNode: parse inline tags (4th) and instances (5th)
        // Syntax: deploymentNode <name> [description] [technology] [tags] [instances] { ... }
        let (inline_tags, instances) = if kind == ElementKind::DeploymentNode {
            let tags_str = self.maybe_string();
            let instances_str = self.maybe_string_or_number();
            (tags_str, instances_str)
        } else {
            (None, None)
        };

        let mut element = ElementNode {
            identifier,
            kind,
            name,
            description,
            technology,
            tags: inline_tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            properties: HashMap::new(),
            children: Vec::new(),
            relationships: Vec::new(),
            instances,
        };

        // Check for optional block
        self.skip_newlines_and_comments();
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            self.parse_element_body(&mut element)?;
        }

        Ok(element)
    }

    fn parse_element_body(&mut self, element: &mut ElementNode) -> Result<()> {
        loop {
            self.skip_newlines_and_comments();

            match self.current_kind().cloned() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::Tags) => {
                    self.advance();
                    let tags = self.expect_string()?;
                    element.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
                }
                Some(TokenKind::Technology) => {
                    self.advance();
                    element.technology = Some(self.expect_string()?);
                }
                Some(TokenKind::Description) => {
                    self.advance();
                    element.description = Some(self.expect_string()?);
                }
                Some(TokenKind::Url) => {
                    self.advance();
                    let url = self.expect_string()?;
                    element.properties.insert("url".to_string(), url);
                }
                Some(TokenKind::Instances) if element.kind == ElementKind::DeploymentNode => {
                    self.advance();
                    // instances can be a number or string (e.g., "2-10")
                    let instances = self.maybe_string_or_number().ok_or_else(|| {
                        Error::Parse(ParseError {
                            message: "Expected number or string after 'instances'".to_string(),
                            location: self.current().map(|t| t.location),
                        })
                    })?;
                    element.instances = Some(instances);
                }
                Some(TokenKind::Properties) => {
                    self.advance();
                    element.properties.extend(self.parse_properties_block()?);
                }
                // Child elements
                Some(TokenKind::Container) if element.kind == ElementKind::SoftwareSystem => {
                    self.advance();
                    let child = self.parse_element(ElementKind::Container, None)?;
                    element.children.push(child);
                }
                Some(TokenKind::Component) if element.kind == ElementKind::Container => {
                    self.advance();
                    let child = self.parse_element(ElementKind::Component, None)?;
                    element.children.push(child);
                }
                Some(TokenKind::DeploymentNode) if element.kind == ElementKind::DeploymentNode => {
                    self.advance();
                    let child = self.parse_element(ElementKind::DeploymentNode, None)?;
                    element.children.push(child);
                }
                Some(TokenKind::InfrastructureNode) if element.kind == ElementKind::DeploymentNode => {
                    self.advance();
                    let child = self.parse_element(ElementKind::InfrastructureNode, None)?;
                    element.children.push(child);
                }
                Some(TokenKind::ContainerInstance) if element.kind == ElementKind::DeploymentNode => {
                    self.advance();
                    let child = self.parse_element(ElementKind::ContainerInstance, None)?;
                    element.children.push(child);
                }
                // Identifier could be child element with identifier or relationship
                Some(TokenKind::Identifier(id)) => {
                    let id = id.clone();
                    self.advance();
                    self.skip_newlines_and_comments();

                    if self.check(&TokenKind::Equals) {
                        self.advance();
                        self.skip_newlines_and_comments();

                        match (self.current_kind(), element.kind) {
                            (Some(TokenKind::Container), ElementKind::SoftwareSystem) => {
                                self.advance();
                                let child = self.parse_element(ElementKind::Container, Some(id))?;
                                element.children.push(child);
                            }
                            (Some(TokenKind::Component), ElementKind::Container) => {
                                self.advance();
                                let child = self.parse_element(ElementKind::Component, Some(id))?;
                                element.children.push(child);
                            }
                            (Some(TokenKind::DeploymentNode), ElementKind::DeploymentNode) => {
                                self.advance();
                                let child = self.parse_element(ElementKind::DeploymentNode, Some(id))?;
                                element.children.push(child);
                            }
                            (Some(TokenKind::ContainerInstance), ElementKind::DeploymentNode) => {
                                self.advance();
                                let child = self.parse_element(ElementKind::ContainerInstance, Some(id))?;
                                element.children.push(child);
                            }
                            (Some(TokenKind::SoftwareSystemInstance), ElementKind::DeploymentNode) => {
                                self.advance();
                                let child = self.parse_element(ElementKind::SoftwareSystemInstance, Some(id))?;
                                element.children.push(child);
                            }
                            (Some(TokenKind::InfrastructureNode), ElementKind::DeploymentNode) => {
                                self.advance();
                                let child = self.parse_element(ElementKind::InfrastructureNode, Some(id))?;
                                element.children.push(child);
                            }
                            _ => {
                                return Err(Error::Parse(ParseError {
                                    message: "Unexpected element type in body".to_string(),
                                    location: self.current().map(|t| t.location),
                                }));
                            }
                        }
                    } else if self.check(&TokenKind::Arrow) {
                        self.advance();
                        let rel = self.parse_relationship(id)?;
                        element.relationships.push(rel);
                    }
                }
                Some(kind) => {
                    return Err(Error::Parse(ParseError {
                        message: format!("Unexpected token in element body: {:?}", kind),
                        location: self.current().map(|t| t.location),
                    }));
                }
                None => {
                    return Err(Error::Parse(ParseError {
                        message: "Unexpected end of input in element body".to_string(),
                        location: None,
                    }));
                }
            }
        }

        Ok(())
    }

    fn parse_relationship(&mut self, source: String) -> Result<RelationshipNode> {
        self.skip_newlines_and_comments();

        let destination = self.expect_identifier()?;
        let description = self.maybe_string();
        let technology = self.maybe_string();

        let mut rel = RelationshipNode {
            source,
            destination,
            description,
            technology,
            tags: Vec::new(),
            properties: HashMap::new(),
            perspectives: HashMap::new(),
        };

        // Check for optional block
        self.skip_newlines_and_comments();
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            loop {
                self.skip_newlines_and_comments();

                match self.current_kind() {
                    Some(TokenKind::CloseBrace) => {
                        self.advance();
                        break;
                    }
                    Some(TokenKind::Tags) => {
                        self.advance();
                        let tags = self.expect_string()?;
                        rel.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    Some(TokenKind::Properties) => {
                        self.advance();
                        rel.properties.extend(self.parse_properties_block()?);
                    }
                    Some(TokenKind::Perspectives) => {
                        self.advance();
                        rel.perspectives.extend(self.parse_perspectives_block()?);
                    }
                    _ => break,
                }
            }
        }

        Ok(rel)
    }

    fn parse_group(&mut self) -> Result<GroupNode> {
        self.skip_newlines_and_comments();
        let name = self.expect_string()?;

        let mut group = GroupNode {
            name,
            elements: Vec::new(),
            groups: Vec::new(),
        };

        self.skip_newlines_and_comments();
        self.expect(TokenKind::OpenBrace)?;

        loop {
            self.skip_newlines_and_comments();

            match self.current_kind().cloned() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::Group) => {
                    self.advance();
                    let nested = self.parse_group()?;
                    group.groups.push(nested);
                }
                Some(TokenKind::Person) => {
                    self.advance();
                    let element = self.parse_element(ElementKind::Person, None)?;
                    group.elements.push(element);
                }
                Some(TokenKind::SoftwareSystem) => {
                    self.advance();
                    let element = self.parse_element(ElementKind::SoftwareSystem, None)?;
                    group.elements.push(element);
                }
                Some(TokenKind::Identifier(id)) => {
                    let id = id.clone();
                    self.advance();
                    self.skip_newlines_and_comments();

                    if self.check(&TokenKind::Equals) {
                        self.advance();
                        self.skip_newlines_and_comments();

                        match self.current_kind() {
                            Some(TokenKind::Person) => {
                                self.advance();
                                let element = self.parse_element(ElementKind::Person, Some(id))?;
                                group.elements.push(element);
                            }
                            Some(TokenKind::SoftwareSystem) => {
                                self.advance();
                                let element = self.parse_element(ElementKind::SoftwareSystem, Some(id))?;
                                group.elements.push(element);
                            }
                            _ => break,
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(group)
    }

    fn parse_deployment_environment(&mut self) -> Result<DeploymentEnvironmentNode> {
        self.skip_newlines_and_comments();
        let name = self.expect_string()?;

        self.skip_newlines_and_comments();
        self.expect(TokenKind::OpenBrace)?;

        let mut children = Vec::new();

        loop {
            self.skip_newlines_and_comments();

            match self.current_kind().cloned() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::DeploymentNode) => {
                    self.advance();
                    let element = self.parse_element(ElementKind::DeploymentNode, None)?;
                    children.push(element);
                }
                Some(TokenKind::DeploymentGroup) => {
                    // DeploymentGroup is parsed as identifier = deploymentGroup "name"
                    // For now, skip this case - it will be handled in identifier parsing
                    self.advance();
                    let _name = self.expect_string()?;
                    // TODO: Store deployment group references for later use
                }
                Some(TokenKind::Identifier(id)) => {
                    let id = id.clone();
                    self.advance();
                    self.skip_newlines_and_comments();

                    if self.check(&TokenKind::Equals) {
                        self.advance();
                        self.skip_newlines_and_comments();

                        match self.current_kind() {
                            Some(TokenKind::DeploymentNode) => {
                                self.advance();
                                let element = self.parse_element(ElementKind::DeploymentNode, Some(id))?;
                                children.push(element);
                            }
                            Some(TokenKind::DeploymentGroup) => {
                                self.advance();
                                let _name = self.expect_string()?;
                                // TODO: Store deployment group with identifier
                            }
                            _ => {
                                return Err(Error::Parse(ParseError {
                                    message: "Expected deploymentNode or deploymentGroup after '='".to_string(),
                                    location: self.current().map(|t| t.location),
                                }));
                            }
                        }
                    } else {
                        return Err(Error::Parse(ParseError {
                            message: "Expected '=' after identifier in deploymentEnvironment".to_string(),
                            location: self.current().map(|t| t.location),
                        }));
                    }
                }
                Some(kind) => {
                    return Err(Error::Parse(ParseError {
                        message: format!("Unexpected token in deploymentEnvironment: {:?}", kind),
                        location: self.current().map(|t| t.location),
                    }));
                }
                None => {
                    return Err(Error::Parse(ParseError {
                        message: "Unexpected end of input in deploymentEnvironment".to_string(),
                        location: None,
                    }));
                }
            }
        }

        Ok(DeploymentEnvironmentNode { name, children })
    }

    fn parse_views(&mut self) -> Result<ViewsNode> {
        self.skip_newlines_and_comments();
        self.expect(TokenKind::OpenBrace)?;

        let mut views = ViewsNode::default();

        loop {
            self.skip_newlines_and_comments();

            match self.current_kind() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::SystemLandscape) => {
                    self.advance();
                    let view = self.parse_system_landscape_view()?;
                    views.system_landscape.push(view);
                }
                Some(TokenKind::SystemContext) => {
                    self.advance();
                    let view = self.parse_system_context_view()?;
                    views.system_context.push(view);
                }
                Some(TokenKind::Container) => {
                    self.advance();
                    let view = self.parse_container_view()?;
                    views.container.push(view);
                }
                Some(TokenKind::Component) => {
                    self.advance();
                    let view = self.parse_component_view()?;
                    views.component.push(view);
                }
                Some(TokenKind::Dynamic) => {
                    self.advance();
                    let view = self.parse_dynamic_view()?;
                    views.dynamic.push(view);
                }
                Some(TokenKind::Deployment) => {
                    self.advance();
                    let view = self.parse_deployment_view()?;
                    views.deployment.push(view);
                }
                Some(TokenKind::Styles) => {
                    self.advance();
                    views.styles = Some(self.parse_styles()?);
                }
                Some(TokenKind::Theme) => {
                    self.advance();
                    let url = self.expect_string()?;
                    views.themes.push(url);
                }
                Some(TokenKind::Themes) => {
                    self.advance();
                    while let Some(url) = self.maybe_string() {
                        views.themes.push(url);
                    }
                }
                Some(kind) => {
                    return Err(Error::Parse(ParseError {
                        message: format!("Unexpected token in views: {:?}", kind),
                        location: self.current().map(|t| t.location),
                    }));
                }
                None => break,
            }
        }

        Ok(views)
    }

    fn parse_view_properties(&mut self) -> Result<ViewPropertiesNode> {
        let key = self.maybe_string();
        let title = self.maybe_string();
        let description = self.maybe_string();

        let mut props = ViewPropertiesNode {
            key,
            title,
            description,
            include: Vec::new(),
            exclude: Vec::new(),
            auto_layout: None,
            properties: HashMap::new(),
            background: None,
        };

        self.skip_newlines_and_comments();
        if self.check(&TokenKind::OpenBrace) {
            self.advance();

            loop {
                self.skip_newlines_and_comments();

                match self.current_kind().cloned() {
                    Some(TokenKind::CloseBrace) => {
                        self.advance();
                        break;
                    }
                    Some(TokenKind::Include) => {
                        self.advance();
                        self.skip_newlines_and_comments();
                        if self.check(&TokenKind::Star) {
                            self.advance();
                            props.include.push(IncludeExclude::All);
                        } else if let Some(TokenKind::String(s)) = self.current_kind().cloned() {
                            self.advance();
                            props.include.push(IncludeExclude::Expression(s));
                        } else if let Some(TokenKind::Identifier(id)) = self.current_kind().cloned() {
                            self.advance();
                            props.include.push(IncludeExclude::Identifier(id));
                        }
                    }
                    Some(TokenKind::Exclude) => {
                        self.advance();
                        self.skip_newlines_and_comments();
                        if let Some(TokenKind::String(s)) = self.current_kind().cloned() {
                            self.advance();
                            props.exclude.push(IncludeExclude::Expression(s));
                        } else if let Some(TokenKind::Identifier(id)) = self.current_kind().cloned() {
                            self.advance();
                            props.exclude.push(IncludeExclude::Identifier(id));
                        }
                    }
                    Some(TokenKind::AutoLayout) => {
                        self.advance();
                        props.auto_layout = Some(self.parse_auto_layout()?);
                    }
                    Some(TokenKind::Properties) => {
                        self.advance();
                        props.properties.extend(self.parse_properties_block()?);
                    }
                    Some(TokenKind::Identifier(ref id)) => {
                        match id.to_lowercase().as_str() {
                            "background" => {
                                self.advance();
                                props.background = Some(self.expect_string()?);
                            }
                            _ => break,
                        }
                    }
                    _ => break,
                }
            }
        }

        Ok(props)
    }

    fn parse_auto_layout(&mut self) -> Result<AutoLayoutNode> {
        self.skip_newlines_and_comments();

        let direction = if let Some(TokenKind::Identifier(id)) = self.current_kind().cloned() {
            // Only consume the identifier if it's a known direction
            match id.to_lowercase().as_str() {
                "tb" => {
                    self.advance();
                    crate::ast::AutoLayoutDirection::TopBottom
                }
                "bt" => {
                    self.advance();
                    crate::ast::AutoLayoutDirection::BottomTop
                }
                "lr" => {
                    self.advance();
                    crate::ast::AutoLayoutDirection::LeftRight
                }
                "rl" => {
                    self.advance();
                    crate::ast::AutoLayoutDirection::RightLeft
                }
                _ => crate::ast::AutoLayoutDirection::TopBottom,
            }
        } else {
            crate::ast::AutoLayoutDirection::TopBottom
        };

        let rank_separation = if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
            self.advance();
            Some(n as u32)
        } else {
            None
        };

        let node_separation = if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
            self.advance();
            Some(n as u32)
        } else {
            None
        };

        Ok(AutoLayoutNode {
            direction,
            rank_separation,
            node_separation,
        })
    }

    fn parse_system_landscape_view(&mut self) -> Result<SystemLandscapeViewNode> {
        let properties = self.parse_view_properties()?;
        Ok(SystemLandscapeViewNode { properties })
    }

    fn parse_system_context_view(&mut self) -> Result<SystemContextViewNode> {
        self.skip_newlines_and_comments();
        let software_system = self.expect_identifier()?;
        let properties = self.parse_view_properties()?;
        Ok(SystemContextViewNode {
            software_system,
            properties,
        })
    }

    fn parse_container_view(&mut self) -> Result<ContainerViewNode> {
        self.skip_newlines_and_comments();
        let software_system = self.expect_identifier()?;
        let properties = self.parse_view_properties()?;
        Ok(ContainerViewNode {
            software_system,
            properties,
        })
    }

    fn parse_component_view(&mut self) -> Result<ComponentViewNode> {
        self.skip_newlines_and_comments();
        let container = self.expect_identifier()?;
        let properties = self.parse_view_properties()?;
        Ok(ComponentViewNode {
            container,
            properties,
        })
    }

    fn parse_dynamic_view(&mut self) -> Result<DynamicViewNode> {
        self.skip_newlines_and_comments();

        // Parse optional scope (element identifier or *)
        let scope = if let Some(TokenKind::Identifier(_)) = self.current_kind() {
            Some(self.expect_identifier()?)
        } else if self.check(&TokenKind::Star) {
            self.advance();
            None
        } else {
            None
        };

        // Parse optional key, title, description strings
        let key = self.maybe_string();
        let title = self.maybe_string();
        let description = self.maybe_string();

        let mut props = ViewPropertiesNode {
            key,
            title,
            description,
            include: Vec::new(),
            exclude: Vec::new(),
            auto_layout: None,
            properties: HashMap::new(),
            background: None,
        };

        let mut content = Vec::new();

        // Parse the block if present
        self.skip_newlines_and_comments();
        if self.check(&TokenKind::OpenBrace) {
            self.advance();
            self.parse_dynamic_view_body(&mut props, &mut content)?;
        }

        Ok(DynamicViewNode {
            scope,
            properties: props,
            content,
        })
    }

    /// Parse the body of a dynamic view, handling view properties, steps, and parallel sequences.
    fn parse_dynamic_view_body(
        &mut self,
        props: &mut ViewPropertiesNode,
        content: &mut Vec<DynamicContent>,
    ) -> Result<()> {
        loop {
            self.skip_newlines_and_comments();

            match self.current_kind().cloned() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::OpenBrace) => {
                    // Start of a parallel sequence block
                    self.advance();
                    let parallel = self.parse_parallel_sequence()?;
                    content.push(DynamicContent::ParallelSequence(parallel));
                }
                Some(TokenKind::Include) => {
                    self.advance();
                    self.skip_newlines_and_comments();
                    if self.check(&TokenKind::Star) {
                        self.advance();
                        props.include.push(IncludeExclude::All);
                    } else if let Some(TokenKind::String(s)) = self.current_kind().cloned() {
                        self.advance();
                        props.include.push(IncludeExclude::Expression(s));
                    } else if let Some(TokenKind::Identifier(id)) = self.current_kind().cloned() {
                        self.advance();
                        props.include.push(IncludeExclude::Identifier(id));
                    }
                }
                Some(TokenKind::Exclude) => {
                    self.advance();
                    self.skip_newlines_and_comments();
                    if let Some(TokenKind::String(s)) = self.current_kind().cloned() {
                        self.advance();
                        props.exclude.push(IncludeExclude::Expression(s));
                    } else if let Some(TokenKind::Identifier(id)) = self.current_kind().cloned() {
                        self.advance();
                        props.exclude.push(IncludeExclude::Identifier(id));
                    }
                }
                Some(TokenKind::AutoLayout) => {
                    self.advance();
                    props.auto_layout = Some(self.parse_auto_layout()?);
                }
                Some(TokenKind::Properties) => {
                    self.advance();
                    props.properties.extend(self.parse_properties_block()?);
                }
                Some(TokenKind::Identifier(source)) => {
                    self.advance();
                    self.skip_newlines_and_comments();

                    if self.check(&TokenKind::Arrow) {
                        // This is a dynamic step: source -> destination "description"
                        self.advance();
                        let step = self.parse_dynamic_step(source)?;
                        content.push(DynamicContent::Step(step));
                    } else {
                        // Handle other identifier-based properties like "background"
                        match source.to_lowercase().as_str() {
                            "background" => {
                                props.background = Some(self.expect_string()?);
                            }
                            _ => {
                                // Unknown identifier, break out
                                break;
                            }
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(())
    }

    /// Parse a parallel sequence block: { content... }
    /// Called after the opening brace has been consumed.
    fn parse_parallel_sequence(&mut self) -> Result<ParallelSequenceNode> {
        let mut content = Vec::new();

        loop {
            self.skip_newlines_and_comments();

            match self.current_kind().cloned() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::OpenBrace) => {
                    // Nested parallel sequence
                    self.advance();
                    let nested = self.parse_parallel_sequence()?;
                    content.push(DynamicContent::ParallelSequence(nested));
                }
                Some(TokenKind::Identifier(source)) => {
                    self.advance();
                    self.skip_newlines_and_comments();

                    if self.check(&TokenKind::Arrow) {
                        self.advance();
                        let step = self.parse_dynamic_step(source)?;
                        content.push(DynamicContent::Step(step));
                    } else {
                        // Unknown identifier in parallel block
                        break;
                    }
                }
                _ => break,
            }
        }

        Ok(ParallelSequenceNode { content })
    }

    /// Parse a single dynamic step: destination "description"
    /// (source was already consumed and is passed in)
    fn parse_dynamic_step(&mut self, source: String) -> Result<DynamicStepNode> {
        self.skip_newlines_and_comments();
        let destination = self.expect_identifier()?;
        let description = self.maybe_string();

        Ok(DynamicStepNode {
            source,
            destination,
            description,
        })
    }

    fn parse_deployment_view(&mut self) -> Result<DeploymentViewNode> {
        self.skip_newlines_and_comments();
        let scope = if let Some(TokenKind::Identifier(_)) = self.current_kind() {
            Some(self.expect_identifier()?)
        } else if self.check(&TokenKind::Star) {
            self.advance();
            None
        } else {
            None
        };

        let environment = self.expect_string()?;
        let properties = self.parse_view_properties()?;

        Ok(DeploymentViewNode {
            scope,
            environment,
            properties,
        })
    }

    fn parse_styles(&mut self) -> Result<StylesNode> {
        self.skip_newlines_and_comments();
        self.expect(TokenKind::OpenBrace)?;

        let mut styles = StylesNode::default();

        loop {
            self.skip_newlines_and_comments();

            match self.current_kind() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::Element) => {
                    self.advance();
                    let style = self.parse_element_style()?;
                    styles.elements.push(style);
                }
                Some(TokenKind::Relationship) => {
                    self.advance();
                    let style = self.parse_relationship_style()?;
                    styles.relationships.push(style);
                }
                _ => break,
            }
        }

        Ok(styles)
    }

    fn parse_element_style(&mut self) -> Result<ElementStyleNode> {
        self.skip_newlines_and_comments();
        let tag = self.expect_string()?;

        let mut style = ElementStyleNode {
            tag,
            shape: None,
            icon: None,
            icon_position: None,
            width: None,
            height: None,
            background: None,
            color: None,
            stroke: None,
            stroke_width: None,
            font_size: None,
            border: None,
            opacity: None,
            metadata: None,
            description: None,
        };

        self.skip_newlines_and_comments();
        if self.check(&TokenKind::OpenBrace) {
            self.advance();

            loop {
                self.skip_newlines_and_comments();

                match self.current_kind().cloned() {
                    Some(TokenKind::CloseBrace) => {
                        self.advance();
                        break;
                    }
                    Some(TokenKind::Identifier(prop)) => {
                        self.advance();
                        match prop.to_lowercase().as_str() {
                            "shape" => {
                                // Shape can be Person (which is a keyword) or other identifiers
                                style.shape = Some(self.expect_identifier_or_shape_keyword()?);
                            }
                            "icon" => style.icon = Some(self.expect_string()?),
                            "iconposition" => {
                                style.icon_position = Some(self.expect_identifier()?);
                            }
                            "width" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.width = Some(n as u32);
                                }
                            }
                            "height" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.height = Some(n as u32);
                                }
                            }
                            "background" => style.background = Some(self.expect_string()?),
                            "color" | "colour" => style.color = Some(self.expect_string()?),
                            "stroke" => style.stroke = Some(self.expect_string()?),
                            "strokewidth" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.stroke_width = Some(n as u32);
                                }
                            }
                            "fontsize" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.font_size = Some(n as u32);
                                }
                            }
                            "border" => style.border = Some(self.expect_identifier()?),
                            "opacity" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.opacity = Some(n as u32);
                                }
                            }
                            "metadata" => {
                                let val = self.expect_identifier()?;
                                style.metadata = Some(val.to_lowercase() == "true");
                            }
                            "description" => {
                                let val = self.expect_identifier()?;
                                style.description = Some(val.to_lowercase() == "true");
                            }
                            _ => {}
                        }
                    }
                    _ => break,
                }
            }
        }

        Ok(style)
    }

    fn parse_relationship_style(&mut self) -> Result<RelationshipStyleNode> {
        self.skip_newlines_and_comments();
        let tag = self.expect_string()?;

        let mut style = RelationshipStyleNode {
            tag,
            thickness: None,
            color: None,
            style: None,
            routing: None,
            font_size: None,
            width: None,
            position: None,
            opacity: None,
        };

        self.skip_newlines_and_comments();
        if self.check(&TokenKind::OpenBrace) {
            self.advance();

            loop {
                self.skip_newlines_and_comments();

                match self.current_kind().cloned() {
                    Some(TokenKind::CloseBrace) => {
                        self.advance();
                        break;
                    }
                    Some(TokenKind::Identifier(prop)) => {
                        self.advance();
                        match prop.to_lowercase().as_str() {
                            "thickness" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.thickness = Some(n as u32);
                                }
                            }
                            "color" | "colour" => style.color = Some(self.expect_string()?),
                            "style" => style.style = Some(self.expect_identifier()?),
                            "routing" => style.routing = Some(self.expect_identifier()?),
                            "fontsize" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.font_size = Some(n as u32);
                                }
                            }
                            "width" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.width = Some(n as u32);
                                }
                            }
                            "position" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.position = Some(n as u32);
                                }
                            }
                            "opacity" => {
                                if let Some(TokenKind::Number(n)) = self.current_kind().cloned() {
                                    self.advance();
                                    style.opacity = Some(n as u32);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => break,
                }
            }
        }

        Ok(style)
    }

    fn parse_properties_block(&mut self) -> Result<HashMap<String, String>> {
        self.skip_newlines_and_comments();
        self.expect(TokenKind::OpenBrace)?;

        let mut props = HashMap::new();

        loop {
            self.skip_newlines_and_comments();

            match self.current_kind() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::String(_)) => {
                    let key = self.expect_string()?;
                    let value = self.expect_string()?;
                    props.insert(key, value);
                }
                _ => break,
            }
        }

        Ok(props)
    }

    /// Parse a perspectives block: perspectives { "Name" "Description" ... }
    fn parse_perspectives_block(&mut self) -> Result<HashMap<String, String>> {
        self.skip_newlines_and_comments();
        self.expect(TokenKind::OpenBrace)?;

        let mut perspectives = HashMap::new();

        loop {
            self.skip_newlines_and_comments();

            match self.current_kind() {
                Some(TokenKind::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(TokenKind::String(_)) => {
                    let name = self.expect_string()?;
                    let description = self.expect_string()?;
                    perspectives.insert(name, description);
                }
                _ => break,
            }
        }

        Ok(perspectives)
    }
}

/// Process !include directives in the AST by reading and merging included files.
fn process_includes(mut ast: WorkspaceNode, base_path: &std::path::Path) -> Result<WorkspaceNode> {
    let mut included_model_nodes = Vec::new();
    let mut included_views_nodes = Vec::new();

    // Find all !include directives
    for directive in &ast.directives {
        if let Directive::Include(path) = directive {
            let include_path = base_path.join(path);
            let content = std::fs::read_to_string(&include_path)
                .map_err(|e| Error::Io(e))?;

            // Parse the included file as an AST
            let parent_dir = include_path.parent().unwrap_or(base_path);
            let mut lexer = Lexer::new(&content);
            let tokens = lexer.tokenize()?;
            let mut parser = Parser::new(tokens);
            let mut included_ast = parser.parse_workspace()?;

            // Recursively process includes in the included file
            included_ast = process_includes(included_ast, parent_dir)?;

            // Collect model and views from included AST
            if let Some(model) = included_ast.model {
                included_model_nodes.push(model);
            }
            if let Some(views) = included_ast.views {
                included_views_nodes.push(views);
            }
        }
    }

    // Merge included model nodes into the main AST
    if !included_model_nodes.is_empty() {
        let mut main_model = ast.model.take().unwrap_or_default();
        for included_model in included_model_nodes {
            main_model.elements.extend(included_model.elements);
            main_model.relationships.extend(included_model.relationships);
            main_model.groups.extend(included_model.groups);
        }
        ast.model = Some(main_model);
    }

    // Merge included views nodes into the main AST
    if !included_views_nodes.is_empty() {
        let mut main_views = ast.views.take().unwrap_or_default();
        for included_views in included_views_nodes {
            main_views.system_landscape.extend(included_views.system_landscape);
            main_views.system_context.extend(included_views.system_context);
            main_views.container.extend(included_views.container);
            main_views.component.extend(included_views.component);
            main_views.dynamic.extend(included_views.dynamic);
            main_views.deployment.extend(included_views.deployment);
            main_views.filtered.extend(included_views.filtered);
            main_views.custom.extend(included_views.custom);
            main_views.themes.extend(included_views.themes);
            // Merge styles if present
            if main_views.styles.is_none() {
                main_views.styles = included_views.styles;
            }
        }
        ast.views = Some(main_views);
    }

    // Remove !include directives from the final AST
    ast.directives.retain(|d| !matches!(d, Directive::Include(_)));

    Ok(ast)
}

/// Substitute constants in a string using the provided constants map.
fn substitute_constants(s: &str, constants: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (name, value) in constants {
        let pattern = format!("${{{}}}", name);
        result = result.replace(&pattern, value);
    }
    result
}

/// Apply constant substitution to all strings in the AST.
fn apply_constants_to_ast(mut ast: WorkspaceNode, constants: &HashMap<String, String>) -> WorkspaceNode {
    // Substitute in workspace properties
    if let Some(name) = ast.name {
        ast.name = Some(substitute_constants(&name, constants));
    }
    if let Some(desc) = ast.description {
        ast.description = Some(substitute_constants(&desc, constants));
    }

    // Substitute in model
    if let Some(mut model) = ast.model {
        model = apply_constants_to_model(model, constants);
        ast.model = Some(model);
    }

    // Substitute in views
    if let Some(mut views) = ast.views {
        views = apply_constants_to_views(views, constants);
        ast.views = Some(views);
    }

    ast
}

/// Apply constant substitution to model elements.
fn apply_constants_to_model(mut model: ModelNode, constants: &HashMap<String, String>) -> ModelNode {
    for element in &mut model.elements {
        apply_constants_to_element(element, constants);
    }
    for rel in &mut model.relationships {
        apply_constants_to_relationship(rel, constants);
    }
    for group in &mut model.groups {
        apply_constants_to_group(group, constants);
    }
    model
}

/// Apply constant substitution to a single element.
fn apply_constants_to_element(element: &mut ElementNode, constants: &HashMap<String, String>) {
    element.name = substitute_constants(&element.name, constants);
    if let Some(desc) = &element.description {
        element.description = Some(substitute_constants(desc, constants));
    }
    if let Some(tech) = &element.technology {
        element.technology = Some(substitute_constants(tech, constants));
    }
    for child in &mut element.children {
        apply_constants_to_element(child, constants);
    }
    for rel in &mut element.relationships {
        apply_constants_to_relationship(rel, constants);
    }
}

/// Apply constant substitution to a relationship.
fn apply_constants_to_relationship(rel: &mut RelationshipNode, constants: &HashMap<String, String>) {
    if let Some(desc) = &rel.description {
        rel.description = Some(substitute_constants(desc, constants));
    }
    if let Some(tech) = &rel.technology {
        rel.technology = Some(substitute_constants(tech, constants));
    }
}

/// Apply constant substitution to a group.
fn apply_constants_to_group(group: &mut GroupNode, constants: &HashMap<String, String>) {
    group.name = substitute_constants(&group.name, constants);
    for element in &mut group.elements {
        apply_constants_to_element(element, constants);
    }
    for nested_group in &mut group.groups {
        apply_constants_to_group(nested_group, constants);
    }
}

/// Apply constant substitution to views.
fn apply_constants_to_views(mut views: ViewsNode, constants: &HashMap<String, String>) -> ViewsNode {
    for view in &mut views.system_landscape {
        apply_constants_to_view_properties(&mut view.properties, constants);
    }
    for view in &mut views.system_context {
        apply_constants_to_view_properties(&mut view.properties, constants);
    }
    for view in &mut views.container {
        apply_constants_to_view_properties(&mut view.properties, constants);
    }
    for view in &mut views.component {
        apply_constants_to_view_properties(&mut view.properties, constants);
    }
    for view in &mut views.dynamic {
        apply_constants_to_view_properties(&mut view.properties, constants);
    }
    for view in &mut views.deployment {
        apply_constants_to_view_properties(&mut view.properties, constants);
    }
    views
}

/// Apply constant substitution to view properties.
fn apply_constants_to_view_properties(props: &mut ViewPropertiesNode, constants: &HashMap<String, String>) {
    if let Some(key) = &props.key {
        props.key = Some(substitute_constants(key, constants));
    }
    if let Some(title) = &props.title {
        props.title = Some(substitute_constants(title, constants));
    }
    if let Some(desc) = &props.description {
        props.description = Some(substitute_constants(desc, constants));
    }
}

/// Generate implied relationships based on existing relationships.
/// If A->B and B->C exist, create A->C.
fn generate_implied_relationships(workspace: &mut Workspace) {
    let mut implied_relationships = Vec::new();
    let relationships = &workspace.model.relationships;

    // Build a map of element -> destinations
    let mut outgoing: HashMap<ElementId, Vec<ElementId>> = HashMap::new();
    for rel in relationships {
        outgoing.entry(rel.source_id)
            .or_insert_with(Vec::new)
            .push(rel.destination_id);
    }

    // Find implied relationships
    for rel in relationships {
        // For each relationship A->B
        let a = rel.source_id;
        let b = rel.destination_id;

        // Check if B->C exists
        if let Some(b_destinations) = outgoing.get(&b) {
            for &c in b_destinations {
                // Check if A->C already exists
                let exists = relationships.iter().any(|r| {
                    r.source_id == a && r.destination_id == c
                });

                if !exists && a != c {
                    // Create implied relationship A->C
                    implied_relationships.push((a, c));
                }
            }
        }
    }

    // Add implied relationships to the workspace
    for (source, dest) in implied_relationships {
        workspace.model_mut().add_relationship(
            source,
            dest,
            String::new(), // Empty description for implied relationships
            None,
        );
    }
}

/// View type for element resolution.
#[derive(Debug, Clone, Copy)]
enum ViewType {
    SystemLandscape,
    SystemContext,
    Container,
    Component,
}

/// Resolve which elements should be included in a view based on include/exclude directives.
fn resolve_view_elements(
    model: &Model,
    view_type: ViewType,
    scope_id: Option<ElementId>,
    include: &[IncludeExclude],
    exclude: &[IncludeExclude],
    identifiers: &HashMap<String, ElementId>,
) -> Vec<ElementView> {
    let mut element_ids: Vec<ElementId> = Vec::new();

    // Determine if we should include all (empty include = include all, or explicit include *)
    let include_all = include.is_empty() || include.iter().any(|i| matches!(i, IncludeExclude::All));

    if include_all {
        // Include all applicable elements based on view type
        match view_type {
            ViewType::SystemLandscape | ViewType::SystemContext => {
                // All people and all software systems
                for person in &model.people {
                    element_ids.push(person.id());
                }
                for system in &model.software_systems {
                    element_ids.push(system.id());
                }
            }
            ViewType::Container => {
                // All people + target system's containers + all other systems
                for person in &model.people {
                    element_ids.push(person.id());
                }
                for system in &model.software_systems {
                    if Some(system.id()) == scope_id {
                        // Include containers from the target system
                        for container in &system.containers {
                            element_ids.push(container.id());
                        }
                    } else {
                        // Include external systems
                        element_ids.push(system.id());
                    }
                }
            }
            ViewType::Component => {
                // All people + target container's components + sibling containers
                for person in &model.people {
                    element_ids.push(person.id());
                }
                for system in &model.software_systems {
                    for container in &system.containers {
                        if Some(container.id()) == scope_id {
                            // Include components from the target container
                            for component in &container.components {
                                element_ids.push(component.id());
                            }
                        } else {
                            // Include sibling containers
                            element_ids.push(container.id());
                        }
                    }
                }
            }
        }
    } else {
        // Only include specifically listed elements
        for inc in include {
            match inc {
                IncludeExclude::All => {
                    // Already handled above
                }
                IncludeExclude::Identifier(name) => {
                    if let Some(&id) = identifiers.get(name) {
                        element_ids.push(id);
                    }
                }
                IncludeExclude::Expression(_expr) => {
                    // TODO: Implement tag expression matching
                    // For now, expressions are not supported
                }
            }
        }
    }

    // Apply excludes
    for exc in exclude {
        match exc {
            IncludeExclude::All => {
                element_ids.clear();
            }
            IncludeExclude::Identifier(name) => {
                if let Some(&id) = identifiers.get(name) {
                    element_ids.retain(|&eid| eid != id);
                }
            }
            IncludeExclude::Expression(_expr) => {
                // TODO: Implement tag expression matching for excludes
            }
        }
    }

    // Convert to ElementView
    element_ids.into_iter().map(ElementView::new).collect()
}

/// Build a Workspace from an AST.
fn build_workspace(mut ast: WorkspaceNode) -> Result<Workspace> {
    // Step 1: Process !const directives to build constants map
    let mut constants = HashMap::new();
    for directive in &ast.directives {
        if let Directive::Const { name, value } = directive {
            constants.insert(name.clone(), value.clone());
        }
    }

    // Step 2: Apply constant substitution to all strings in the AST
    if !constants.is_empty() {
        ast = apply_constants_to_ast(ast, &constants);
    }

    // Step 3: Check for !impliedRelationships directive
    let mut implied_relationships_enabled = false;
    for directive in &ast.directives {
        if let Directive::ImpliedRelationships(enabled) = directive {
            implied_relationships_enabled = *enabled;
        }
    }

    // Step 4: Build the workspace normally
    let mut workspace = Workspace::new(
        ast.name.unwrap_or_else(|| "Untitled".to_string()),
        ast.description.unwrap_or_default(),
    );

    workspace.properties = ast.properties;

    // Step 5: Store !docs and !adrs paths in workspace properties
    for directive in &ast.directives {
        match directive {
            Directive::Docs(path) => {
                workspace.set_property("structurizr.docs", path);
            }
            Directive::Adrs(path) => {
                workspace.set_property("structurizr.adrs", path);
            }
            _ => {}
        }
    }

    // Build identifier mapping
    let mut identifiers: HashMap<String, ElementId> = HashMap::new();

    // Process model
    if let Some(model_node) = ast.model {
        for element in &model_node.elements {
            build_element(&mut workspace, element, &mut identifiers)?;
        }

        // Build groups after all elements exist
        for group in &model_node.groups {
            build_group(&mut workspace, group, &mut identifiers)?;
        }

        // Build deployment environments
        for env in &model_node.deployment_environments {
            for child in &env.children {
                if child.kind == ElementKind::DeploymentNode {
                    let mut node = build_deployment_node(child, &mut identifiers)?;
                    node.environment = Some(env.name.clone());

                    // Register the node's identifier
                    if let Some(id) = &child.identifier {
                        identifiers.insert(id.clone(), node.id());
                    }

                    workspace.model_mut().deployment_nodes.push(node);
                }
            }
        }

        // Build relationships after all elements exist
        for rel in &model_node.relationships {
            build_relationship(&mut workspace, rel, &identifiers)?;
        }

        // Also process relationships defined inside elements
        for element in &model_node.elements {
            build_element_relationships(&mut workspace, element, &identifiers)?;
        }

        // Step 6: Generate implied relationships if enabled
        if implied_relationships_enabled {
            generate_implied_relationships(&mut workspace);
        }
    }

    // Process views
    if let Some(views_node) = ast.views {
        for view in views_node.system_landscape {
            let mut v = SystemLandscapeView::new(
                view.properties.key.unwrap_or_else(|| "SystemLandscape".to_string()),
            );
            if let Some(title) = view.properties.title {
                v.properties = v.properties.with_title(title);
            }
            if let Some(auto_layout) = view.properties.auto_layout {
                v.properties = v.properties.with_auto_layout(convert_auto_layout(auto_layout));
            }
            if let Some(background) = view.properties.background {
                v.properties = v.properties.with_background(background);
            }
            // Resolve include/exclude to populate elements
            v.properties.elements = resolve_view_elements(
                workspace.model(),
                ViewType::SystemLandscape,
                None,
                &view.properties.include,
                &view.properties.exclude,
                &identifiers,
            );
            workspace.views_mut().add_system_landscape_view(v);
        }

        for view in views_node.system_context {
            if let Some(&system_id) = identifiers.get(&view.software_system) {
                let mut v = SystemContextView::new(
                    view.properties.key.unwrap_or_else(|| format!("SystemContext-{}", view.software_system)),
                    system_id,
                );
                if let Some(title) = view.properties.title {
                    v.properties = v.properties.with_title(title);
                }
                if let Some(auto_layout) = view.properties.auto_layout {
                    v.properties = v.properties.with_auto_layout(convert_auto_layout(auto_layout));
                }
                if let Some(background) = view.properties.background {
                    v.properties = v.properties.with_background(background);
                }
                // Resolve include/exclude to populate elements
                v.properties.elements = resolve_view_elements(
                    workspace.model(),
                    ViewType::SystemContext,
                    Some(system_id),
                    &view.properties.include,
                    &view.properties.exclude,
                    &identifiers,
                );
                workspace.views_mut().add_system_context_view(v);
            }
        }

        for view in views_node.container {
            if let Some(&system_id) = identifiers.get(&view.software_system) {
                let mut v = ContainerView::new(
                    view.properties.key.unwrap_or_else(|| format!("Container-{}", view.software_system)),
                    system_id,
                );
                if let Some(title) = view.properties.title {
                    v.properties = v.properties.with_title(title);
                }
                if let Some(auto_layout) = view.properties.auto_layout {
                    v.properties = v.properties.with_auto_layout(convert_auto_layout(auto_layout));
                }
                if let Some(background) = view.properties.background {
                    v.properties = v.properties.with_background(background);
                }
                // Resolve include/exclude to populate elements
                v.properties.elements = resolve_view_elements(
                    workspace.model(),
                    ViewType::Container,
                    Some(system_id),
                    &view.properties.include,
                    &view.properties.exclude,
                    &identifiers,
                );
                workspace.views_mut().add_container_view(v);
            }
        }

        // Process component views
        for view in views_node.component {
            if let Some(&container_id) = identifiers.get(&view.container) {
                let mut v = ComponentView::new(
                    view.properties.key.unwrap_or_else(|| format!("Component-{}", view.container)),
                    container_id,
                );
                if let Some(title) = view.properties.title {
                    v.properties = v.properties.with_title(title);
                }
                if let Some(auto_layout) = view.properties.auto_layout {
                    v.properties = v.properties.with_auto_layout(convert_auto_layout(auto_layout));
                }
                if let Some(background) = view.properties.background {
                    v.properties = v.properties.with_background(background);
                }
                // Resolve include/exclude to populate elements
                v.properties.elements = resolve_view_elements(
                    workspace.model(),
                    ViewType::Component,
                    Some(container_id),
                    &view.properties.include,
                    &view.properties.exclude,
                    &identifiers,
                );
                workspace.views_mut().add_component_view(v);
            }
        }

        // Process dynamic views
        for view in views_node.dynamic {
            let mut v = if let Some(ref scope_id) = view.scope {
                if let Some(&element_id) = identifiers.get(scope_id) {
                    DynamicView::for_element(
                        view.properties.key.unwrap_or_else(|| format!("Dynamic-{}", scope_id)),
                        element_id,
                    )
                } else {
                    // Scope element not found - create unscoped view
                    DynamicView::new(
                        view.properties.key.unwrap_or_else(|| "Dynamic".to_string()),
                    )
                }
            } else {
                DynamicView::new(
                    view.properties.key.unwrap_or_else(|| "Dynamic".to_string()),
                )
            };

            // Apply view properties
            if let Some(title) = view.properties.title {
                v.properties = v.properties.with_title(title);
            }
            if let Some(description) = view.properties.description {
                v.properties = v.properties.with_description(description);
            }
            if let Some(auto_layout) = view.properties.auto_layout {
                v.properties = v.properties.with_auto_layout(convert_auto_layout(auto_layout));
            }
            if let Some(background) = view.properties.background {
                v.properties = v.properties.with_background(background);
            }

            // Convert content from AST nodes to core types with proper ordering
            v.steps = flatten_dynamic_content(&view.content, &identifiers);

            workspace.views_mut().add_dynamic_view(v);
        }

        // Process deployment views
        for view in views_node.deployment {
            let mut v = if let Some(ref scope_id) = view.scope {
                if let Some(&system_id) = identifiers.get(scope_id) {
                    DeploymentView::for_system(
                        view.properties.key.unwrap_or_else(|| format!("Deployment-{}-{}", scope_id, view.environment)),
                        system_id,
                        &view.environment,
                    )
                } else {
                    // Scope system not found - create unscoped view
                    DeploymentView::new(
                        view.properties.key.unwrap_or_else(|| format!("Deployment-{}", view.environment)),
                        &view.environment,
                    )
                }
            } else {
                DeploymentView::new(
                    view.properties.key.unwrap_or_else(|| format!("Deployment-{}", view.environment)),
                    &view.environment,
                )
            };

            // Apply view properties
            if let Some(title) = view.properties.title {
                v.properties = v.properties.with_title(title);
            }
            if let Some(description) = view.properties.description {
                v.properties = v.properties.with_description(description);
            }
            if let Some(auto_layout) = view.properties.auto_layout {
                v.properties = v.properties.with_auto_layout(convert_auto_layout(auto_layout));
            }
            if let Some(background) = view.properties.background {
                v.properties = v.properties.with_background(background);
            }

            workspace.views_mut().add_deployment_view(v);
        }

        // Process styles
        if let Some(styles_node) = views_node.styles {
            for elem_style in styles_node.elements {
                let mut style = ElementStyle::new(&elem_style.tag);
                if let Some(bg) = elem_style.background {
                    style = style.with_background(bg);
                }
                if let Some(color) = elem_style.color {
                    style = style.with_color(color);
                }
                if let Some(stroke) = elem_style.stroke {
                    style = style.with_stroke(stroke);
                }
                if let Some(font_size) = elem_style.font_size {
                    style = style.with_font_size(font_size);
                }
                if let Some(shape) = elem_style.shape {
                    if let Some(s) = parse_shape(&shape) {
                        style = style.with_shape(s);
                    }
                }
                if let Some(icon) = elem_style.icon {
                    style.icon = Some(icon);
                }
                if let Some(icon_pos) = elem_style.icon_position {
                    if let Some(pos) = parse_icon_position(&icon_pos) {
                        style.icon_position = Some(pos);
                    }
                }
                workspace.styles_mut().add_element_style(style);
            }

            for rel_style in styles_node.relationships {
                let mut style = RelationshipStyle::new(&rel_style.tag);
                if let Some(thickness) = rel_style.thickness {
                    style = style.with_thickness(thickness);
                }
                if let Some(color) = rel_style.color {
                    style = style.with_color(color);
                }
                if let Some(line_style) = rel_style.style {
                    if let Some(s) = parse_line_style(&line_style) {
                        style = style.with_style(s);
                    }
                }
                if let Some(routing) = rel_style.routing {
                    if let Some(r) = parse_routing(&routing) {
                        style = style.with_routing(r);
                    }
                }
                workspace.styles_mut().add_relationship_style(style);
            }
        }

        for theme in views_node.themes {
            workspace.styles_mut().add_theme(theme);
        }
    }

    Ok(workspace)
}

fn build_element(
    workspace: &mut Workspace,
    element: &ElementNode,
    identifiers: &mut HashMap<String, ElementId>,
) -> Result<ElementId> {
    let id = match element.kind {
        ElementKind::Person => {
            let mut person = Person::new(&element.name);
            if let Some(ref desc) = element.description {
                person = person.with_description(desc);
            }
            let id = person.id();
            workspace.model_mut().people.push(person);
            id
        }
        ElementKind::SoftwareSystem => {
            let mut system = SoftwareSystem::new(&element.name);
            if let Some(ref desc) = element.description {
                system = system.with_description(desc);
            }

            // Process child containers
            for child in &element.children {
                if child.kind == ElementKind::Container {
                    let container_id = build_container(&mut system, child, identifiers)?;
                    if let Some(ref child_id) = child.identifier {
                        identifiers.insert(child_id.clone(), container_id);
                    }
                }
            }

            let id = system.id();
            workspace.model_mut().software_systems.push(system);
            id
        }
        ElementKind::DeploymentNode => {
            let node = build_deployment_node(element, identifiers)?;
            let id = node.id();
            workspace.model_mut().deployment_nodes.push(node);
            id
        }
        _ => {
            return Err(Error::Parse(ParseError {
                message: format!("Unexpected element kind at top level: {:?}", element.kind),
                location: None,
            }));
        }
    };

    if let Some(ref identifier) = element.identifier {
        identifiers.insert(identifier.clone(), id);
    }

    Ok(id)
}

fn build_group(
    workspace: &mut Workspace,
    group: &GroupNode,
    identifiers: &mut HashMap<String, ElementId>,
) -> Result<()> {
    let mut element_ids = Vec::new();

    // Build elements within the group and collect their IDs
    for element in &group.elements {
        let id = build_element(workspace, element, identifiers)?;
        element_ids.push(id);
    }

    // Recursively handle nested groups
    for nested_group in &group.groups {
        build_group(workspace, nested_group, identifiers)?;
    }

    // Add the group to the model
    if !element_ids.is_empty() {
        let mut model_group = structurizr_core::model::Group::new(&group.name);
        model_group.element_ids = element_ids;
        workspace.model_mut().groups.push(model_group);
    }

    Ok(())
}

fn build_container(
    system: &mut SoftwareSystem,
    element: &ElementNode,
    identifiers: &mut HashMap<String, ElementId>,
) -> Result<ElementId> {
    let mut container = Container::new(&element.name);
    if let Some(ref desc) = element.description {
        container = container.with_description(desc);
    }
    if let Some(ref tech) = element.technology {
        container = container.with_technology(tech);
    }

    // Process child components
    for child in &element.children {
        if child.kind == ElementKind::Component {
            let mut component = Component::new(&child.name);
            if let Some(ref desc) = child.description {
                component = component.with_description(desc);
            }
            if let Some(ref tech) = child.technology {
                component = component.with_technology(tech);
            }
            let comp_id = component.id();
            container.add_component(component);

            if let Some(ref child_id) = child.identifier {
                identifiers.insert(child_id.clone(), comp_id);
            }
        }
    }

    let id = container.id();
    system.add_container(container);
    Ok(id)
}

fn build_deployment_node(
    element: &ElementNode,
    identifiers: &mut HashMap<String, ElementId>,
) -> Result<DeploymentNode> {
    let mut node = DeploymentNode::new(&element.name);

    if let Some(ref desc) = element.description {
        node.properties.description = Some(desc.clone());
    }
    if let Some(ref tech) = element.technology {
        node.technology = Some(tech.clone());
    }
    if let Some(ref instances) = element.instances {
        node.instances = instances.clone();
    }

    // Add tags
    for tag in &element.tags {
        if !node.properties.tags.contains(tag) {
            node.properties.tags.push(tag.clone());
        }
    }

    // Add properties
    for (key, value) in &element.properties {
        node.properties.properties.insert(key.clone(), value.clone());
    }

    // Register this node's identifier
    if let Some(ref id) = element.identifier {
        identifiers.insert(id.clone(), node.id());
    }

    // Process children
    for child in &element.children {
        match child.kind {
            ElementKind::DeploymentNode => {
                let child_node = build_deployment_node(child, identifiers)?;
                node.children.push(child_node);
            }
            ElementKind::InfrastructureNode => {
                let mut infra = InfrastructureNode::new(&child.name);
                if let Some(ref desc) = child.description {
                    infra.properties.description = Some(desc.clone());
                }
                if let Some(ref tech) = child.technology {
                    infra.technology = Some(tech.clone());
                }
                for tag in &child.tags {
                    if !infra.properties.tags.contains(tag) {
                        infra.properties.tags.push(tag.clone());
                    }
                }
                if let Some(ref id) = child.identifier {
                    identifiers.insert(id.clone(), infra.id());
                }
                node.infrastructure_nodes.push(infra);
            }
            ElementKind::ContainerInstance => {
                // The "name" here is actually the identifier of the container being instantiated
                // We need to look it up in identifiers, but it might not be resolved yet
                // For now, create a placeholder using a dummy ID
                let container_id = identifiers.get(&child.name).copied().unwrap_or_else(|| {
                    // Create a placeholder ID based on the name
                    ElementId::from_name(&child.name)
                });
                let instance = ContainerInstance {
                    id: ElementId::from_name(&format!("{}Instance", child.name)),
                    container_id,
                    instance_id: 1,
                    tags: child.tags.clone(),
                    properties: child.properties.clone(),
                };
                if let Some(ref id) = child.identifier {
                    identifiers.insert(id.clone(), instance.id);
                }
                node.container_instances.push(instance);
            }
            ElementKind::SoftwareSystemInstance => {
                // The "name" here is the identifier of the software system being instantiated
                let system_id = identifiers.get(&child.name).copied().unwrap_or_else(|| {
                    ElementId::from_name(&child.name)
                });
                let instance = SoftwareSystemInstance {
                    id: ElementId::from_name(&format!("{}Instance", child.name)),
                    software_system_id: system_id,
                    instance_id: 1,
                    tags: child.tags.clone(),
                    properties: child.properties.clone(),
                };
                if let Some(ref id) = child.identifier {
                    identifiers.insert(id.clone(), instance.id);
                }
                node.software_system_instances.push(instance);
            }
            _ => {}
        }
    }

    Ok(node)
}

fn build_relationship(
    workspace: &mut Workspace,
    rel: &RelationshipNode,
    identifiers: &HashMap<String, ElementId>,
) -> Result<()> {
    let source_id = identifiers.get(&rel.source).ok_or_else(|| {
        Error::UndefinedIdentifier(rel.source.clone())
    })?;
    let dest_id = identifiers.get(&rel.destination).ok_or_else(|| {
        Error::UndefinedIdentifier(rel.destination.clone())
    })?;

    workspace.model_mut().add_relationship_with_metadata(
        *source_id,
        *dest_id,
        rel.description.clone().unwrap_or_default(),
        rel.technology.clone(),
        rel.tags.clone(),
        rel.properties.clone(),
        rel.perspectives.clone(),
    );

    Ok(())
}

fn build_element_relationships(
    workspace: &mut Workspace,
    element: &ElementNode,
    identifiers: &HashMap<String, ElementId>,
) -> Result<()> {
    // Get this element's ID
    let source_id = if let Some(ref id) = element.identifier {
        identifiers.get(id).copied()
    } else {
        None
    };

    if let Some(source_id) = source_id {
        for rel in &element.relationships {
            if let Some(&dest_id) = identifiers.get(&rel.destination) {
                workspace.model_mut().add_relationship_with_metadata(
                    source_id,
                    dest_id,
                    rel.description.clone().unwrap_or_default(),
                    rel.technology.clone(),
                    rel.tags.clone(),
                    rel.properties.clone(),
                    rel.perspectives.clone(),
                );
            }
        }
    }

    // Recurse into children
    for child in &element.children {
        build_element_relationships(workspace, child, identifiers)?;
    }

    Ok(())
}

/// Tracks the current sequence number for dynamic view ordering.
/// Supports hierarchical numbering like "1", "2", "2.1", "2.2", "3" for parallel sequences.
struct SequenceCounter {
    /// Stack of sequence levels. Each level tracks the current index.
    /// e.g., [2, 1] represents "2.1"
    levels: Vec<u32>,
    /// Counter for parallel branches at the current level
    parallel_counter: u32,
    /// Whether we're currently processing parallel blocks
    in_parallel: bool,
}

impl SequenceCounter {
    fn new() -> Self {
        Self {
            levels: vec![0],
            parallel_counter: 0,
            in_parallel: false,
        }
    }

    /// Advance to the next sequential step and return its order string.
    fn next(&mut self) -> String {
        self.levels[0] += 1;
        self.parallel_counter = 0; // Reset parallel counter for new sequence
        self.current()
    }

    /// Start processing a parallel branch. Returns the order string for this branch.
    fn start_parallel_branch(&mut self) {
        if !self.in_parallel {
            // First parallel block at this level - increment main sequence
            self.levels[0] += 1;
            self.in_parallel = true;
        }
        self.parallel_counter += 1;
        self.levels.push(self.parallel_counter);
    }

    /// End a parallel branch, returning to the parent level.
    fn end_parallel_branch(&mut self) {
        if self.levels.len() > 1 {
            self.levels.pop();
        }
    }

    /// Signal that all parallel blocks at this level are done.
    fn finish_parallel_group(&mut self) {
        self.in_parallel = false;
    }

    /// Get current order as a string (e.g., "2" or "2.1")
    fn current(&self) -> String {
        self.levels
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

/// Flatten DynamicContent tree into a flat list of DynamicViewStep with order strings.
fn flatten_dynamic_content(
    content: &[DynamicContent],
    identifiers: &HashMap<String, ElementId>,
) -> Vec<DynamicViewStep> {
    let mut steps = Vec::new();
    let mut counter = SequenceCounter::new();

    flatten_content_recursive(content, identifiers, &mut steps, &mut counter);

    steps
}

fn flatten_content_recursive(
    content: &[DynamicContent],
    identifiers: &HashMap<String, ElementId>,
    steps: &mut Vec<DynamicViewStep>,
    counter: &mut SequenceCounter,
) {
    // Group consecutive parallel blocks together
    let mut i = 0;
    while i < content.len() {
        match &content[i] {
            DynamicContent::Step(step) => {
                // If we were in a parallel group, finish it
                counter.finish_parallel_group();

                // Regular sequential step
                if let (Some(&src), Some(&dst)) = (
                    identifiers.get(&step.source),
                    identifiers.get(&step.destination),
                ) {
                    let order = counter.next();
                    steps.push(DynamicViewStep {
                        source_id: src,
                        destination_id: dst,
                        description: step.description.clone(),
                        order,
                    });
                }
                i += 1;
            }
            DynamicContent::ParallelSequence(_) => {
                // Collect all consecutive parallel blocks
                let parallel_start = i;
                while i < content.len() {
                    if matches!(&content[i], DynamicContent::ParallelSequence(_)) {
                        i += 1;
                    } else {
                        break;
                    }
                }

                // Process each parallel block
                for j in parallel_start..i {
                    if let DynamicContent::ParallelSequence(parallel) = &content[j] {
                        counter.start_parallel_branch();
                        // Recursively process content within the parallel block
                        flatten_parallel_content(&parallel.content, identifiers, steps, counter);
                        counter.end_parallel_branch();
                    }
                }

                counter.finish_parallel_group();
            }
        }
    }
}

/// Flatten content within a parallel sequence block.
fn flatten_parallel_content(
    content: &[DynamicContent],
    identifiers: &HashMap<String, ElementId>,
    steps: &mut Vec<DynamicViewStep>,
    counter: &mut SequenceCounter,
) {
    for item in content {
        match item {
            DynamicContent::Step(step) => {
                if let (Some(&src), Some(&dst)) = (
                    identifiers.get(&step.source),
                    identifiers.get(&step.destination),
                ) {
                    let order = counter.current();
                    steps.push(DynamicViewStep {
                        source_id: src,
                        destination_id: dst,
                        description: step.description.clone(),
                        order,
                    });
                }
            }
            DynamicContent::ParallelSequence(nested) => {
                // Nested parallel - recurse
                flatten_parallel_content(&nested.content, identifiers, steps, counter);
            }
        }
    }
}

fn convert_auto_layout(node: AutoLayoutNode) -> AutoLayout {
    // Use AutoLayout's defaults when not specified in DSL
    let default_layout = AutoLayout::default();

    AutoLayout {
        direction: match node.direction {
            crate::ast::AutoLayoutDirection::TopBottom => structurizr_core::view::AutoLayoutDirection::TopBottom,
            crate::ast::AutoLayoutDirection::BottomTop => structurizr_core::view::AutoLayoutDirection::BottomTop,
            crate::ast::AutoLayoutDirection::LeftRight => structurizr_core::view::AutoLayoutDirection::LeftRight,
            crate::ast::AutoLayoutDirection::RightLeft => structurizr_core::view::AutoLayoutDirection::RightLeft,
        },
        rank_separation: node.rank_separation.unwrap_or(default_layout.rank_separation),
        node_separation: node.node_separation.unwrap_or(default_layout.node_separation),
    }
}

fn parse_shape(s: &str) -> Option<Shape> {
    match s.to_lowercase().as_str() {
        "box" => Some(Shape::Box),
        "roundedbox" => Some(Shape::RoundedBox),
        "circle" => Some(Shape::Circle),
        "ellipse" => Some(Shape::Ellipse),
        "hexagon" => Some(Shape::Hexagon),
        "cylinder" => Some(Shape::Cylinder),
        "pipe" => Some(Shape::Pipe),
        "person" => Some(Shape::Person),
        "robot" => Some(Shape::Robot),
        "folder" => Some(Shape::Folder),
        "webbrowser" => Some(Shape::WebBrowser),
        "mobiledeviceportrait" => Some(Shape::MobileDevicePortrait),
        "mobiledevicelandscape" => Some(Shape::MobileDeviceLandscape),
        "component" => Some(Shape::Component),
        _ => None,
    }
}

fn parse_line_style(s: &str) -> Option<LineStyle> {
    match s.to_lowercase().as_str() {
        "solid" => Some(LineStyle::Solid),
        "dashed" => Some(LineStyle::Dashed),
        "dotted" => Some(LineStyle::Dotted),
        _ => None,
    }
}

fn parse_routing(s: &str) -> Option<Routing> {
    match s.to_lowercase().as_str() {
        "direct" => Some(Routing::Direct),
        "orthogonal" => Some(Routing::Orthogonal),
        "curved" => Some(Routing::Curved),
        _ => None,
    }
}

fn parse_icon_position(s: &str) -> Option<IconPosition> {
    match s.to_lowercase().as_str() {
        "top" => Some(IconPosition::Top),
        "bottom" => Some(IconPosition::Bottom),
        "left" => Some(IconPosition::Left),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_workspace() {
        let dsl = r#"
workspace "Test" "A test workspace" {
    model {
        user = person "User" "A user"
        system = softwareSystem "System" "A system"
        user -> system "Uses"
    }
}
"#;
        let workspace = parse(dsl).unwrap();
        assert_eq!(workspace.name, "Test");
        assert_eq!(workspace.model().people.len(), 1);
        assert_eq!(workspace.model().software_systems.len(), 1);
        assert_eq!(workspace.model().relationships.len(), 1);
    }

    #[test]
    fn test_parse_with_views() {
        let dsl = r#"
workspace {
    model {
        user = person "User"
        system = softwareSystem "System"
    }
    views {
        systemContext system {
            include *
            autoLayout
        }
    }
}
"#;
        let workspace = parse(dsl).unwrap();
        assert_eq!(workspace.views().system_context_views.len(), 1);
    }

    #[test]
    fn test_const_directive() {
        let dsl = r#"
workspace "Test" {
    !const COMPANY_NAME "Acme Corp"
    !const TECH_STACK "Java/Spring"

    model {
        user = person "${COMPANY_NAME} User" "A user"
        system = softwareSystem "System" "Built with ${TECH_STACK}"
    }
}
"#;
        let workspace = parse(dsl).unwrap();
        assert_eq!(workspace.model().people.len(), 1);
        assert_eq!(workspace.model().people[0].name(), "Acme Corp User");
        assert_eq!(workspace.model().software_systems.len(), 1);
        assert_eq!(workspace.model().software_systems[0].properties.description, Some("Built with Java/Spring".to_string()));
    }

    #[test]
    fn test_implied_relationships() {
        let dsl = r#"
workspace "Test" {
    !impliedRelationships true

    model {
        a = person "A"
        b = softwareSystem "B"
        c = softwareSystem "C"

        a -> b "Uses"
        b -> c "Calls"
    }
}
"#;
        let workspace = parse(dsl).unwrap();
        // Should have 3 relationships: A->B, B->C, and implied A->C
        assert_eq!(workspace.model().relationships.len(), 3);
    }

    #[test]
    fn test_docs_adrs_directives() {
        let dsl = r#"
workspace "Test" {
    !docs "docs"
    !adrs "decisions"

    model {
        user = person "User"
    }
}
"#;
        let workspace = parse(dsl).unwrap();
        assert_eq!(workspace.get_property("structurizr.docs"), Some(&"docs".to_string()));
        assert_eq!(workspace.get_property("structurizr.adrs"), Some(&"decisions".to_string()));
    }

    #[test]
    fn test_implied_relationships_disabled() {
        let dsl = r#"
workspace "Test" {
    !impliedRelationships false

    model {
        a = person "A"
        b = softwareSystem "B"
        c = softwareSystem "C"

        a -> b "Uses"
        b -> c "Calls"
    }
}
"#;
        let workspace = parse(dsl).unwrap();
        // Should have only 2 relationships: A->B, B->C
        assert_eq!(workspace.model().relationships.len(), 2);
    }

    #[test]
    fn test_const_substitution_in_relationships() {
        let dsl = r#"
workspace "Test" {
    !const PROTOCOL "HTTPS"

    model {
        a = person "A"
        b = softwareSystem "B"

        a -> b "Uses" "${PROTOCOL}"
    }
}
"#;
        let workspace = parse(dsl).unwrap();
        assert_eq!(workspace.model().relationships.len(), 1);
        assert_eq!(workspace.model().relationships[0].technology, Some("HTTPS".to_string()));
    }
}
