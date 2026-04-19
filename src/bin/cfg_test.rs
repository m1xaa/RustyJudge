use std::path::Path;
use rusty_judge::{
    build_semantic_model_from_root,
    parse_source,
    print_liveness,
    print_statement_liveness,
    read_js_file,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = read_js_file(Path::new("test/cfg.js"))?;
    let root = parse_source(&source)?;
    let semantic_model = build_semantic_model_from_root(root)?;

    println!("{:#?}", semantic_model.script_cfg());
    println!("{:#?}", semantic_model.symbols);
    print_liveness(semantic_model.script_cfg(), semantic_model.script_liveness());
    print_statement_liveness(semantic_model.script_cfg(), semantic_model.script_liveness());

    for (idx, function) in semantic_model.functions.iter().enumerate() {
        println!("function cfg {idx}");
        println!("name: {:?}", function.name);
        println!("symbol_id: {:?}", function.symbol_id);
        println!("{:#?}", function.body.cfg);
        print_liveness(&function.body.cfg, &function.body.liveness);
        print_statement_liveness(&function.body.cfg, &function.body.liveness);
    }

    Ok(())
}