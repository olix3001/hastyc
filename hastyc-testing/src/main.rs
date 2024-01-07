use hastyc_common::{identifiers::{PkgID, SourceFileID}, source::SourceFile};
use hastyc_parser::lexer::Lexer;

const CODE: &str = "
struct Hello {
    bruh: i32
}

impl Hello {
    pub override(+) fn add_i32(self, other: i32) -> i32 {
        self.bruh + other
    }

    pub fn inc(self) {
        self.bruh += 1
    }
}
";

fn main() {
    let source = SourceFile::new_raw(
        CODE.to_string(),
        PkgID::new_unique(),
        SourceFileID::new_unique()
    );

    let ts = Lexer::lex(&source);

    println!("Token stream: {:?}", ts);
}
