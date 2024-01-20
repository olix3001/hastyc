use std::collections::BTreeMap;

use hastyc_common::{identifiers::{ASTNodeID, Ident}, path::Path, error::{ErrorDisplay, CommonErrorContext}};
use hastyc_parser::parser::{ItemKind, StmtKind, LetBindingKind, ExprKind};

use crate::util::RibStack;

use super::ASTPass;

#[derive(Debug)]
pub struct NameResolvePass {
    stack: RibStack,
    resolved: BTreeMap<ASTNodeID, ASTNodeID>,
    subpasses: BTreeMap<ASTNodeID, NameResolvePass>,
}

impl NameResolvePass {
    pub fn new() -> Self {
        Self {
            stack: RibStack::new(),
            resolved: BTreeMap::new(),
            subpasses: BTreeMap::new()
        }
    }

    pub fn resolve_ident(&self, ident: Ident) -> Option<&ASTNodeID> {
        return self.stack.get_ident(&ident);
    }

    pub fn resolve_path(&self, path: &Path) -> Result<&ASTNodeID, u32> {
        let mut segments = path.segments.iter();
        let mut c = 0;
        let mut sub = self;
        while let Some(ref seg) = segments.next() {
            let seg = sub.resolve_ident(seg.ident.clone());
            if seg.is_none() { return Err(c); }
            c += 1;
            if let Some(ref subsub) = sub.subpasses.get(seg.unwrap()) {
                sub = subsub;
            } else { return Ok(seg.unwrap()); }
        }
        Err(0)
    }
}

pub enum NameResolveError {
    UnknownPath {
        path: Path,
        start_idx: u32
    }
}

impl<'ctx> ASTPass<'ctx> for NameResolvePass {
    type Err = NameResolveError;

    fn traverse_itemstream(
        &mut self,
        stream: &hastyc_parser::parser::ItemStream,
        ctx: &mut super::QueryContext
    ) -> Result<(), NameResolveError> {
        // Register all item names
        for item in stream.items.iter() {
            self.stack.add_ident_mapping(item.ident.clone(), item.id);

            if let ItemKind::Module(ref module) = item.kind {
                let mut subpass = NameResolvePass::new();
                subpass.traverse_itemstream(module, ctx)?;
                self.subpasses.insert(item.id, subpass);
            }
        }
        self.stack.push();

        // Visit all items
        for item in stream.items.iter() {
            self.visit_item(item, ctx)?;
        }
        self.stack.pop();
        Ok(())
    }

    fn traverse_stmtstream(
        &mut self,
        stream: &hastyc_parser::parser::StmtStream,
        ctx: &mut super::QueryContext
    ) -> Result<(), NameResolveError> {
        self.stack.push();
        for stmt in stream.stmts.iter() {
            self.visit_stmt(stmt, ctx)?;
        }
        self.stack.pop();
        Ok(())
    }

    fn visit_item(
        &mut self,
        item: &hastyc_parser::parser::Item, 
        ctx: &mut super::QueryContext
    ) -> Result<(), NameResolveError> {
        match item.kind {
            ItemKind::Module(ref _module) => {}
            ItemKind::Fn(ref function) => {
                // TODO: Generics
                // Go to signature
                for input in function.signature.inputs.iter() {
                    // Register input as variable
                    if let Some(ident) = input.pat.ident() {
                        self.stack.add_ident_mapping(ident.clone(), input.id);
                    }
                }

                // Go to body
                self.traverse_stmtstream(&function.body.as_ref().unwrap().stmts, ctx)?;
            }
            _ => { println!("UNIMPLEMENTED NAME RESOLVE ITEM KIND") }
        }
        Ok(())
    }

    fn visit_stmt(
        &mut self,
        stmt: &hastyc_parser::parser::Stmt,
        ctx: &mut super::QueryContext
    ) -> Result<(), NameResolveError> {
        match stmt.kind {
            StmtKind::LetBinding(ref binding) => {
                if let Some(ident) = binding.pat.ident() {
                    self.stack.add_ident_mapping(ident.clone(), binding.id);
                }
                if let LetBindingKind::Init(ref expr) = binding.kind {
                    self.visit_expr(expr, ctx)?;
                }
            }
            StmtKind::Expr(ref expr) => {
                self.visit_expr(expr, ctx)?;
            }
            StmtKind::ExprNS(ref expr) => {
                self.visit_expr(expr, ctx)?;
            }
            _ => { println!("UNIMPLEMENTED NAME RESOLVE STMT KIND") }
        }
        Ok(())
    }

    fn visit_expr(
        &mut self,
        expr: &hastyc_parser::parser::Expr,
        _ctx: &mut super::QueryContext
    ) -> Result<(), NameResolveError> {
        match expr.kind {
            ExprKind::Path(ref path) => {
                let target = self.resolve_path(path);
                match target {
                    Ok(target) => self.resolved.insert(expr.id, *target),
                    Err(idx) => Err(NameResolveError::UnknownPath {
                        path: path.clone(), start_idx: idx 
                    })?
                };
            }
            _ => { println!("UNIMPLEMENTED NAME RESOLVE EXPR KIND") }
        }
        Ok(())
    }
}

impl<'ctx> ErrorDisplay<'ctx, CommonErrorContext<'ctx>> for NameResolveError {
    fn fmt(&self, fmt: &mut hastyc_common::error::ErrorFmt<'ctx>, ctx: &'ctx CommonErrorContext) {
        match self {
            NameResolveError::UnknownPath { ref path, ref start_idx } => {
                fmt
                    .title("Path could not be resolved.")
                    .source(ctx.source, path.shifted_clone(*start_idx).span)
                    .cause("This path could not have been resolved.")
                    .help("Ensure that this path is spelled correctly and that there are items with these names.");
            }
        }
    }
}