use std::path::Path;
use rusty_judge::{build_cfg_from_root, compute_liveness, parse_source, print_liveness, read_js_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = read_js_file(Path::new("test/cfg.js"))?;
    let root = parse_source(&source)?;
    //println!("{:#?}", &root);
    let cfg = build_cfg_from_root(root)?;
    println!("{:#?}", &cfg);
    let liveness = compute_liveness(&cfg);
    print_liveness(&cfg, &liveness);
    Ok(())
}