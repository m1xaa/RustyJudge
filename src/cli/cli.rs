use std::path::PathBuf;
use std::{env, fs};
use crate::cli::command::CliCommands;
use crate::cli::error::CliError;


pub fn parse_cli() -> Result<CliCommands, CliError> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return Err(CliError::InsufficientArgs);
    }

    let mut cli_commands = CliCommands::new();

    get_file_paths(&args, &mut cli_commands)?;
    if cli_commands.commands().len() == 0 {
        return Err(CliError::InsufficientArgs);
    } 
    Ok(cli_commands)
}

pub fn get_file_paths(args: &[String], cli_commands: &mut CliCommands) -> Result<(), CliError> {
    let mut idx = 0;
    let cwd = env::current_dir()?;
    while idx < args.len() {

        let arg = &args[idx];
        if arg.starts_with("--") {
            if  arg != "--max-line-length" {
                return Err(CliError::InvalidFlag(arg.to_owned()));
            }

            let value =args
                .get(idx + 1)
                .ok_or_else(|| CliError::InvalidFlagValue(arg.to_owned()))?;

            let max_len: usize = value
                .parse::<usize>()
                .map_err(|_| CliError::InvalidFlagValue(value.to_owned()))?;

            cli_commands.override_max_line_length(max_len);
            idx += 2;
            continue;
        }

        if arg.ends_with("/*") {
            let dir_name = arg.trim_end_matches("/*");
            let dir_path = cwd.join(dir_name);

            if !dir_path.is_dir() {
                return Err(CliError::InvalidFlag(dir_name.to_owned()));
            }

            for entry in fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if is_js_file(&path) {
                    cli_commands.add_command(path);
                }
            }

            idx += 1;
            continue;
        }

        let path = cwd.join(arg);
        if !is_js_file(&path) {
            return Err(CliError::InvalidFlag(arg.to_owned()));
        }
        cli_commands.add_command(path);
        idx += 1;
    }

    Ok(())
}

fn is_js_file(path: &PathBuf) -> bool {
    path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("js")
}