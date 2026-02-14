use super::*;
use std::path::PathBuf;
use crate::cli::command::CliCommands;
use crate::cli::error::CliError;
use std::fs::File;

#[test]
fn new_cli_commands_has_default_values() {
    let cli = CliCommands::new();

    assert_eq!(cli.commands().len(), 0);
    assert_eq!(cli.overridden_rules.max_line_length, 120);
}

#[test]
fn add_command_adds_file_path() {
    let mut cli = CliCommands::new();
    cli.add_command("test.js");

    assert_eq!(cli.commands().len(), 1);
    assert_eq!(
        cli.commands()[0].file_path,
        PathBuf::from("test.js")
    );
}

#[test]
fn override_max_line_length_changes_value() {
    let mut cli = CliCommands::new();
    cli.override_max_line_length(200);

    assert_eq!(cli.overridden_rules.max_line_length, 200);
}

#[test]
fn parses_max_line_length_flag() {
    let args = vec![
        "--max-line-length".to_string(),
        "150".to_string(),
    ];

    let mut cli = CliCommands::new();
    get_file_paths(&args, &mut cli).unwrap();

    assert_eq!(cli.overridden_rules.max_line_length, 150);
}

#[test]
fn returns_error_for_unknown_flag() {
    let args = vec!["--unknown".to_string()];
    let mut cli = CliCommands::new();

    let result = super::get_file_paths(&args, &mut cli);

    assert!(matches!(result, Err(CliError::InvalidFlag(_))));
}

#[test]
fn returns_error_for_invalid_flag_value() {
    let args = vec![
        "--max-line-length".to_string(),
        "abc".to_string(),
    ];

    let mut cli = CliCommands::new();
    let result = super::get_file_paths(&args, &mut cli);

    assert!(matches!(result, Err(CliError::InvalidFlagValue(_))));
}

#[test]
fn adds_single_js_file() {
    let file_name = "test_temp_file.js";
    File::create(file_name).unwrap();

    let args = vec![file_name.to_string()];
    let mut cli = CliCommands::new();

    get_file_paths(&args, &mut cli).unwrap();
    assert_eq!(cli.commands().len(), 1);

    std::fs::remove_file(file_name).unwrap();
}

#[test]
fn rejects_non_js_file() {
    let file_name = "test_temp_file.txt";
    File::create(file_name).unwrap();

    let args = vec![file_name.to_string()];
    let mut cli = CliCommands::new();

    let result = super::get_file_paths(&args, &mut cli);

    assert!(matches!(result, Err(CliError::InvalidFlag(_))));

    std::fs::remove_file(file_name).unwrap();
}



