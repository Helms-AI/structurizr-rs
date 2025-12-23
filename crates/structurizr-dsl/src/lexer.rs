//! Lexer for the Structurizr DSL.
//!
//! Tokenizes the input DSL into a stream of tokens.

use crate::error::{Location, Result, Error};

/// Token types for the Structurizr DSL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Workspace,
    Model,
    Views,
    Person,
    SoftwareSystem,
    Container,
    Component,
    DeploymentNode,
    InfrastructureNode,
    ContainerInstance,
    SoftwareSystemInstance,
    Group,
    SystemLandscape,
    SystemContext,
    Dynamic,
    Deployment,
    Filtered,
    Custom,
    Styles,
    Element,
    Relationship,
    Theme,
    Themes,
    Include,
    Exclude,
    AutoLayout,
    Animation,
    Properties,
    Perspectives,
    Tags,
    Technology,
    Description,
    Url,
    Extends,

    // Directives (start with !)
    DirectiveIdentifiers,
    DirectiveImpliedRelationships,
    DirectiveConst,
    DirectiveInclude,
    DirectiveDocs,
    DirectiveAdrs,
    DirectiveElement,
    DirectiveRelationship,
    DirectiveElements,
    DirectiveRelationships,
    DirectiveRef,

    // Symbols
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Arrow,       // ->
    Equals,
    Comma,
    Colon,
    Star,        // *

    // Literals
    String(String),
    Identifier(String),
    Number(i64),

    // Special
    Newline,
    Comment,
    Eof,
}

/// A token with its location in the source.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub location: Location,
    pub len: usize,
}

impl Token {
    pub fn new(kind: TokenKind, location: Location, len: usize) -> Self {
        Self { kind, location, len }
    }
}

/// Lexer for tokenizing DSL input.
pub struct Lexer<'a> {
    #[allow(dead_code)]
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn current_location(&self) -> Location {
        Location {
            line: self.line,
            column: self.column,
        }
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((_, c)) = result {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        result
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() && c != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        let mut depth = 1;
        while depth > 0 {
            match self.advance() {
                Some((_, '/')) if self.peek() == Some('*') => {
                    self.advance();
                    depth += 1;
                }
                Some((_, '*')) if self.peek() == Some('/') => {
                    self.advance();
                    depth -= 1;
                }
                None => break,
                _ => {}
            }
        }
    }

    #[allow(dead_code)]
    fn read_string(&mut self) -> Result<String> {
        let location = self.current_location();
        let mut s = String::new();

        // Skip opening quote
        self.advance();

        loop {
            match self.advance() {
                Some((_, '"')) => break,
                Some((_, '\\')) => {
                    // Handle escape sequences
                    match self.advance() {
                        Some((_, 'n')) => s.push('\n'),
                        Some((_, 'r')) => s.push('\r'),
                        Some((_, 't')) => s.push('\t'),
                        Some((_, '\\')) => s.push('\\'),
                        Some((_, '"')) => s.push('"'),
                        Some((_, c)) => s.push(c),
                        None => {
                            return Err(Error::Lexer {
                                message: "Unexpected end of input in string".to_string(),
                                location,
                            });
                        }
                    }
                }
                Some((_, c)) => s.push(c),
                None => {
                    return Err(Error::Lexer {
                        message: "Unterminated string".to_string(),
                        location,
                    });
                }
            }
        }

        Ok(s)
    }

    fn read_identifier(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        s
    }

    fn read_number(&mut self, first: char) -> i64 {
        let mut s = String::new();
        s.push(first);

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        s.parse().unwrap_or(0)
    }

    fn keyword_or_identifier(&self, s: &str) -> TokenKind {
        // Case-insensitive keyword matching
        match s.to_lowercase().as_str() {
            "workspace" => TokenKind::Workspace,
            "model" => TokenKind::Model,
            "views" => TokenKind::Views,
            "person" => TokenKind::Person,
            "softwaresystem" => TokenKind::SoftwareSystem,
            "container" => TokenKind::Container,
            "component" => TokenKind::Component,
            "deploymentnode" => TokenKind::DeploymentNode,
            "infrastructurenode" => TokenKind::InfrastructureNode,
            "containerinstance" => TokenKind::ContainerInstance,
            "softwaresysteminstance" => TokenKind::SoftwareSystemInstance,
            "group" => TokenKind::Group,
            "systemlandscape" => TokenKind::SystemLandscape,
            "systemcontext" => TokenKind::SystemContext,
            "dynamic" => TokenKind::Dynamic,
            "deployment" => TokenKind::Deployment,
            "filtered" => TokenKind::Filtered,
            "custom" => TokenKind::Custom,
            "styles" => TokenKind::Styles,
            "element" => TokenKind::Element,
            "relationship" => TokenKind::Relationship,
            "theme" => TokenKind::Theme,
            "themes" => TokenKind::Themes,
            "include" => TokenKind::Include,
            "exclude" => TokenKind::Exclude,
            "autolayout" => TokenKind::AutoLayout,
            "animation" => TokenKind::Animation,
            "properties" => TokenKind::Properties,
            "perspectives" => TokenKind::Perspectives,
            "tags" => TokenKind::Tags,
            "technology" => TokenKind::Technology,
            "description" => TokenKind::Description,
            "url" => TokenKind::Url,
            "extends" => TokenKind::Extends,
            _ => TokenKind::Identifier(s.to_string()),
        }
    }

    fn read_directive(&mut self) -> TokenKind {
        let mut s = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match s.to_lowercase().as_str() {
            "identifiers" => TokenKind::DirectiveIdentifiers,
            "impliedrelationships" => TokenKind::DirectiveImpliedRelationships,
            "const" => TokenKind::DirectiveConst,
            "include" => TokenKind::DirectiveInclude,
            "docs" => TokenKind::DirectiveDocs,
            "adrs" => TokenKind::DirectiveAdrs,
            "element" => TokenKind::DirectiveElement,
            "relationship" => TokenKind::DirectiveRelationship,
            "elements" => TokenKind::DirectiveElements,
            "relationships" => TokenKind::DirectiveRelationships,
            "ref" => TokenKind::DirectiveRef,
            _ => TokenKind::Identifier(format!("!{}", s)),
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let location = self.current_location();

        match self.advance() {
            None => Ok(Token::new(TokenKind::Eof, location, 0)),

            Some((_, '\n')) => Ok(Token::new(TokenKind::Newline, location, 1)),

            Some((_, '{')) => Ok(Token::new(TokenKind::OpenBrace, location, 1)),
            Some((_, '}')) => Ok(Token::new(TokenKind::CloseBrace, location, 1)),
            Some((_, '[')) => Ok(Token::new(TokenKind::OpenBracket, location, 1)),
            Some((_, ']')) => Ok(Token::new(TokenKind::CloseBracket, location, 1)),
            Some((_, '(')) => Ok(Token::new(TokenKind::OpenParen, location, 1)),
            Some((_, ')')) => Ok(Token::new(TokenKind::CloseParen, location, 1)),
            Some((_, '=')) => Ok(Token::new(TokenKind::Equals, location, 1)),
            Some((_, ',')) => Ok(Token::new(TokenKind::Comma, location, 1)),
            Some((_, ':')) => Ok(Token::new(TokenKind::Colon, location, 1)),
            Some((_, '*')) => Ok(Token::new(TokenKind::Star, location, 1)),

            Some((_, '-')) => {
                if self.peek() == Some('>') {
                    self.advance();
                    Ok(Token::new(TokenKind::Arrow, location, 2))
                } else if self.peek() == Some('-') {
                    // Could be extended arrow like --->
                    while self.peek() == Some('-') {
                        self.advance();
                    }
                    if self.peek() == Some('>') {
                        self.advance();
                        Ok(Token::new(TokenKind::Arrow, location, 3))
                    } else {
                        // Treat as identifier
                        let s = self.read_identifier('-');
                        let len = s.len();
                        Ok(Token::new(TokenKind::Identifier(s), location, len))
                    }
                } else {
                    let s = self.read_identifier('-');
                    let len = s.len();
                    Ok(Token::new(TokenKind::Identifier(s), location, len))
                }
            }

            Some((_, '/')) => {
                if self.peek() == Some('/') {
                    self.skip_line_comment();
                    Ok(Token::new(TokenKind::Comment, location, 0))
                } else if self.peek() == Some('*') {
                    self.advance();
                    self.skip_block_comment();
                    Ok(Token::new(TokenKind::Comment, location, 0))
                } else {
                    let s = self.read_identifier('/');
                    let len = s.len();
                    Ok(Token::new(TokenKind::Identifier(s), location, len))
                }
            }

            Some((_, '#')) => {
                self.skip_line_comment();
                Ok(Token::new(TokenKind::Comment, location, 0))
            }

            Some((_, '!')) => {
                let kind = self.read_directive();
                Ok(Token::new(kind, location, 1))
            }

            Some((_, '"')) => {
                // Put back the quote and read the string
                let s = self.read_string_from_current()?;
                let len = s.len() + 2; // Include quotes
                Ok(Token::new(TokenKind::String(s), location, len))
            }

            Some((_, c)) if c.is_ascii_digit() => {
                let n = self.read_number(c);
                Ok(Token::new(TokenKind::Number(n), location, 1))
            }

            Some((_, c)) if c.is_alphabetic() || c == '_' => {
                let s = self.read_identifier(c);
                let len = s.len();
                let kind = self.keyword_or_identifier(&s);
                Ok(Token::new(kind, location, len))
            }

            Some((_, c)) => Err(Error::Lexer {
                message: format!("Unexpected character: '{}'", c),
                location,
            }),
        }
    }

    fn read_string_from_current(&mut self) -> Result<String> {
        let location = self.current_location();
        let mut s = String::new();

        loop {
            match self.advance() {
                Some((_, '"')) => break,
                Some((_, '\\')) => {
                    match self.advance() {
                        Some((_, 'n')) => s.push('\n'),
                        Some((_, 'r')) => s.push('\r'),
                        Some((_, 't')) => s.push('\t'),
                        Some((_, '\\')) => s.push('\\'),
                        Some((_, '"')) => s.push('"'),
                        Some((_, c)) => s.push(c),
                        None => {
                            return Err(Error::Lexer {
                                message: "Unexpected end of input in string".to_string(),
                                location,
                            });
                        }
                    }
                }
                Some((_, c)) => s.push(c),
                None => {
                    return Err(Error::Lexer {
                        message: "Unterminated string".to_string(),
                        location,
                    });
                }
            }
        }

        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("workspace { }");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Workspace));
        assert!(matches!(tokens[1].kind, TokenKind::OpenBrace));
        assert!(matches!(tokens[2].kind, TokenKind::CloseBrace));
    }

    #[test]
    fn test_string_token() {
        let mut lexer = Lexer::new("\"Hello World\"");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "Hello World"));
    }

    #[test]
    fn test_arrow_token() {
        let mut lexer = Lexer::new("a -> b");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "a"));
        assert!(matches!(tokens[1].kind, TokenKind::Arrow));
        assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "b"));
    }

    #[test]
    fn test_directive() {
        let mut lexer = Lexer::new("!identifiers hierarchical");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::DirectiveIdentifiers));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "hierarchical"));
    }
}
