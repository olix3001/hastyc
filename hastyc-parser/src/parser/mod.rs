mod ast_node;

pub use ast_node::*;
use hastyc_common::{source::SourceFile, identifiers::{IDCounter, SymbolStorage, Ident, ASTNodeID}, span::Span};

use crate::lexer::{TokenStream, Token, TokenKind};

pub struct Parser<'pkg, 'a> {
    package: &'pkg Package,
    tokens: &'a TokenStream,
    current: usize,
    symbol_storage: SymbolStorage,
    source_file: Option<&'a SourceFile>
}

#[derive(Debug)]
pub enum ParserError {
    ExpectedToken {
        expected: TokenKind,
        found: Token
    },
    ExpectedItem {
        found: Token
    },
    ExpectedName {
        target: NameTarget,
        found: Token
    },
}

#[derive(Debug)]
pub enum NameTarget {
    Module
}

impl<'pkg, 'a> Parser<'pkg, 'a> {
    pub fn parse_from_root(root_file: &'a SourceFile, root_ts: &'a TokenStream) -> Result<Package, ParserError> {
        let counter = IDCounter::create();
        let mut package = Package {
            attrs: Attributes::default(), // TODO: Parse attributes
            items: ItemStream::empty(),
            id: (&counter).into(),
            idgen: counter
        };

        let items = Self::parse_stream(root_file, root_ts, &package)?;

        package.items = items;

        Ok(package)
    }

    // Utility functions
    fn node_id(&self) -> ASTNodeID {
        (&self.package.idgen).into()
    }

    fn ident(&mut self, token: &Token) -> Ident {
        if let Some(ref source_file) = self.source_file {
            let token_text = source_file.get_span(&token.span);
            Ident::new(
                self.symbol_storage.get_or_register(&token_text),
                token.span.clone()
            )
        } else {
            unimplemented!("Source code is unknown")
        }
    }

    fn expect_ident(&mut self, err: ParserError) -> Result<Ident, ParserError> {
        // Clone is there to avoid problems with multiple mutable borrows
        let token = self.expect(TokenKind::Ident, err)?.clone();
        Ok(self.ident(&token))
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> &Token {
        // Safety note: unwrapping here is safe as every time before calling
        // this method we check to ensure we are not at the end.
        self.tokens.iter().nth(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.iter().nth(self.current - 1).unwrap()
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        self.previous()
    }

    fn check(&self, tk: TokenKind) -> bool {
        if self.is_at_end() { return false; }
        self.peek().kind == tk
    }

    fn try_match(&mut self, tk: TokenKind) -> bool {
        if self.check(tk) {
            self.advance();
            return true;
        }
        false
    }
    
    /// Expect given token kind returning error if it doesn't match
    fn expect(&mut self, tk: TokenKind, err: ParserError) -> Result<&Token, ParserError> {
        if self.try_match(tk) { Ok(self.previous()) } else { Err(err) }
    }

    /// Consume token returning ExpectedToken error if it doesn't match
    fn consume(&mut self, tk: TokenKind) -> Result<&Token, ParserError> {
        if self.check(tk) {
            Ok(self.advance())
        } else {
            Err(
                ParserError::ExpectedToken {
                    expected: tk,
                    found: self.peek().clone()
                }
            )
        }
    }

    // Parsing functions
    pub fn parse_stream(root_file: &'a SourceFile, token_stream: &'a TokenStream, pkg: &Package) -> Result<ItemStream, ParserError> {
        let mut parser = Parser {
            tokens: token_stream,
            current: 0,
            symbol_storage: SymbolStorage::new(),
            source_file: Some(root_file),
            package: pkg,
        };
        let mut items = Vec::new();

        while !parser.is_at_end() {
            let item = parser.parse_item()?;
            items.push(item);
        }

        Ok(ItemStream::from_items(items))
    }

    /// Parse single item, this can be module definition, structure,
    /// trait, function or anything top-level.
    pub fn parse_item(&mut self) -> Result<Item, ParserError> {
        let vis = if self.try_match(TokenKind::Pub) {
            Visibility::Public
        } else { Visibility::Inherited };

        // Every item has its own keyword, which makes the work a lot easier :D
        let mut item = match self.advance().kind {
            TokenKind::Module => self.parse_module()?,
            _ => {
                Err(
                    ParserError::ExpectedItem {
                        found: self.previous().clone()
                    }
                )?
            }
        };

        item.visibility = vis;
        Ok(item)
    }

    /// Module definition like `module hello { ... }`.
    pub fn parse_module(&mut self) -> Result<Item, ParserError> {
        let span_keyword = self.previous().span;
        let name = self.expect_ident(
            ParserError::ExpectedName { 
                target: NameTarget::Module,
                found: self.previous().clone()
            }
        )?;

        self.consume(TokenKind::LeftBrace)?;

        let mut items = Vec::new();
        while !self.check(TokenKind::RightBrace) {
            let i = self.parse_item()?;
            items.push(i);
        }

        self.consume(TokenKind::RightBrace)?;

        let span_end = self.previous().span;
        Ok(Item {
            attrs: Attributes::default(), // TODO: Parse attributes !IMPORTANT!
            id: self.node_id(),
            visibility: Visibility::Inherited,
            kind: ItemKind::Module(ItemStream::from_items(items)),
            ident: name,
            span: Span::new(span_keyword.source, span_keyword.start, span_end.end)
        })
    }
}