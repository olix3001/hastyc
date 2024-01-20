use hastyc_ast_fmt::PackageASTPrettyPrinter;
use hastyc_common::{identifiers::{PkgID, SourceFileID}, source::SourceFile, error::{ErrorDisplay, CommonErrorContext}};
use hastyc_parser::{lexer::Lexer, parser::Parser};
use hastyc_passes::passes::{QueryContext, name_resolve::NameResolvePass, ASTPass};

// const CODE: &str = "
// #[test_attribute]
// #[yoooo]
// import hello::world;

// pub fn hello_world(self, hello_world: hello::world::this_is::MyType) -> () {
//     import inline::test;
//     let a;
//     fn b() {
//         let x;
//     }
//     let typed: i32 = 1;
//     let c = 5.method;
//     let hello = a.b.c_d;
//     let bruh = -4;
//     let binary_bruh = -bruh + (1 - 2);
//     let comparison: bool = bruh >= binary_bruh;
//     let function_call = hello_world(1, 2, bruh, 3 + 1);
//     let method_call = object.method();

//     {
//         let a = 1;
//     }

//     if bruh == -4 {
//         let conditional = 1;
//         conditional
//     } else {
//         1 + 2
//     }

//     let a = 1;
//     loop {
//         a = a + 1;
//         if a > 10 {
//             break;
//         }
//     }

//     while bruh > 1 {
//         bruh = bruh + 1;
//     }

//     for test in hello.world() {
//         continue;
//     }
// }

// struct Hello;
// struct World(i32, pub f32);
// struct HelloWorld {
//     pub a: i32,
//     b: usize
// }

// enum Hello {
//     VariantA,
//     VariantB(i32, pub usize),
//     VariantC {
//         hello: isize,
//         pub world: usize
//     }
// }

// fn test() {
//     path::to::my_struct {
//         a: 1 + 1,
//         b: function.call()
//     }
// }

// // fn a(hello: i32) hello
// // let a = 1;
// ";

const CODE: &str = "
    module hello {
        module world {
            fn my_function() {}
        }
    }

    fn bruh() {
        let a = hello::world::my_function;
        let b = a;
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
        println!(
            "{}",
            err.fmt_error(&CommonErrorContext {
                source: &source
            })
        );
        return;
    }

    println!("AST: {}", 
        PackageASTPrettyPrinter::pretty_print(package.as_ref().unwrap())
    );

    let pkg = package.as_ref().unwrap();
    let mut ctx = QueryContext::for_package(pkg);
    let mut pass = NameResolvePass::new();
    if let Err(err) = pass.traverse(&mut ctx) {
        println!(
            "{}",
            err.fmt_error(&CommonErrorContext {
                source: &source
            })
        );
        return;
    }
    println!("Pass: {:?}", pass);
}
