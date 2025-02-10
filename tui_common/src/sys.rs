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
    // Use AppleScript to open a new Terminal window and run a command
    let applescript = format!(
        r#"tell application "Terminal"
    do script "{}"
end tell"#,
        command
    );

    Command::new("osascript")
        .args(&["-e", &applescript])
        .spawn()?;
    Ok(())
}
