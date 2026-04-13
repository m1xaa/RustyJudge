use std::path::Path;
use rusty_judge::{build_semantic_model_from_root, compute_liveness, parse_source, print_liveness, print_statement_liveness, read_js_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = read_js_file(Path::new("test/cfg.js"))?;
    let root = parse_source(&source)?;
    //println!("{:#?}", &root);
    let semantic_model = build_semantic_model_from_root(root)?;
    println!("{:#?}", &semantic_model.cfg);
    print_liveness(&semantic_model.cfg, &semantic_model.liveness);
    print_statement_liveness(&semantic_model.cfg, &semantic_model.liveness);
    Ok(())
}