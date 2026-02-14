
use rusty_judge::{parse_cli, parse_js_file, RuleContext, NoVar, Rule, StrictEquality, NoConsoleLog, Linter, default_rules};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_commands = parse_cli()?;
    let linter = Linter::with_rules(default_rules());
    cli_commands.commands()
        .iter()
        .for_each(|command| {
            let (ast, source) = parse_js_file(&command.file_path).expect("Failed to parse file");
            let context = RuleContext::new(ast, source, cli_commands.overridden_rules.max_line_length);
            //println!("{:#?}", context.root);
            println!("{:#?}" ,linter.run(&context));
        }
        );

    Ok(())
}
