use std::error::Error;
use crate::cli::parse_cli;
use crate::linter;
use crate::linter::file_report::FileReport;
use crate::linter::lint_file;
use crate::parser::{parse_source, read_js_file};
use crate::rules::{default_rules, RuleContext};

pub fn run_cli() -> Result<(), Box<dyn Error>> {
    let cli_commands = parse_cli()?;
    let linter = linter::Linter::with_rules(default_rules());

    let mut reports: Vec<FileReport> = Vec::new();

    for command in cli_commands.commands().iter() {
        let source = read_js_file(&command.file_path)?;

        let diagnostics = lint_file(
            source,
            command.file_path.display().to_string(),
            cli_commands.overridden_rules.max_line_length,
            &linter
        )?;

        reports.push(FileReport {
            file: command.file_path.display().to_string(),
            diagnostics,
        });
    }
    println!("{}", serde_json::to_string_pretty(&reports)?);
    Ok(())
}