use std::process::{Command, Stdio};

pub fn open_terminal_with_command(command: &str) -> std::io::Result<()> {
    Command::new("terminator")
        .arg("-e")
        .arg(command)
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(())
}
