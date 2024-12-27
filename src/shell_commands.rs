use std::io;
use std::process::{Command, Output};

#[derive(Debug)]
pub struct ShellCommand {
    command: String,
    arguments: String,
}

impl ShellCommand {
    pub fn new(command: &str, arguments: &str) -> Self {
        ShellCommand {
            command: command.to_string(),
            arguments: arguments.to_string(),
        }
    }

    pub fn execute(self) -> io::Result<Output> {
        let res = self.execute_core();

        res
    }

    #[cfg(target_os = "windows")]
    fn execute_core(self) -> io::Result<Output> {
        Command::new("cmd")
            .args([
                "/C",
                &*format!("{} {}", self.command, self.arguments).to_string(),
            ])
            .output()
    }

    #[cfg(target_os = "linux")]
    fn execute_core(self) -> io::Result<Output> {
        Command::new("sh")
            .args([
                "-c",
                &*format!("{} {}", self.command, self.arguments).to_string(),
            ])
            .output()
    }
}

#[cfg(test)]
mod tests {
    use crate::shell_commands::ShellCommand;

    #[test]
    fn test_successful_command() {
        const TEST_ECHO: &str = "Hello World";

        let cmd = ShellCommand::new("echo", TEST_ECHO);
        let cmd_result = cmd.execute().expect("Command did not execute successfully");
        let cmd_output_str =
            String::from_utf8(cmd_result.stdout).expect("Could not convert std out to string");
        let cmd_output_trimmed_str = cmd_output_str.trim();

        assert_eq!(cmd_output_trimmed_str, TEST_ECHO);
    }
}
