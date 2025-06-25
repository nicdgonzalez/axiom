use std::io::{BufRead, Read, Seek, Write};

use anyhow::Context;

use super::{TMUX_SERVER_NAME, TMUX_SESSION_NAME};

#[derive(Debug, Clone, clap::Args)]
pub struct Stop {}

impl crate::commands::Run for Stop {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<(), crate::error::Error> {
        let package = ctx
            .package()
            .with_context(|| "failed to get package manifest")?;

        // Read the `latest.log` file to determine if the server closed properly.
        let latest_log = package.server().logs().join("latest.log");
        let file = std::fs::File::open(&latest_log).with_context(|| "failed to open latest.log")?;
        let mut reader = std::io::BufReader::new(file);
        let mut position = 0;
        // Position the cursor at the end of the file before stopping the server so we are as close
        // as possible to the "Stopping server" message.
        reader
            .seek(std::io::SeekFrom::End(0))
            .with_context(|| "failed to seek to end of file")?;

        // Send CTRL+C into the target server's pane.
        //
        // There were 2 alternatives I also considered:
        // - Send "stop" and "Enter"
        // - Send SIGTERM to the process directly.
        //
        // Sending "stop" assumes that there is no other command currently being typed into the console.
        // If there is a command being typed, we have to clear it (or give up and return an error,
        // as they could be actively typing while we are trying to close).
        //
        // I think ideally we would send SIGTERM to the process directly, but we would need a reliable
        // way to get the process ID for the pane.
        //
        // I think sending CTRL+C is the fastest and simplest solution we can implement right now.
        let status = std::process::Command::new("tmux")
            .args([
                "-L",
                TMUX_SERVER_NAME,
                "send-keys",
                "-t",
                &format!("={}:{}", TMUX_SESSION_NAME, package.name()),
                "C-c",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .with_context(|| "failed to execute command 'tmux'")?;

        if !status.success() {
            crate::bail!("failed to send Ctrl+C (SIGTERM) to tmux window");
        }

        // TODO: Maybe it would be better to have a command that pipes the output of
        // the `latest.log` file into `less` and suggest running that command instead?
        let hint = format!(
            "Run `cat {} | tail -n 50` to read the error logs",
            latest_log.display()
        );

        let mut stderr = std::io::stderr().lock();
        for attempt in 0..12 {
            tracing::debug!("Checking server status: attempt #{}", attempt + 1);

            reader
                .seek(std::io::SeekFrom::Start(position))
                .with_context(|| "failed to seek to end of the file")?;
            let mut lines = Vec::new();

            for line in reader.by_ref().lines() {
                let line = line.with_context(|| "failed to read line")?;
                lines.push(line);
            }

            for line in lines {
                tracing::debug!("Reading line: {}", line);

                if line.ends_with(r#"Stopping server"#) {
                    // TODO: Provide better output:
                    //
                    // Stopping the Minecraft server...
                    // ----------------------------------------------------------------------------
                    // Uptime: 2h 15m
                    // Players joined: 7
                    // Most concurrent players: 2
                    // ----------------------------------------------------------------------------
                    // Server has been stopped.
                    writeln!(stderr, "🔴 server has been stopped").ok();
                    return Ok(());
                } else {
                    position = reader
                        .stream_position()
                        .with_context(|| "failed to get cursor position")?;
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(3));
        }

        // Failed to stop the server / determine if it is stopped.
        let message = "Axiom timed out while waiting for the server to stop".to_owned();
        Err(crate::error::Error::new_with_hint(
            anyhow::anyhow!(message),
            hint,
        ))
    }
}
