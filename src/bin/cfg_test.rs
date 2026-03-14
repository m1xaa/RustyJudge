use std::path::Path;
use rusty_judge::{build_cfg_from_root, parse_source, read_js_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = read_js_file(Path::new("test/cfg.js"))?;
    let root = parse_source(&source)?;
    let cfg = build_cfg_from_root(root);
    println!("{:#?}", &cfg);
    Ok(())
}