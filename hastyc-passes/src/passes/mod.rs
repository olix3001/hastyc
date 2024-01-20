use std::collections::HashMap;

use hastyc_common::identifiers::ASTNodeID;
use hastyc_parser::parser::{ItemStream, Package, Item, StmtStream, Stmt, Expr};

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
    fn traverse(&mut self, ctx: &'ctx mut QueryContext) {
        self.traverse_itemstream(&ctx.package.items, ctx)
    }
    fn traverse_itemstream(&mut self, stream: &ItemStream, ctx: &mut QueryContext) {
        for item in stream.items.iter() {
            self.visit_item(item, ctx);
        }
    }
    fn traverse_stmtstream(&mut self, stream: &StmtStream, ctx: &mut QueryContext) {
        for stmt in stream.stmts.iter() {
            self.visit_stmt(stmt, ctx);
        }
    }

    fn visit_item(&mut self, item: &Item, ctx: &mut QueryContext);
    fn visit_stmt(&mut self, stmt: &Stmt, ctx: &mut QueryContext);
    fn visit_expr(&mut self, expr: &Expr, ctx: &mut QueryContext);
    fn finish(&mut self, _ctx: &mut QueryContext) {}
}

impl<'ctx> QueryContext<'ctx> {
    pub fn for_package(
        package: &'ctx Package
    ) -> Self {
        Self {
            package: &package,
            resolved_names: HashMap::new()
        }
    }
}