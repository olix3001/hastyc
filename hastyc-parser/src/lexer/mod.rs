mod token;
use std::sync::Arc;

use hastyc_common::{source::SourceFile, span::Span};
pub use token::*;

#[derive(Debug)]
pub enum LexerError {
    EmptySource,
    UnterminatedString {
        span: Span,
    },
    UnexpectedCharacter {
        position: u32
    }
}

pub struct Lexer<'a> {
    source: &'a SourceFile,
    src: &'a str,
    tokens: Vec<Token>,
    current: u32,
    start: u32
}

impl<'a> Lexer<'a> {
    /// Create TokenStream from the given source file.
    pub fn lex(source: &'a SourceFile) -> Result<TokenStream, LexerError> {
        if source.src.is_none() {
            return Err(LexerError::EmptySource)
        }

        let mut lexer = Lexer {
            source,
            src: source.src.as_ref().unwrap().as_str(),
            tokens: Vec::new(),
            current: 0,
            start: 0
        };

        while !lexer.is_at_end() {
            // Begin new span
            lexer.start = lexer.current;
            lexer.scan_token()?;
        }

        Ok(TokenStream {
            source: source.id,
            tokens: Arc::new(lexer.tokens)
        })
    }

    /// Check whether reader has reached the and of source file.
    fn is_at_end(&self) -> bool {
        self.current as usize >= self.source.clen
    }

    fn nth_src_char(&self, n: u32) -> char {
        self.src.chars().nth(n as usize).unwrap()
    }

    /// Get char and move cursor to the next one.
    fn advance(&mut self) -> char {
        let current_char = self.nth_src_char(self.current);
        self.current += 1;
        current_char
    }

    /// Check character without consuming it.
    fn peek(&self) -> char {
        if self.is_at_end() { return '\0' }
        self.nth_src_char(self.current)
    }

    /// Peek next character.
    fn peek_next(&self) -> char {
        if self.current as usize + 1 >= self.source.clen { return '\0' }
        self.nth_src_char(self.current + 1)
    }

    /// Add token to the currently built token stream.
    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token::new(
            kind,
            Span::new(self.source.id, self.start, self.current)
        ))
    }

    /// Tries to match character if possible, consuming it if matches.
    fn try_match(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false; }
        if self.peek() != expected { return false; }

        self.current += 1;
        return true;
    }

    /// Create span from current data
    fn cspan(&self) -> Span {
        Span::new(self.source.id, self.start, self.current)
    }

    /// Scan single token, this may produce multiple tokens in some cases.
    fn scan_token(&mut self) -> Result<(), LexerError> {
        // Utility macros
        macro_rules! try_match {
            ($char:literal => $a:ident | $b:ident) => {{
                let tt = if self.try_match($char) { TokenKind::$a } else { TokenKind::$b };
                self.add_token(tt);
            }};
        }

        let c = self.advance();
        match c {
            // Single-character
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            '[' => self.add_token(TokenKind::LeftBracket),
            ']' => self.add_token(TokenKind::RightBracket),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            ';' => self.add_token(TokenKind::Semi),
            '*' => self.add_token(TokenKind::Star),
            '%' => self.add_token(TokenKind::Percent),
            '~' => self.add_token(TokenKind::Tilde),
            '?' => self.add_token(TokenKind::Question),
            '#' => self.add_token(TokenKind::Hash),

            // Single or double
            ':' => try_match!(':' => DColon | Colon),
            '!' => try_match!('=' => BangEq | Bang),
            '=' => try_match!('=' => EqualEq | Equal),
            '<' => try_match!('=' => LessEq | Less),
            '>' => try_match!('=' => GreaterEq | Greater),
            '+' => try_match!('+' => Inc | Plus),
            '-' => try_match!('-' => Dec | Minus),
            '&' => try_match!('&' => And | Ampersand),
            '|' => try_match!('|' => Or | Pipe),

            // More complicated
            '/' => {
                // Comment
                if self.try_match('/') {
                    while self.peek() != '\n' && !self.is_at_end()
                        { self.advance(); }
                } else {
                    self.add_token(TokenKind::Slash)
                }
            }
            '"' => { self.string()?; },
            '\'' => { self.character()?; },
            '0'..='9' => { self.number()?; },
            '_' | '$' => {
                if self.peek().is_whitespace() {
                    self.add_token(match c {
                        '_' => TokenKind::Underscore,
                        '$' => TokenKind::Dollar,
                        _ => unreachable!()
                    });
                } else {
                    self.identifier()?;
                }
            },
            'A'..='z' => { self.identifier()?; },

            // Other
            ' ' | '\r' | '\t' | '\n' => { /* ignore */ },
            _ => {
                Err(
                    LexerError::UnexpectedCharacter { position: self.current - 1 }
                )?
            }
        }

        Ok(())
    }

    fn string(&mut self) -> Result<(), LexerError> {
        while self.peek() != '"' && !self.is_at_end() {
            // TODO: Support escape characters
            self.advance();
        }

        if self.is_at_end() {
            return Err(LexerError::UnterminatedString { 
                span: self.cspan()
            });
        }

        // Match closing '"'
        self.advance();
        self.add_token(TokenKind::Literal { kind: LiteralKind::Str });
        Ok(())
    }

    fn character(&mut self) -> Result<(), LexerError> {
        unimplemented!()
    }

    fn number(&mut self) -> Result<(), LexerError> {
        // TODO: Support for other radix numbers
        while self.peek().is_digit(10) { self.advance(); }
        let mut kind = TokenKind::Literal { 
            kind: LiteralKind::Int { base: Base::Decimal }
        };

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // TODO: Support exponent notation
            kind = TokenKind::Literal { 
                kind: LiteralKind::Float { has_exponent: false }
            };

            // Consume '.'
            self.advance();
            while self.peek().is_digit(10) { self.advance(); }
        }

        self.add_token(kind);
        Ok(())
    }

    fn identifier(&mut self) -> Result<(), LexerError> {
        while {
            let c = self.peek();
            c.is_alphanumeric() || c == '_'
        } {
            self.advance();
        }

        let text: String = self.src.chars()
            .skip(self.start as usize)
            .take(self.current as usize - self.start as usize)
            .collect();
        let text = text.as_str();

        self.add_token(
            match text {
                // Keywords
                "fn" => TokenKind::Fn,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "while" => TokenKind::While,
                "for" => TokenKind::For,
                "in" => TokenKind::In,
                "loop" => TokenKind::Loop,
                "break" => TokenKind::Break,
                "continue" => TokenKind::Continue,
                "return" => TokenKind::Return,
                "self" => TokenKind::LSelf,
                "Self" => TokenKind::USelf,
                "let" => TokenKind::Let,
                "nil" => TokenKind::Nil,
                "guard" => TokenKind::Guard,
                "pub" => TokenKind::Pub,
                "const" => TokenKind::Const,
                "static" => TokenKind::Static,
                "import" => TokenKind::Import,
                "as" => TokenKind::As,
                "module" => TokenKind::Module,
                "super" => TokenKind::Super,
                "pkg" => TokenKind::Pkg,
                "match" => TokenKind::Match,
                "struct" => TokenKind::Struct,
                "trait" => TokenKind::Trait,
                "impl" => TokenKind::Impl,
                "enum" => TokenKind::Enum,
                "getter" => TokenKind::Getter,
                "setter" => TokenKind::Setter,
                "override" => TokenKind::Override,
                "where" => TokenKind::Where,

                // Ident
                _ => TokenKind::Ident
            }
        );

        Ok(())
    }
}