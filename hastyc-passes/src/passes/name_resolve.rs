use std::collections::BTreeMap;

use hastyc_common::{identifiers::{ASTNodeID, Ident}, path::Path, error::{ErrorDisplay, CommonErrorContext}};
use hastyc_parser::parser::{DataVariant, ExprKind, ItemKind, LetBindingKind, StmtKind, TyKind};

use crate::util::RibStack;

use super::{ASTPass, QueryContext};

#[derive(Debug)]
pub struct NameResolvePass {
    stack: RibStack,
    subpasses: BTreeMap<ASTNodeID, NameResolvePass>,
}

impl NameResolvePass {
    pub fn new() -> Self {
        Self {
            stack: RibStack::new(),
            subpasses: BTreeMap::new()
        }
    }

    pub fn resolve_ident(&self, ident: Ident) -> Option<&ASTNodeID> {
        return self.stack.get_ident(&ident);
    }

    pub fn resolve_path(&self, path: &Path) -> Result<&ASTNodeID, NameResolveError> {
        let mut segments = path.segments.iter();
        let mut c = 0;
        let mut sub = self;
        while let Some(ref seg) = segments.next() {
            let seg = sub.resolve_ident(seg.ident.clone());
            if seg.is_none() { return Err(
                NameResolveError::UnknownPath {
                    path: path.clone(),
                    start_idx: c
                }
            ); }
            c += 1;
            if let Some(ref subsub) = sub.subpasses.get(seg.unwrap()) {
                sub = subsub;
            } else { return Ok(seg.unwrap()); }
        }
        Err(NameResolveError::UnknownPath {
            path: path.clone(),
            start_idx: 0
        })
    }

    fn resolve_ty(
        &mut self, ty: &hastyc_parser::parser::Ty,
    ) -> Result<Option<&ASTNodeID>, NameResolveError> {
        match ty.kind {
            TyKind::Path(ref path) => Ok(Some(self.resolve_path(path)?)),
            TyKind::SelfTy => unimplemented!("Name resolution for Self type is not implemented"),
            _ => { Ok(None) }
        }
    }

    fn visit_datavariant(
        &mut self,
        dv: &DataVariant,
        cx: &mut QueryContext,
        item_id: ASTNodeID
    ) -> Result<(), NameResolveError> {
        match dv {
            DataVariant::Unit => { },
            DataVariant::Struct { ref fields } => {
                let mut subpass = NameResolvePass::new();
                for field in fields.iter() {
                    if let Some(rty) = self.resolve_ty(&field.ty)? {
                        let rty = *rty;
                        cx.resolved_names.insert(field.id, rty);
                    }
                    subpass.stack.add_ident_mapping(field.ident.as_ref().unwrap().clone(), field.id);
                }
                self.subpasses.insert(item_id, subpass);
            },
            DataVariant::Tuple { ref fields } => {
                let mut subpass = NameResolvePass::new();
                unimplemented!("Tuple struct variant is not yet supported")
            }
        }
        Ok(())
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
            ItemKind::Module(ref module) => {
                if !self.subpasses.contains_key(&item.id) {
                    let mut subpass = NameResolvePass::new();
                    subpass.traverse_itemstream(module, ctx)?;
                    self.subpasses.insert(item.id, subpass);
                }
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
                self.traverse_stmtstream(&function.body.as_ref().unwrap().stmts, ctx)?;
            }
            ItemKind::Import(ref kind, ref tree) => {
                unimplemented!("Name resolution for imports is not yet implemented");
            },
            ItemKind::Struct(ref datavar) => {
                self.visit_datavariant(datavar, ctx, item.id)?;
            },
            _ => todo!()
        }
        Ok(())
    }

    fn visit_stmt(
        &mut self,
        stmt: &hastyc_parser::parser::Stmt,
        cx: &mut super::QueryContext
    ) -> Result<(), NameResolveError> {
        match stmt.kind {
            StmtKind::LetBinding(ref binding) => {
                if let Some(ident) = binding.pat.ident() {
                    self.stack.add_ident_mapping(ident.clone(), binding.id);
                }
                if let Some(ref ty) = binding.ty {
                    if let Some(ty_resolved) = self.resolve_ty(ty)? {
                        let ty_resolved = *ty_resolved;
                        cx.resolved_names.insert(binding.id, ty_resolved);
                    }
                }
                if let LetBindingKind::Init(ref expr) = binding.kind {
                    self.visit_expr(expr, cx)?;
                }
            }
            StmtKind::Expr(ref expr) => {
                self.visit_expr(expr, cx)?;
            }
            StmtKind::ExprNS(ref expr) => {
                self.visit_expr(expr, cx)?;
            }
            StmtKind::Item(ref item) => {
                self.visit_item(&item, cx)?;
            }
        }
        Ok(())
    }

    fn visit_expr(
        &mut self,
        expr: &hastyc_parser::parser::Expr,
        cx: &mut super::QueryContext
    ) -> Result<(), NameResolveError> {
        match expr.kind {
            ExprKind::Path(ref path) => {
                let target = self.resolve_path(path)?;
                cx.resolved_names.insert(expr.id, *target);
            }
            ExprKind::Field(ref subexpr, ref ident) => {
                self.visit_expr(&subexpr, cx)?;
            }
            _ => todo!()
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