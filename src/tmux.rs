use anyhow::anyhow;

pub fn exists(name: &str) -> anyhow::Result<bool> {
    let result = std::process::Command::new("tmux")
        .args(["has-session", "-t", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?
        .success();

    Ok(result)
}

pub fn create(name: &str, directory: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    if exists(name)? {
        return Ok(());
    }

    let mut command = std::process::Command::new("tmux");
    command
        .args(["new-session", "-d", "-s", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    if let Some(dir) = directory {
        let path = dir
            .to_str()
            .ok_or_else(|| anyhow!("expected path to be valid unicode"))?;

        command.args(["-c", path]);
    };

    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("failed to create tmux session"))
    }
}

pub fn destroy(name: &str) -> anyhow::Result<()> {
    if !exists(name)? {
        return Ok(());
    }

    let status = std::process::Command::new("tmux")
        .args(["kill-session", "-t", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("failed to destroy tmux session"))
    }
}

pub fn send_command(name: &str, command: &str) -> anyhow::Result<()> {
    if !exists(name)? {
        return Err(anyhow!("tmux session with name '{name}' does not exist"));
    }

    let status = std::process::Command::new("tmux")
        .args(["send-keys", "-t", name, command, "Enter"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("failed to send command to tmux session"))
    }
}
