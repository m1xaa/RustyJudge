use std::error::Error;
use crate::{default_rules, parse_cli, parse_js_file, Rule, RuleContext};
use crate::diagnostics::Diagnostic;

pub struct Linter {
    rules: Vec<Box<dyn Rule>>,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    pub fn with_rules(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }
    
    pub fn add_rule<R: Rule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }
    
    pub fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for rule in &self.rules {
            diagnostics.extend(rule.check(ctx));
        }

        diagnostics
    }
}

pub fn run_cli() -> Result<(), Box<dyn Error>> {
    let cli_commands = parse_cli()?;
    let linter = Linter::with_rules(default_rules());
    cli_commands.commands()
        .iter()
        .for_each(|command| {
            let (ast, source) = parse_js_file(&command.file_path).expect("Failed to parse file");
            let context = RuleContext::new(command.file_path.display().to_string(), ast, source, cli_commands.overridden_rules.max_line_length);
            println!("{}", &context);
            linter.run(&context).iter().for_each(|d| println!("{}", d));
            println!();
            println!();
        }
        );

    Ok(())
}