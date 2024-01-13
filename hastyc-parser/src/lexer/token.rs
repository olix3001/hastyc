use std::{sync::Arc, fmt::Debug};

use hastyc_common::{span::Span, identifiers::SourceFileID};

#[derive(Debug, Clone)]
pub struct TokenStream {
    pub source: SourceFileID,
    pub tokens: Arc<Vec<Token>>,
}

impl TokenStream {
    pub fn empty() -> Self {
        Self {
            source: SourceFileID(0),
            tokens: Arc::new(Vec::new())
        }
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, Token> {
        self.tokens.iter()
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
        .debug_set()
        .entry(&self.kind)
        .entry(&(self.span.start..self.span.end))
        .finish()
    }
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self {
            kind,
            span
        }
    }
}

/// Kind of token, this does not contain any information about its
/// contents nor any additional data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Comma, Dot, Minus,
    Plus, Semi, Slash, Star, Underscore, Bang,
    Equal, Less, Greater, Ampersand, Pipe, Colon, Percent,
    Dollar, Tilde, Question, Hash,

    // Two-character tokens
    BangEq, EqualEq, LessEq, GreaterEq,
    And, Or, Inc, Dec, DColon, ThinArrow, ThickArrow,

    // Keywords
    Fn, If, Else, True, False, While, For, In, Loop,
    Break, Continue, Return, LSelf, USelf, Let, Nil,
    Guard, Pub, Const, Static, Import, As, Module,
    Super, Pkg, Match, Struct, Trait, Impl, Enum,
    Getter, Setter, Override, Where, Async, Await,

    // Special and other
    Ident,
    Literal {
        kind: LiteralKind
    },

    // ONLY for error reporting
    EOF
}

/// Kind of literal token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    Int {
        base: Base
    },
    Float {
        has_exponent: bool
    },
    Char,
    Str,
    Any // Useful for error reporting
}

/// Numeric base of integer literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    Binary,
    Octal,
    Decimal,
    Hexadecimal
}