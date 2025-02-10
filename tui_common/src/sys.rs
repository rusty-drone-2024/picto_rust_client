use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
pub fn open_terminal_with_command(command: &str) -> std::io::Result<()> {
    Command::new("terminator")
        .arg("-e")
        .arg(command)
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn open_terminal_with_command(command: &str) -> std::io::Result<()> {
    Command::new("open")
        .args(&["-a", "Terminal", command])
        .spawn()?;

    Ok(())
}
