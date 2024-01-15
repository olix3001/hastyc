use hastyc_common::{identifiers::{Ident, Symbol}, path::Path};
use hastyc_parser::parser::{Package, Item, ItemKind, ItemStream, ImportTree, ImportTreeKind, Attributes, AttributeKind, FnSignature, Pat, PatKind, Ty, TyKind, FnRetTy, Block, Stmt, StmtKind, LetBindingKind, Expr, ExprKind, Lit, LitKind};

pub struct PackageASTPrettyPrinter<'pkg> {
    result: String,
    indent: usize,
    pkg: &'pkg Package
}

impl<'pkg> PackageASTPrettyPrinter<'pkg> {
    fn subformatter(&self) -> Self {
        Self {
            result: String::new(),
            indent: self.indent,
            pkg: self.pkg
        }
    }

    fn into_text(self) -> String {
        self.result
    }

    fn pushi(&mut self) {
        self.indent += 1;
    }
    fn popi(&mut self) {
        self.indent -= 1;
    }

    fn push_line(&mut self, text: &str) {
        self.result.push_str(&format!(
            "{}{}\n",
            "    ".repeat(self.indent),
            text
        ));
    }

    fn ident(&self, ident: &Ident) -> &str {
        self.pkg.symbol_storage.text_of(ident.symbol).unwrap()
    }
    fn symbol(&self, symbol: &Symbol) -> &str {
        self.pkg.symbol_storage.text_of(symbol.clone()).unwrap()
    }

    pub fn pretty_print(package: &'pkg Package) -> String {
        let mut printer = Self {
            pkg: package,
            indent: 0,
            result: String::new()
        };

        printer.push_line("Package: ");
        printer.pushi();
        printer.item_stream(&printer.pkg.items);

        printer.result
    }

    fn item_stream(&mut self, item_stream: &ItemStream) {
        for item in item_stream.items.iter() {
            self.item(item)        
        }
    }

    fn attributes(&mut self, attributes: &Attributes) {
        for attr in attributes.attributes.iter() {
            match attr.kind {
                AttributeKind::FlagAttribute => 
                    self.push_line(&format!("#[{}]", self.ident(&attr.ident)))
            }
        }
    }

    fn item(&mut self, item: &Item) {
        self.attributes(&item.attrs);
        match item.kind {
            ItemKind::Module(ref is) => {
                self.push_line(&format!("Module \"{}\":", self.ident(&item.ident)));
                self.pushi();
                self.item_stream(is);
                self.popi();
            },
            ItemKind::Import(ref kind, ref it) => {
                self.push_line(&format!("Import ({:?}):", kind));
                self.pushi();
                self.import_tree(it);
                self.popi();
            }
            ItemKind::Fn(ref function) => {
                self.push_line(&format!("Function {}:", self.ident(&item.ident)));
                self.pushi();
                self.function_signature(&function.signature);
                self.block(function.body.as_ref().unwrap());
                self.popi();
            }
        }
    }

    fn import_tree(&mut self, tree: &ImportTree) {
        self.push_line(&format!("prefix: {}", self.path(&tree.prefix)));
        match tree.kind {
            ImportTreeKind::Glob => self.push_line("Import: glob"),
            ImportTreeKind::SelfImport => self.push_line("Import: self"),
            ImportTreeKind::Simple(ref i) => self.push_line(&format!("Import: {}", self.ident(i))),
            ImportTreeKind::Nested(ref subtries) => {
                self.push_line("Nested: [");
                self.pushi();
                for subtree in subtries.iter() {
                    self.import_tree(&subtree.0);
                }
                self.popi();
                self.push_line("]")
            }
        }
    }

    fn path(&self, path: &Path) -> String {
        let mut txt = String::new();
        for segment in path.segments.iter() {
            txt.push_str(self.ident(&segment.ident));
            txt.push_str("::");
        }

        txt.pop();txt.pop(); // remove last '::'
        txt
    }

    fn function_signature(&mut self, sig: &FnSignature) {
        let mut string = String::new();

        if sig.is_const { string.push_str("const ")}
        if sig.is_async { string.push_str("async ")}
        
        string.push_str("fn(");

        for arg in sig.inputs.iter() {
            string.push_str(&self.pat(&arg.pat));
            string.push_str(": ");
            string.push_str(&self.ty(&arg.ty));

            string.push_str(", ");
        }
        if sig.inputs.len() > 0 {
            string.pop();
            string.pop();
        }
        string.push(')');

        string.push_str(" -> ");

        let output = match sig.output {
            FnRetTy::Default => "default".to_string(),
            FnRetTy::Ty(ref ty) => self.ty(ty).to_string()
        };
        string.push_str(&output);

        self.push_line(&string);
    }

    fn pat(&self, pat: &Pat) -> String {
        match pat.kind {
            PatKind::SelfPat => "self".to_string(),
            PatKind::Ident(ref ident) => self.ident(ident).to_string()
        }
    }

    fn ty(&self, ty: &Ty) -> String {
        match ty.kind {
            TyKind::SelfTy => "self".to_string(),
            TyKind::Void => "void".to_string(),
            TyKind::Never => "never".to_string(),
            TyKind::Path(ref path) => self.path(path),
            TyKind::Infer => "<infer>".to_string()
        }
    }

    fn block(&mut self, block: &Block) {
        self.push_line("{");
        self.pushi();

        for ref stmt in block.stmts.stmts.iter() {
            self.stmt(stmt);
        }

        self.popi();
        self.push_line("}");
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt.kind {
            StmtKind::LetBinding(ref let_binding) => {
                match let_binding.kind {
                    LetBindingKind::Decl => self.push_line(&format!(
                        "let {}: {};",
                        self.pat(&let_binding.pat),
                        self.ty(let_binding.ty.as_ref().unwrap())
                    )),
                    LetBindingKind::Init(ref init) => self.push_line(&format!(
                        "let {}: {} = {};",
                        self.pat(&let_binding.pat),
                        self.ty(let_binding.ty.as_ref().unwrap()),
                        self.expr(&init)
                    ))
                }
            },
            StmtKind::Item(ref item) => {
                self.item(item)
            },
            StmtKind::Expr(ref expr) => self.push_line(&format!("{};", self.expr(expr))),
            StmtKind::ExprNS(ref expr) => self.push_line(&self.expr(expr)),
        }
    }

    pub fn expr(&self, expr: &Expr) -> String {
        match expr.kind {
            ExprKind::Path(ref path) => format!("Path({})", self.path(path)),
            ExprKind::Literal(ref lit) => self.lit(lit),
            ExprKind::Field(ref expr, ref field) => format!("{}.{}", self.expr(expr), self.ident(field)),
            ExprKind::Unary(ref unop, ref expr) => format!("Unary<{:?}>({})", unop, self.expr(expr)),
            ExprKind::Binary(ref binop, ref expr1, ref expr2) =>
                format!("Binary<{:?}>({}; {})", binop.kind, self.expr(expr1), self.expr(expr2)),
            ExprKind::Call(ref target, ref args) =>
                format!("Call<{}>({})", self.expr(target), args.iter().map(|a| self.expr(a)).collect::<Vec<String>>().join(", ")),
            ExprKind::Block(ref block) => { let mut sf = self.subformatter(); sf.block(block); format!("\n{}\n", sf.into_text()) },
            ExprKind::If(ref condition, ref block, ref else_expr) =>
                {
                    let mut if_block = self.subformatter();
                    if_block.block(block);
                    let if_block = if_block.into_text();
                    if else_expr.is_some() {
                        format!("if ({})\n{}{}else {}",
                        self.expr(condition),
                        if_block,
                        "    ".repeat(self.indent),
                        self.expr(else_expr.as_ref().unwrap()))
                    } else {
                        format!("if ({}) {}", self.expr(condition), if_block)
                    }
                }
        }
    }

    pub fn lit(&self, lit: &Lit) -> String {
        let mut string = String::from("Lit<");
        string.push_str(match lit.kind {
            LitKind::Bool => "bool",
            LitKind::Char => "char",
            LitKind::Float => "float",
            LitKind::Integer => "int",
            LitKind::String => "str"
        });
        string.push_str(">(");
        string.push_str(self.symbol(&lit.symbol));
        string.push(')');
        string
    }
}