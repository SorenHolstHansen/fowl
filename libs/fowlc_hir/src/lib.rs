use fowlc_lexer::Lexer;
use fowlc_package_manager::Package;
use fowlc_parser::{Parser, print_with_source};
use walkdir::WalkDir;

pub fn lower_to_hir(package: Package) {
    let src_path = package.path.join("src");
    for entry in WalkDir::new(src_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let src_main = std::fs::read_to_string(path).unwrap();
            let lexer = Lexer::new(&src_main, path);
            let parser = Parser::new(lexer);
            let tree = parser.parse();
            let mut s = Vec::new();
            print_with_source(&mut s, &tree, &src_main).unwrap();
            let s = String::from_utf8(s).unwrap();
            eprintln!("{s}");
        }
    }
}
