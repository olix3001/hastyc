use std::collections::BTreeMap;

use hastyc_common::identifiers::{Ident, ASTNodeID};

/// Stack that holds ribs, which are modifications to the scope. These ribs are data structures
/// that can add or shadow something in the scope. Addition modifies the latest rib, while shadowing
/// pushes new one onto the stack.
#[derive(Debug, Clone)]
pub struct RibStack {
    stack: Vec<Rib>
}

/// Rib is a single modification to the scope.
#[derive(Debug, Default, Clone)]
pub struct Rib {
    /// Identifiers created in this rib
    pub created_ident: BTreeMap<Ident, ASTNodeID>,
}

impl RibStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self) { self.stack.push(Rib::default()) }
    pub fn pop(&mut self) -> Option<Rib> { self.stack.pop() }

    /// Get last from the stack creating new if there is none
    fn get_last(&mut self) -> &mut Rib {
        if self.stack.len() == 0 {
            self.stack.push(Rib::default())
        }
        self.stack.last_mut().unwrap()
    }

    /// Get ident mapping looking at the stack top to bottom
    pub fn get_ident(&self, ident: &Ident) -> Option<&ASTNodeID> {
        for elem in self.stack.iter() {
            if let Some(node) = elem.try_get_ident_mapping(ident) {
                return Some(node)
            }
        }
        None
    }

    pub fn add_ident_mapping(&mut self, ident: Ident, def_node: ASTNodeID) {
        if let Some(_node) = self.get_ident(&ident) {
            self.push();
        }
        self.get_last().ident_mapping(ident, def_node)
    }
}

impl Rib {
    pub fn ident_mapping(&mut self, ident: Ident, def_node: ASTNodeID) {
        self.created_ident.insert(ident, def_node);
    }

    pub fn try_get_ident_mapping(&self, ident: &Ident) -> Option<&ASTNodeID> {
        self.created_ident.get(&ident)
    }
}