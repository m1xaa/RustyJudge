use std::env;
use std::path::PathBuf;

use rusty_judge::parser;

fn main() {
    let mut path: PathBuf = env::current_dir().expect("failed to get cwd");
    path.push("fajl.js");
    println!("{:?}", path);

    match parser::parse_js_file(&path) {
        Ok(parse) => {
            println!("Syntax tree:\n{:#?}", parse.syntax());
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }
}
