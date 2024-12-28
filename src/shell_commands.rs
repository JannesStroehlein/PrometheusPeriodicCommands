use std::io;
use std::process::Output;
use tokio::process::Command;

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

    pub async fn execute(self) -> io::Result<Output> {
        let res = self.execute_core().await;

        res
    }

    #[cfg(target_os = "windows")]
    async fn execute_core(self) -> io::Result<Output> {
        Command::new("cmd")
            .args([
                "/C",
                &*format!("{} {}", self.command, self.arguments).to_string(),
            ])
            .output()
            .await
    }

    #[cfg(target_os = "linux")]
    async fn execute_core(self) -> io::Result<Output> {
        Command::new("sh")
            .args([
                "-c",
                &*format!("{} {}", self.command, self.arguments).to_string(),
            ])
            .output()
            .await
    }
}
