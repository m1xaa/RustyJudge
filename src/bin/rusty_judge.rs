use rusty_judge::{parse_cli, parse_js_file, RuleContext, NoVar, Rule, NoUnusedVars};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_commands = parse_cli()?;
    cli_commands.commands()
        .iter()
        .for_each(|command| {
            let (ast, source) = parse_js_file(&command.file_path).expect("Failed to parse file");
            let context = RuleContext::new(ast, source);
            println!("{:#?}", &context.root);
            println!("No var: {}" ,NoVar.check(&context));
            println!("No unused vars: {}", NoUnusedVars.check(&context));
        }
        );

    Ok(())
}
