use hastyc_common::{identifiers::Ident, path::Path};
use hastyc_parser::parser::{Package, Item, ItemKind, ItemStream, ImportTree, ImportTreeKind};

pub struct PackageASTPrettyPrinter<'pkg> {
    result: String,
    indent: usize,
    pkg: &'pkg Package
}

impl<'pkg> PackageASTPrettyPrinter<'pkg> {
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

    fn item(&mut self, item: &Item) {
        match item.kind {
            ItemKind::Module(ref is) => {
                self.push_line(&format!("Module \"{}\":", self.ident(&item.ident)));
                self.pushi();
                self.item_stream(is);
                self.popi();
            },
            ItemKind::Import(ref it) => {
                self.push_line("Import:");
                self.pushi();
                self.import_tree(it);
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
}