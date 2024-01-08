mod ast_node;

pub use ast_node::*;
use hastyc_common::{source::SourceFile, identifiers::{IDCounter, SymbolStorage}};

use crate::lexer::{TokenStream, Token, TokenKind};

pub struct Parser<'a> {
    tokens: &'a TokenStream,
    current: usize,
    symbol_storage: SymbolStorage
}

#[derive(Debug)]
pub enum ParserError {
    ExpectedToken {
        expected: TokenKind,
        found: Token
    }
}

impl<'a> Parser<'a> {
    pub fn parse_from_root(root_file: &'a SourceFile, root_ts: &'a TokenStream) -> Result<Package, ParserError> {
        let counter = IDCounter::create();
        let mut package = Package {
            attrs: Attributes::default(), // TODO: Parse attributes
            items: ItemStream::empty(),
            id: (&counter).into(),
            idgen: counter
        };

        let items = Self::parse_stream(root_ts)?;

        Ok(package)
    }

    // Utility functions
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
    fn expect(&mut self, tk: TokenKind, err: ParserError) -> Result<(), ParserError> {
        if self.try_match(tk) { Ok(()) } else { Err(err) }
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
    pub fn parse_stream(token_stream: &'a TokenStream) -> Result<ItemStream, ParserError> {
        let parser = Parser {
            tokens: token_stream,
            current: 0,
            symbol_storage: SymbolStorage::new()
        };
        let mut items = Vec::new();

        while !parser.is_at_end() {
            let item = parser.parse_item()?;
            items.push(item);
        }

        Ok(ItemStream::from_items(items))
    }

    pub fn parse_item(&self) -> Result<Item, ParserError> {
        todo!("Parse different kinds of items")
    }
}