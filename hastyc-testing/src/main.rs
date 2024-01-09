use hastyc_ast_fmt::PackageASTPrettyPrinter;
use hastyc_common::{identifiers::{PkgID, SourceFileID}, source::SourceFile};
use hastyc_parser::{lexer::Lexer, parser::Parser};

const CODE: &str = "
import hello::world::{bruh, lool::self};
import hello::world::{lorem::ipsum, dolor::{sit, amet::self}, self};
";

fn main() {
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
