use std::error::Error;
use crate::cli::parse_cli;
use crate::diagnostics::Diagnostic;
use crate::linter::file_report::FileReport;
use crate::parser::parse_js_file;
use crate::rules::{default_rules, Rule, RuleContext};

pub struct Linter {
    rules: Vec<Box<dyn Rule>>,
}

impl Linter {
    pub fn with_rules(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
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

    let mut reports: Vec<FileReport> = Vec::new();

    for command in cli_commands.commands().iter() {
        let (ast, source) = parse_js_file(&command.file_path)?;

        let context = RuleContext::new(
            command.file_path.display().to_string(),
            ast,
            source,
            cli_commands.overridden_rules.max_line_length,
        );

        let diagnostics = linter.run(&context);

        reports.push(FileReport {
            file: command.file_path.display().to_string(),
            diagnostics,
        });
    }

    println!("{}", serde_json::to_string_pretty(&reports)?);


    Ok(())
}
