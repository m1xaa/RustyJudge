
use rusty_judge::{parse_cli, parse_js_file, RuleContext, NoVar, Rule, StrictEquality, NoConsoleLog};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_commands = parse_cli()?;
    cli_commands.commands()
        .iter()
        .for_each(|command| {
            let (ast, source) = parse_js_file(&command.file_path).expect("Failed to parse file");
            let context = RuleContext::new(ast, source, cli_commands.overridden_rules.max_line_length);
            println!("{:#?}", &context.root);
            println!("no empty block: {}" , rusty_judge::NoEmptyBlock.check(&context));
        }
        );

    Ok(())
}
