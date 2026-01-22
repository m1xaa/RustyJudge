use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct CliCommands {
    commands: Vec<Command>,
    pub overridden_rules: RuleOverrides
}

impl CliCommands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            overridden_rules: RuleOverrides::default(),
        }
    }

    pub fn add_command<P: AsRef<Path>>(&mut self, path: P) {
        self.commands.push(Command {
            file_path: path.as_ref().to_path_buf(),
        });
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
    
    pub fn override_max_line_length(&mut self, max_length: usize) {
        self.overridden_rules.max_line_length = max_length;
    }
}

#[derive(Debug)]
pub struct Command {
    pub file_path: PathBuf
}

#[derive(Debug)]
pub struct RuleOverrides {
    pub max_line_length: usize,
}

impl Default for RuleOverrides {
    fn default() -> Self {
        Self {
            max_line_length: 120,
        }
    }
}
