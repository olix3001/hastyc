use std::collections::{HashMap, BTreeMap};

use hastyc_common::identifiers::ASTNodeID;
use hastyc_parser::parser::{ItemKind, StmtKind, LetBindingKind, ExprKind};

use crate::util::RibStack;

use super::ASTPass;

pub struct NameResolvePass {
    stack: RibStack,
    resolved: BTreeMap<ASTNodeID, ASTNodeID>
}

impl NameResolvePass {
    pub fn new() -> Self {
        Self {
            stack: RibStack::new(),
            resolved: BTreeMap::new()
        }
    }

    pub fn join(&mut self, other: NameResolvePass) {
        for (k, v) in other.resolved.into_iter() {
            self.resolved.insert(k, v);
        }
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
            self.stack.add_ident_mapping(item.ident.clone(), item.id)
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
            ItemKind::Module(ref module) => { // Begin subpass
                let mut subpass = NameResolvePass::new();
                subpass.traverse_itemstream(module, ctx);
                self.join(subpass);
            }
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
                // TODO: Figure out how to resolve paths with multiple segments
            }
            _ => { println!("UNIMPLEMENTED NAME RESOLVE EXPR KIND") }
        }
    }
}