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
    let c;
    let hello = a;
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
