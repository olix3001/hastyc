use hastyc_ast_fmt::PackageASTPrettyPrinter;
use hastyc_common::{identifiers::{PkgID, SourceFileID}, source::SourceFile};
use hastyc_parser::{lexer::Lexer, parser::Parser};

const CODE: &str = "
#[test_attribute]
#[yoooo]
import hello::world;

pub fn hello_world(self, hello_world: hello::world::this_is::MyType) -> () {
    import inline::test;
    let a;
    fn b() {
        let x;
    }
    let typed: i32 = 1;
    let c = 5.method;
    let hello = a.b.c_d;
    let bruh = -4;
    let binary_bruh = -bruh + (1 - 2);
    let comparison: bool = bruh >= binary_bruh;
    let function_call = hello_world(1, 2, bruh, 3 + 1);
    let method_call = object.method();

    {
        let a = 1;
    }

    if bruh == -4 {
        let conditional = 1;
        conditional
    } else {
        1 + 2
    }
}
";

fn main() {
    env_logger::init();

    let source = SourceFile::new_raw(
        CODE.to_string(),
        PkgID::new_unique(),
        SourceFileID::new_unique()
    );

    let ts = Lexer::lex(&source).unwrap();
    let package = Parser::parse_from_root(&source, &ts);

    if let Err(err) = package {
        panic!("{:?}", err);
    }

    println!("AST: {}", 
        PackageASTPrettyPrinter::pretty_print(&package.unwrap())
    );
}
