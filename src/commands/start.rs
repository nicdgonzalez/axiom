use std::io::{BufRead, Read, Seek, Write};
use std::os::unix::process::ExitStatusExt;

use anyhow::Context;

use super::build::Build;
use crate::commands::Run;

#[derive(Debug, Clone, clap::Args)]
pub struct Start {
    #[clap(flatten)]
    pub cwd: crate::args::BaseDirectory,
}

impl Run for Start {
    fn run(&self) -> Result<(), anyhow::Error> {
        let directory = self.cwd.to_path_buf();
        let server = directory.join("server");

        let mut stderr = std::io::stderr().lock();

        // Run the build command before starting the server.
        #[rustfmt::skip]
        writeln!(stderr, "Applying any changes made to the configuration file...").ok();
        Build::run(&Build {
            cwd: self.cwd.clone(),
            accept_eula: false,
        })?;

        let window_name = directory
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .with_context(|| "expected path to current directory to be valid unicode")?;

        // XXX: How to handle case when we need to send a command to a window, but there are two
        // windows with the same name...? If I give servers IDs instead, where would I save them?
        //
        // If two windows have the same name, tmux returns exit code 1 and says "can't find window"
        //
        // If I use -P when creating the window, tmux prints "session:window_no.pane" instead of
        // "session.window_name.pane". We can store this somewhere to communicate with the server.
        // (Except, a separate window closing would cause the windows to be re-ordered...)
        //
        // The problem with using tmux is that the user can go in and mess up our state in between
        // commands... would it be possible to create a separate server with our sessions so it's
        // more explicit that the user is not meant to be able to touch anything without the cli?
        //
        // "axiom" server -> "servers" session -> Minecraft servers running in separate windows.
        writeln!(stderr, "Starting the server...").ok();
        tracing::info!("Creating a new window in tmux session...");
        let status = std::process::Command::new("tmux")
            .args([
                "new-window",
                "-c",
                server
                    .to_str()
                    .with_context(|| "failed to convert current directory to string")?,
                "-d",
                "-t",
                "=axiom",
                "-n",
                window_name,
                "./start.sh",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .with_context(|| "failed to execute tmux command")?;

        if !status.success() {
            tracing::info!(
                "Couldn't create window because session was missing. \
                Creating session, then window..."
            );
            let status = std::process::Command::new("tmux")
                .args([
                    "new-session",
                    "-c",
                    server
                        .to_str()
                        .with_context(|| "failed to convert current directory to string")?,
                    "-d",
                    "-s",
                    "axiom",
                    "-n",
                    window_name,
                    "./start.sh",
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .with_context(|| "failed to execute tmux command")?;

            if !status.success() {
                match status.code() {
                    Some(code) => tracing::error!("Command terminated with exit code: {code}"),
                    None => tracing::error!(
                        "Command terminated by signal: {}",
                        status.signal().unwrap()
                    ),
                }

                return Err(anyhow::anyhow!("failed to create tmux session"));
            }
        }

        let latest_log = server.join("logs").join("latest.log");

        let sleep_duration = std::time::Duration::from_secs(5);
        tracing::debug!(
            "Sleeping for {:?} seconds to give the server a chance to create a new latest.log...",
            sleep_duration
        );
        std::thread::sleep(sleep_duration);

        let file =
            std::fs::File::open(&latest_log).with_context(|| "failed to open latest log file")?;
        let mut reader = std::io::BufReader::new(file);
        let mut position = 0;

        // A hint that might be helpful for debugging in the event an error occurs.
        let hint = format!(
            "Run `cat {} | tail -n 50` to read the error logs",
            latest_log.display()
        );

        for attempt in 0..12 {
            tracing::debug!("Checking server status: attempt #{}", attempt + 1);

            reader.seek(std::io::SeekFrom::Start(position))?;
            let mut lines = Vec::new();

            for line in reader.by_ref().lines() {
                let line = line?;
                lines.push(line);
            }

            for line in lines {
                tracing::debug!("Reading line: {}", line);

                if line.ends_with(r#"s)! For help, type "help""#) {
                    writeln!(stderr, "Server started successfully!").ok();
                    return Ok(());
                } else if line.ends_with("Failed to start the minecraft server") {
                    let message = "An error occurred while starting the server".to_owned();
                    #[rustfmt::skip]
                    return Err(crate::Error::new(message, None).with_hint(Some(hint)).into());
                } else {
                    position = reader.stream_position()?;
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(5));
        }

        // Check if the window is still open as a last effort.
        // Ping the server to see if it disconnected as a last effort.

        let message = "Axiom timed out while waiting for the server to start".to_owned();
        Err(crate::Error::new(message, None)
            .with_hint(Some(hint))
            .into())
    }
}
