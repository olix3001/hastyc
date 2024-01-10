mod items;
mod stmt;

pub use items::*;
pub use stmt::*;
use hastyc_common::{source::SourceFile, identifiers::{IDCounter, SymbolStorage, Ident, ASTNodeID}, span::Span, path::{Path, PathSegment}};

use crate::lexer::{TokenStream, Token, TokenKind};

use log::{debug, trace};

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
    Module,
    Import,
    Attribute
}

impl<'pkg, 'a> Parser<'pkg, 'a> {
    pub fn parse_from_root(root_file: &'a SourceFile, root_ts: &'a TokenStream) -> Result<Package, ParserError> {
        let counter = IDCounter::create();
        let mut package = Package {
            attrs: Attributes::empty(), // TODO: Parse global attributes
            items: ItemStream::empty(),
            id: (&counter).into(),
            idgen: counter,
            symbol_storage: SymbolStorage::new()
        };

        debug!(target: "parser", "Starting parse of package from root: {:?}.", root_file.name);
        let items = Self::parse_root_stream(root_file, root_ts, &mut package)?;

        package.items = items;

        trace!(target: "parser", "Package symbol storage dump: {:?}.", package.symbol_storage);
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
    /// Can return EOF, but clones the value, so peek is preferable.
    fn safe_peek(&self) -> Token {
        if self.is_at_end() { Token {
            kind: TokenKind::EOF,
            span: Span::dummy()
        }} else {
            self.peek().clone()
        }
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
            if self.is_at_end() {
                Err(
                    ParserError::ExpectedToken {
                        expected: tk,
                        found: Token { kind: TokenKind::EOF, span: Span::dummy() }
                    }
                )?
            }

            Err(
                ParserError::ExpectedToken {
                    expected: tk,
                    found: self.peek().clone()
                }
            )
        }
    }

    // Parsing functions
    pub fn parse_root_stream(root_file: &'a SourceFile, token_stream: &'a TokenStream, pkg: &mut Package) -> Result<ItemStream, ParserError> {
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

        pkg.symbol_storage = parser.symbol_storage;
        Ok(ItemStream::from_items(items))
    }

    /// Parse attribute like `#[attribute]`.
    pub fn try_parse_attribute(&mut self, _can_be_global: bool) -> Result<Option<Attribute>, ParserError> {
        //? can_be_global is a placeholder for later use
        // Try to parse hashtag
        if self.try_match(TokenKind::Hash) {
            self.consume(TokenKind::LeftBracket)?;
            
            let ident = self.expect_ident(
                ParserError::ExpectedName {
                    target: NameTarget::Attribute,
                    found: self.previous().clone()
                }
            )?;

            // Currently only option is unnamed argument, so just expect that
            self.consume(TokenKind::RightBracket)?;
            Ok(Some(Attribute { ident, kind: AttributeKind::FlagAttribute }))
        } else {
            Ok(None)
        }
    }

    /// Parse attributes. This can return empty vector if none are found.
    pub fn parse_attributes(&mut self) -> Result<Attributes, ParserError> {
        let mut attribs = Vec::new();
        loop {
            match self.try_parse_attribute(false)? {
                Some(attr) => attribs.push(attr),
                None => break
            }
        }
        Ok(Attributes {
            attributes: attribs
        })
    }

    /// Parse single item, this can be module definition, structure,
    /// trait, function or anything top-level.
    pub fn parse_item(&mut self) -> Result<Item, ParserError> {
        let attribs = self.parse_attributes()?;

        let vis = if self.try_match(TokenKind::Pub) {
            Visibility::Public
        } else { Visibility::Inherited };

        // Every item has its own keyword, which makes the work a lot easier :D
        let mut item = match self.advance().kind {
            TokenKind::Module => self.parse_module()?,
            TokenKind::Import => self.parse_import()?,
            _ => {
                Err(
                    ParserError::ExpectedItem {
                        found: self.previous().clone()
                    }
                )?
            }
        };

        item.visibility = vis;
        item.attrs = attribs;
        debug!(target: "parser",
            "Parsed item '{}' of type '{}'.",
            self.symbol_storage.text_of(item.ident.symbol).unwrap(),
            item.kind.name_of_type()
        );
        trace!(target: "parser", "Parsed item: {:?}.", item);
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
            attrs: Attributes::empty(),
            id: self.node_id(),
            visibility: Visibility::Inherited,
            kind: ItemKind::Module(ItemStream::from_items(items)),
            ident: name,
            span: Span::from_begin_end(span_keyword, span_end)
        })
    }

    /// Import like `import hello::world` or `import hello::{world, lorem::{ipsum, self}}`
    pub fn parse_import(&mut self) -> Result<Item, ParserError> {
        let span_keyword = self.previous().span;

        // Special cases: pkg and super
        let mut kind = ImportKind::Relative;
        if self.try_match(TokenKind::Pkg) {
            kind = ImportKind::Package;
            self.consume(TokenKind::DColon)?;
        } else if self.try_match(TokenKind::Super) {
            kind = ImportKind::Super;
            self.consume(TokenKind::DColon)?;
        }

        let tree = self.parse_import_tree()?;

        // Semicolon at the end of import :D
        self.consume(TokenKind::Semi)?;

        Ok(Item {
            attrs: Attributes::empty(),
            id: self.node_id(),
            visibility: Visibility::Inherited,
            kind: ItemKind::Import(kind, tree),
            ident: Ident::dummy(), // Import is the only item without name
            span: Span::from_begin_end(span_keyword, self.previous().span)
        })
    }

    pub fn parse_import_tree(&mut self) -> Result<ImportTree, ParserError> {
        let span_start = self.previous().span;
        let prefix = self.parse_import_prefix_path()?;
        trace!(target: "parser", "Found path with prefix '{:?}'.", prefix);

        // First: check for glob
        if self.try_match(TokenKind::Star) {
            let span = Span::from_begin_end(span_start, self.previous().span);
            return Ok(ImportTree::glob(prefix, span))
        }
        
        // Second: Self import
        let has_dcolon = self.previous().kind == TokenKind::DColon;
        if (has_dcolon || prefix.len() == 0) && self.try_match(TokenKind::LSelf) {
            return Ok(ImportTree::self_import(
                prefix,
                Span::from_begin_end(span_start, self.previous().span)
            ))
        }

        // Third: check for nested tree
        if has_dcolon && self.try_match(TokenKind::LeftBrace) {
            let mut subtrees = Vec::new();
            trace!(target: "parser", "Parsing nested import tree.");
            loop {
                let subtree = self.parse_import_tree()?;
                subtrees.push(subtree);
                if !self.try_match(TokenKind::Comma) { break; }
            }
            self.consume(TokenKind::RightBrace)?;
            return Ok(ImportTree::nested(
                prefix,
                subtrees.into_iter().map(|i| (i, self.node_id())).collect(),
                Span::from_begin_end(span_start, self.previous().span)
            ))
        }

        // Forth: Simple import
        if prefix.len() == 0 {
            Err(
                ParserError::ExpectedName {
                    target: NameTarget::Import,
                    found: self.safe_peek()
                }
            )?
        }

        Ok(ImportTree::simple(
            prefix, 
            Span::from_begin_end(span_start, self.previous().span)
        ))
    }

    /// For import like `hello::world::{lorem, ipsum}` prefix path would be the hello::world part.
    pub fn parse_import_prefix_path(&mut self) -> Result<Path, ParserError> {
        let span_start = self.previous().span;
        let mut path_segments = Vec::new();

        while self.check(TokenKind::Ident) {
            let ident = self.expect_ident(
                ParserError::ExpectedName { 
                    target: NameTarget::Import, 
                    found: self.previous().clone()
                }
            )?;

            path_segments.push(PathSegment::new(ident));

            // Check for double colon
            if !self.try_match(TokenKind::DColon) { break; }
        }
        let span_end = self.previous().span;
        let span = Span::from_begin_end(span_start, span_end);

        Ok(Path {
            segments: path_segments, 
            span
        })
    }
}