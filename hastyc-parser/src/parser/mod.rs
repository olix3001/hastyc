mod items;
mod stmt;

pub use items::*;
pub use stmt::*;
use hastyc_common::{source::SourceFile, identifiers::{IDCounter, SymbolStorage, Ident, ASTNodeID}, span::Span, path::{Path, PathSegment}};

use crate::lexer::{TokenStream, Token, TokenKind, LiteralKind};

use log::{debug, trace};

macro_rules! basic_binary_expression_impl {
    ($(for $name:ident use $fun:ident where $($kind:ident => $ty:ident),+);+;) => {
        $(
            fn $name(&mut self) -> Result<Expr, ParserError> {
                let span_start = self.previous().span;
                let lhs = self.$fun()?;
                let mut kind = lhs.kind;
                
                while $(self.try_match(TokenKind::$kind))||* {
                    let op_kind = self.previous().kind;
                    let rhs = self.$fun()?;
                    kind = ExprKind::Binary(
                        match op_kind {
                            $(TokenKind::$kind => BinOpKind::$ty),+,
                            _ => { unreachable!() }
                        }.spanned(self.previous().span),
                        Box::new(Expr {
                            id: self.node_id(),
                            kind,
                            span: lhs.span,
                            attrs: Attributes::empty()
                        }),
                        Box::new(rhs)
                    )    
                }

                Ok(Expr {
                    id: self.node_id(),
                    kind,
                    span: Span::from_begin_end(span_start, self.previous().span),
                    attrs: Attributes::empty()
                })
            }
        )+
    };
}

pub struct Parser<'pkg, 'a> {
    package: &'pkg Package,
    tokens: &'a TokenStream,
    current: usize,
    symbol_storage: SymbolStorage,
    source_file: &'a SourceFile
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
    Attribute,
    Fn,
    Type,
    Field
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
        let token_text = self.source_file.get_span(&token.span);
        Ident::new(
            self.symbol_storage.get_or_register(&token_text),
            token.span.clone()
        )
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

    fn unwind_one(&mut self) {
        self.current -= 1;
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
            source_file: root_file,
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
            TokenKind::Fn => self.parse_fn()?,
            _ => {
                self.unwind_one();
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

    pub fn parse_fn(&mut self) -> Result<Item, ParserError> {
        let span_start = self.previous().span;
        // get function name
        let ident = self.expect_ident(
            ParserError::ExpectedName {
                target: NameTarget::Fn,
                found: self.previous().clone()
            }
        )?;

        // Argument list
        let mut args = Vec::new();
        self.consume(TokenKind::LeftParen)?;
        while !self.check(TokenKind::RightParen) {
            let arg = self.parse_fn_arg()?;
            args.push(arg);

            if !self.try_match(TokenKind::Comma) {
                break;
            }
        }
        self.consume(TokenKind::RightParen)?;

        // Possible return type
        let ret_ty = if self.try_match(TokenKind::ThinArrow) {
            FnRetTy::Ty(self.parse_ty()?)
        } else { FnRetTy::Default };

        let sig_span_end = self.previous().span;

        // Body
        // Temporary
        let block = self.parse_block()?;

        // Return
        Ok(Item {
            attrs: Attributes::empty(),
            id: self.node_id(),
            visibility: Visibility::Inherited,
            kind: ItemKind::Fn(
                Function {
                    generics: Generics {},
                    signature: FnSignature {
                        is_const: false,
                        is_async: false,
                        inputs: args,
                        output: ret_ty,
                        span: Span::from_begin_end(span_start, sig_span_end)
                    },
                    body: Some(Box::new(block))
                }
            ),
            ident,
            span: Span::from_begin_end(span_start, self.previous().span)
        })
    }

    pub fn parse_fn_arg(&mut self) -> Result<FnInput, ParserError> {
        let attributes = self.parse_attributes()?;
        let span_start = self.previous().span;

        // Special case: self
        if self.try_match(TokenKind::LSelf) {
            return Ok(
                FnInput {
                    attributes,
                    id: self.node_id(),
                    span: self.previous().span,
                    pat: Pat {
                        id: self.node_id(),
                        kind: PatKind::SelfPat,
                        span: self.previous().span
                    },
                    ty: Ty {
                        id: self.node_id(),
                        kind: TyKind::SelfTy,
                        span: self.previous().span
                    }
                }
            )
        }

        let pat = self.parse_pattern()?;
        self.consume(TokenKind::Colon)?;
        let ty = self.parse_ty()?;

        Ok(FnInput {
            attributes,
            id: self.node_id(),
            span: Span::from_begin_end(span_start, self.previous().span),
            pat,
            ty 
        })
    }

    pub fn parse_pattern(&mut self) -> Result<Pat, ParserError> {
        // TODO: Add more patterns
        if let Ok(token) = self.consume(TokenKind::Ident) {
            let token = token.clone();
            return Ok(
                Pat {
                    id: self.node_id(),
                    kind: PatKind::Ident(self.ident(&token)),
                    span: token.span
                }
            )
        }
        unimplemented!("Only ident patterns are available")
    }

    pub fn parse_ty(&mut self) -> Result<Ty, ParserError> {
        let span_start = self.previous().span;
        // Void / Tuple
        if self.try_match(TokenKind::LeftParen) {
            if self.try_match(TokenKind::RightParen) {
                return Ok(Ty {
                    id: self.node_id(),
                    kind: TyKind::Void,
                    span: Span::from_begin_end(span_start, self.previous().span)
                });
            }
            unimplemented!("Tuple type is not yet supported")
        }

        // Never type
        if self.try_match(TokenKind::Bang) {
            return Ok(Ty {
                id: self.node_id(),
                kind: TyKind::Never,
                span: self.previous().span
            });
        }

        // Path type
        let path = self.parse_path()?;
        let path_span = path.span;
        Ok(Ty {
            id: self.node_id(),
            kind: TyKind::Path(path),
            span: path_span
        })
    }

    pub fn parse_path(&mut self) -> Result<Path, ParserError> {
        let mut segments = Vec::new();
        let span_start = self.previous().span;

        loop {
            let segment = self.parse_path_segment()?;
            segments.push(segment);
            if !self.try_match(TokenKind::DColon) {
                break;
            }
        }

        Ok(Path {
            segments,
            span: Span::from_begin_end(span_start, self.previous().span)
        })
    }

    pub fn parse_path_segment(&mut self) -> Result<PathSegment, ParserError> {
        let ident = self.expect_ident(
            ParserError::ExpectedName {
                target: NameTarget::Type,
                found: self.previous().clone()
            }
        )?;

        Ok(PathSegment { ident })
    }
    
    pub fn parse_block(&mut self) -> Result<Block, ParserError> {
        let span_start = self.previous().span;

        self.consume(TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();

        loop {
            if self.try_match(TokenKind::RightBrace) {
                break;
            }
            // TODO: Check is at end

            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        Ok(Block {
            stmts: StmtStream::from_vec(stmts),
            id: self.node_id(),
            span: Span::from_begin_end(span_start, self.previous().span)
        })
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        let span_start = self.previous().span;
        // First: Let binding
        let attrib = self.parse_attributes();
        if self.try_match(TokenKind::Let) {
            let mut kind = StmtKind::LetBinding(Box::new(self.parse_let_binding()?));
            if let Ok(attrib) = attrib {
                let StmtKind::LetBinding(ref mut lb) = kind else { unreachable!() };
                lb.attribs = attrib;
            }
            
            self.consume(TokenKind::Semi)?;

            return Ok(Stmt {
                id: self.node_id(),
                kind,
                span: Span::from_begin_end(span_start, self.previous().span)
            })   
        } else {
            match self.parse_item() {
                Ok(item) => {
                    let kind = StmtKind::Item(Box::new(item));
                    return Ok(Stmt {
                        id: self.node_id(),
                        kind,
                        span: Span::from_begin_end(span_start, self.previous().span)
                    });
                },
                Err(ParserError::ExpectedItem { .. }) => { /* ignore and check next */},
                Err(err) => return Err(err) 
            }

            // This is neither let binding nor an item.
            let expr = self.parse_expr()?;
            let kind = if self.try_match(TokenKind::Semi) {
                StmtKind::Expr(Box::new(expr))
            } else {
                StmtKind::ExprNS(Box::new(expr))
            };

            Ok(Stmt {
                id: self.node_id(),
                kind,
                span: Span::from_begin_end(span_start, self.previous().span)
            })
        }
    }

    pub fn parse_let_binding(&mut self) -> Result<LetBinding, ParserError> {
        let span_start = self.previous().span;
        let pat = self.parse_pattern()?;
        let ty = if self.try_match(TokenKind::Colon) {
            self.parse_ty()?
        } else { Ty {
            id: self.node_id(),
            kind: TyKind::Infer,
            span: self.previous().span
        } };

        let kind = if self.try_match(TokenKind::Equal) {
            LetBindingKind::Init(Box::new(self.parse_expr()?))
        } else {
            LetBindingKind::Decl
        };

        Ok(LetBinding {
            id: self.node_id(),
            pat,
            ty: Some(ty),
            kind,
            span: Span::from_begin_end(span_start, self.previous().span),
            attribs: Attributes::empty()
        })
    }

    pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.expr_block()
    }

    fn expr_block(&mut self) -> Result<Expr, ParserError> {
        if self.check(TokenKind::LeftBrace) {
            let block = self.parse_block()?;
            let span = block.span;
            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::Block(Box::new(block)),
                span,
                attrs: Attributes::empty()
            });
        }
        self.expr_if()
    }

    fn expr_if(&mut self) -> Result<Expr, ParserError> {
        if self.try_match(TokenKind::If) {
            let span_start = self.previous().span;
            let condition = self.parse_expr()?;
            let block = self.parse_block()?;
            let else_expr = if self.try_match(TokenKind::Else) {
                Some(Box::new(self.parse_expr()?))
            } else { None };

            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::If(
                    Box::new(condition),
                    Box::new(block),
                    else_expr
                ),
                span: Span::from_begin_end(span_start, self.previous().span),
                attrs: Attributes::empty()
            })
        }

        self.expr_loop()
    }

    fn expr_loop(&mut self) -> Result<Expr, ParserError> {
        if self.try_match(TokenKind::Loop) {
            let span_start = self.previous().span;
            let block = self.parse_block()?;

            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::Loop(Box::new(block)),
                span: Span::from_begin_end(span_start, self.previous().span),
                attrs: Attributes::empty()
            })
        }

        if self.try_match(TokenKind::While) {
            let span_start = self.previous().span;
            let condition = self.parse_expr()?;
            let block = self.parse_block()?;

            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::While(Box::new(condition), Box::new(block)),
                span: Span::from_begin_end(span_start, self.previous().span),
                attrs: Attributes::empty()
            })
        }

        if self.try_match(TokenKind::For) {
            let span_start = self.previous().span;
            let pat = self.parse_pattern()?;
            self.consume(TokenKind::In)?;
            let expr = self.parse_expr()?;
            let block = self.parse_block()?;

            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::For(pat, Box::new(expr), Box::new(block)),
                span: Span::from_begin_end(span_start, self.previous().span),
                attrs: Attributes::empty()
            })
        }

        self.expr_break_continue()
    }

    fn expr_break_continue(&mut self) -> Result<Expr, ParserError> {
        if self.try_match(TokenKind::Continue) {
            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::Continue,
                span: self.previous().span,
                attrs: Attributes::empty()
            })
        } else if self.try_match(TokenKind::Break) {
            let span_start = self.previous().span;
            let expr = if self.check(TokenKind::Semi) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::Break(expr.map(|e| Box::new(e))),
                span: Span::from_begin_end(span_start, self.previous().span),
                attrs: Attributes::empty()
            })
        }

        self.expr_logic_or()   
    }

    basic_binary_expression_impl!(
        for expr_logic_or use expr_logic_and where Or => Or;
        for expr_logic_and use expr_equality where And => And;
        for expr_equality use expr_comparison where
            EqualEq => Eq, BangEq => Ne;
        for expr_comparison use expr_term where
            Greater => Gt, GreaterEq => Ge,
            Less => Lt, LessEq => Le;
        for expr_term use expr_factor where
            Plus => Add, Minus => Sub;
        for expr_factor use expr_unary where
            Slash => Div, Star => Mul;
    );

    fn expr_unary(&mut self) -> Result<Expr, ParserError> {
        if self.try_match(TokenKind::Bang) || self.try_match(TokenKind::Minus) {
            let token_span = self.previous().span;
            let op_kind = self.previous().kind;
            let right = self.expr_unary()?;
            let right_span = right.span;
            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::Unary(
                    match op_kind {
                        TokenKind::Bang => UnOpKind::Not,
                        TokenKind::Minus => UnOpKind::Neg,
                        _ => unreachable!()
                    },
                    Box::new(right)
                ),
                span: Span::from_begin_end(token_span, right_span),
                attrs: Attributes::empty()
            })
        }

        self.expr_call()
    }

    fn expr_call(&mut self) -> Result<Expr, ParserError> {
        let expr = self.expr_assignment()?;

        if self.try_match(TokenKind::LeftParen) {
            let args_start = self.previous().span;
            // Argument list
            let mut args = Vec::new();
            while !self.try_match(TokenKind::RightParen) {
                let arg_expr = self.parse_expr()?;
                args.push(Box::new(arg_expr));
                if !self.try_match(TokenKind::Comma) {
                    self.consume(TokenKind::RightParen)?;
                    break;
                }
            }

            return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::Call(Box::new(expr), args),
                span: Span::from_begin_end(args_start, self.previous().span),
                attrs: Attributes::empty()
            })
        }

        Ok(expr)
    }
    
    fn expr_assignment(&mut self) -> Result<Expr, ParserError> {
        let lvalue = self.expr_field_access()?;

        if self.try_match(TokenKind::Equal) {
            let rvalue = self.parse_expr()?;
            let span = Span::from_begin_end(lvalue.span, rvalue.span);

           return Ok(Expr {
                id: self.node_id(),
                kind: ExprKind::Assign(Box::new(lvalue), Box::new(rvalue)),
                span,
                attrs: Attributes::empty()
            })
        }

       Ok(lvalue)
    }

    fn expr_field_access(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.expr_primary()?;

        while self.try_match(TokenKind::Dot) {
            let ident = self.expect_ident(
                ParserError::ExpectedName {
                    target: NameTarget::Field,
                    found: self.safe_peek().clone()
                }
            )?;

            let ident_span = ident.span;
            expr = Expr {
                id: self.node_id(),
                kind: ExprKind::Field(Box::new(expr), ident),
                span: ident_span,
                attrs: Attributes::empty()
            }
        }

        Ok(expr)
    }

    fn expr_primary(&mut self) -> Result<Expr, ParserError> {
        let span_start = self.previous().span;

        // Grouping
        if self.try_match(TokenKind::LeftParen) {
            let expr = self.parse_expr()?;
            self.consume(TokenKind::RightParen)?;
            return Ok(expr);
        }

        // Path expr
        let kind = if let Ok(path) = self.parse_path() {
            ExprKind::Path(path)
        } else if let Ok(lit) = self.parse_lit() {
            ExprKind::Literal(lit)
        } else { unimplemented!("Only path and literal expressions are implemented, found token: {:?}", self.safe_peek()) };

        Ok(Expr {
            id: self.node_id(),
            kind,
            span: Span::from_begin_end(span_start, self.previous().span),
            attrs: Attributes::empty()
        })
    }

    /// Try to parse literal
    pub fn parse_lit(&mut self) -> Result<Lit, ParserError> {
        if let TokenKind::Literal { .. } = self.peek().kind {
            let token = self.advance();
            let TokenKind::Literal { kind } = token.kind else { unreachable!() };
            
            let lit_kind = match kind { // TODO: Fix bases
                LiteralKind::Int { base: _base } => LitKind::Integer,
                LiteralKind::Float { has_exponent: _has_exponent } => LitKind::Float,
                LiteralKind::Str => LitKind::String,
                LiteralKind::Char => LitKind::Char,
                _ => unreachable!() // Any cannot be produced by the lexer
            };

            let t_span = token.span; // For borrow checker satisfaction
            Ok(Lit {
                id: self.node_id(),
                kind: lit_kind,
                symbol: self.symbol_storage.get_or_register(
                    &self.source_file.get_span(&t_span)
                )
            })
        } else {
            Err(ParserError::ExpectedToken { 
                expected: TokenKind::Literal 
                    { kind: crate::lexer::LiteralKind::Any },
                found: self.safe_peek().clone() })
        }
    }
}
