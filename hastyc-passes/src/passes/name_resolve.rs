use std::collections::{HashMap, BTreeMap};

use hastyc_common::{identifiers::{ASTNodeID, Ident}, path::Path};
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

    pub fn resolve_path(&self, path: &Path) -> Option<&ASTNodeID> {
        let mut segments = path.segments.iter();
        let mut seg = self.resolve_ident(segments.next().unwrap().ident.clone());
        #[allow(unused_assignments)]
        let mut sub = self;
        if seg.is_none() { return None }
        while let Some(ref subseg) = segments.next() {
            if let Some(ref subsub) = self.subpasses.get(seg.unwrap()) {
                sub = subsub
            } else { break; }
            seg = sub.resolve_ident(subseg.ident.clone());
        }
        seg
    }
}

impl<'ctx> ASTPass<'ctx> for NameResolvePass {
    fn traverse_itemstream(
        &mut self,
        stream: &hastyc_parser::parser::ItemStream,
        ctx: &mut super::QueryContext
    ) {
        // Register all item names
        for item in stream.items.iter() {
            self.stack.add_ident_mapping(item.ident.clone(), item.id);

            if let ItemKind::Module(ref module) = item.kind {
                let mut subpass = NameResolvePass::new();
                subpass.traverse_itemstream(module, ctx);
                self.subpasses.insert(item.id, subpass);
            }
        }
        self.stack.push();

        // Visit all items
        for item in stream.items.iter() {
            self.visit_item(item, ctx)
        }
        self.stack.pop();
    }

    fn traverse_stmtstream(
        &mut self,
        stream: &hastyc_parser::parser::StmtStream,
        ctx: &mut super::QueryContext
    ) {
        self.stack.push();
        for stmt in stream.stmts.iter() {
            self.visit_stmt(stmt, ctx);
        }
        self.stack.pop();
    }

    fn visit_item(
        &mut self,
        item: &hastyc_parser::parser::Item, 
        ctx: &mut super::QueryContext
    ) {
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
                self.traverse_stmtstream(&function.body.as_ref().unwrap().stmts, ctx);
            }
            _ => { println!("UNIMPLEMENTED NAME RESOLVE ITEM KIND") }
        }
    }

    fn visit_stmt(
        &mut self,
        stmt: &hastyc_parser::parser::Stmt,
        ctx: &mut super::QueryContext
    ) {
        match stmt.kind {
            StmtKind::LetBinding(ref binding) => {
                if let Some(ident) = binding.pat.ident() {
                    self.stack.add_ident_mapping(ident.clone(), binding.id);
                }
                if let LetBindingKind::Init(ref expr) = binding.kind {
                    self.visit_expr(expr, ctx);
                }
            }
            StmtKind::Expr(ref expr) => {
                self.visit_expr(expr, ctx);
            }
            StmtKind::ExprNS(ref expr) => {
                self.visit_expr(expr, ctx);
            }
            _ => { println!("UNIMPLEMENTED NAME RESOLVE STMT KIND") }
        }
    }

    fn visit_expr(
        &mut self,
        expr: &hastyc_parser::parser::Expr,
        ctx: &mut super::QueryContext
    ) {
        match expr.kind {
            ExprKind::Path(ref path) => {
                let target = self.resolve_path(path);
                self.resolved.insert(expr.id, *target.unwrap());
            }
            _ => { println!("UNIMPLEMENTED NAME RESOLVE EXPR KIND") }
        }
    }
}