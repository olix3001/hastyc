use std::collections::HashMap;

use hastyc_common::identifiers::ASTNodeID;
use hastyc_parser::parser::{Block, DataVariant, Expr, FieldDef, FnInput, Function, Item, ItemKind, ItemStream, LetBinding, Package, Pat, Stmt, StmtKind, StmtStream, Ty};

pub mod name_resolve;

/// Context for the current compiler pass. This contains all information about resolved
/// names, types, and other things.
pub struct QueryContext<'ctx> {
    pub package: &'ctx Package,
    /// Mapping of which AST node refers to which AST node
    pub resolved_names: HashMap<ASTNodeID, ASTNodeID>
}

/// Pass that modifies AST or query context
pub trait ASTPass<'ctx> {
    type Err;

    fn traverse(&mut self, ctx: &'ctx mut QueryContext) -> Result<(), Self::Err> {
        self.traverse_itemstream(&ctx.package.items, ctx)
    }
    fn traverse_itemstream(&mut self, stream: &ItemStream, ctx: &mut QueryContext) -> Result<(), Self::Err> {
        for item in stream.items.iter() {
            self.visit_item(item, ctx)?;
        }
        Ok(())
    }
    fn traverse_stmtstream(&mut self, stream: &StmtStream, ctx: &mut QueryContext) -> Result<(), Self::Err> {
        for stmt in stream.stmts.iter() {
            self.visit_stmt(stmt, ctx)?;
        }
        Ok(())
    }

    fn visit_item(&mut self, item: &Item, ctx: &mut QueryContext) -> Result<(), Self::Err>;
    fn visit_stmt(&mut self, stmt: &Stmt, ctx: &mut QueryContext) -> Result<(), Self::Err>;
    fn visit_expr(&mut self, expr: &Expr, ctx: &mut QueryContext) -> Result<(), Self::Err>;
    fn finish(&mut self, _ctx: &mut QueryContext) -> Result<(), Self::Err> { Ok(()) }
}

impl<'cx> QueryContext<'cx> {
    pub fn for_package(
        package: &'cx Package
    ) -> Self {
        Self {
            package: &package,
            resolved_names: HashMap::new()
        }
    }

    pub fn query<Q>(&'cx self, query: Q) -> Q::Result<'cx> where Q: Query {
        query.run(self)
    }
}

pub trait Query {
    type Result<'cx>;

    fn run<'cx>(&self, cx: &'cx QueryContext) -> Self::Result<'cx>;
}

pub struct ResolveIdQuery(ASTNodeID);

pub enum ResolvedId<'cx> {
    Unknown,
    Item(&'cx Item),
    Expr(&'cx Expr),
    Stmt(&'cx Stmt),
    FnInput(&'cx FnInput),
    Block(&'cx Block),
    LetBinding(&'cx LetBinding),
    Pat(&'cx Pat),
    Ty(&'cx Ty),
    FieldDef(&'cx FieldDef)
}

impl Query for ResolveIdQuery {
    type Result<'cx> = ResolvedId<'cx>;

    fn run<'cx>(&self, cx: &'cx QueryContext) -> Self::Result<'cx> {
        let Some(resolved) = self.item_stream(&cx.package.items)
            else { return ResolvedId::Unknown };
        
        resolved
    }
}

impl ResolveIdQuery {
    fn item_stream<'cx>(&self, is: &'cx ItemStream) -> Option<ResolvedId<'cx>> {
        for item in is.items.iter() {
            if let Some(r) = self.item(item) {
                return Some(r)
            } 
        }
        None
    }

    fn item<'cx>(&self, i: &'cx Item) -> Option<ResolvedId<'cx>> {
        if i.id == self.0 { return Some(ResolvedId::Item(i)) }
        match i.kind {
            ItemKind::Module(ref is) => self.item_stream(is),
            ItemKind::Fn(ref fun) => self.fun(fun),
            ItemKind::Import(ref kind, ref tree) => todo!(),
            ItemKind::Struct(ref datavar) => self.datavar(datavar),
            ItemKind::Enum(ref datavar) => todo!(),
            _ => None
        }
    }

    fn stmt_stream<'cx>(&self, ss: &'cx StmtStream) -> Option<ResolvedId<'cx>> {
        for stmt in ss.stmts.iter() {
            if let Some(r) = self.stmt(stmt) {
                return Some(r)
            }
        }
        None
    }

    fn stmt<'cx>(&self, s: &'cx Stmt) -> Option<ResolvedId<'cx>> {
        if s.id == self.0 { return Some(ResolvedId::Stmt(s)) }
        match s.kind {
            StmtKind::LetBinding(ref binding) => {
                if binding.id == self.0 { return Some(ResolvedId::LetBinding(binding)) }
                if binding.pat.id == self.0 { return Some(ResolvedId::Pat(&binding.pat)) }
                if let Some(ref ty) = binding.ty {
                    if ty.id == self.0 { return Some(ResolvedId::Ty(ty)) }
                }
                None
            },
            StmtKind::Item(ref item) => self.item(item),
            StmtKind::Expr(ref expr) => self.expr(expr),
            StmtKind::ExprNS(ref expr) => self.expr(expr),
            _ => None
        }
    }

    fn expr<'cx>(&self, e: &'cx Expr) -> Option<ResolvedId<'cx>> {
        if e.id == self.0 { return Some(ResolvedId::Expr(e)) }
        None
    }

    fn fun<'cx>(&self, fun: &'cx Function) -> Option<ResolvedId<'cx>> {
        for input in fun.signature.inputs.iter() {
            if input.id == self.0 { return Some(ResolvedId::FnInput(input)) }
        }
        if let Some(ref body) = fun.body {
            if body.id == self.0 { return Some(ResolvedId::Block(body)) }
            return self.stmt_stream(&body.stmts);
        }
        None
    }

    fn datavar<'cx>(&self, dv: &'cx DataVariant) -> Option<ResolvedId<'cx>> {
        match dv {
            DataVariant::Unit => None,
            DataVariant::Struct { ref fields } => {
                for field in fields.iter() {
                    if field.id == self.0 { return Some(ResolvedId::FieldDef(field)) }
                    if field.ty.id == self.0 { return Some(ResolvedId::Ty(&field.ty)) }
                }
                None
            },
            DataVariant::Tuple { ref fields } => {
                for field in fields.iter() {
                    if field.id == self.0 { return Some(ResolvedId::FieldDef(field)) }
                    if field.ty.id == self.0 { return Some(ResolvedId::Ty(&field.ty)) }
                }
                None
            }
        }
    }
}

pub struct GetTyQuery(ASTNodeID);

impl Query for GetTyQuery {
    type Result<'cx> = Option<&'cx Ty>;

    fn run<'cx>(&self, cx: &'cx QueryContext) -> Self::Result<'cx> {
        // TODO: Implement

        None
    }
}