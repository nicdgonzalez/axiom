use std::io::{BufRead, Read, Seek, Write};
use std::os::unix::process::ExitStatusExt;

use anyhow::Context;

use super::build::Build;
use super::{TMUX_SERVER_NAME, TMUX_SESSION_NAME};

#[derive(clap::Args)]
pub struct Start;

impl crate::commands::Run for Start {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<(), crate::error::Error> {
        let package = ctx
            .package()
            .with_context(|| "failed to get package manifest")?;

        let tmux_window_name = package.name();

        let status = std::process::Command::new("tmux")
            .current_dir(package.path())
            .args([
                "-L",
                TMUX_SERVER_NAME,
                "has-session",
                "-t",
                &format!("={}:{}", TMUX_SESSION_NAME, tmux_window_name),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .with_context(|| "failed to execute command 'tmux'")?;

        if status.success() {
            crate::bail!("a package with the same name is already running");
        }

        tracing::info!("building the Minecraft server");
        Build::run(&Build { accept_eula: false }, ctx)?;

        let server = package.server();

        tracing::info!("starting the server");
        let status = std::process::Command::new("tmux")
            .args([
                "-L",
                TMUX_SERVER_NAME,
                "new-window",
                "-c",
                server
                    .path()
                    .to_str()
                    .with_context(|| "failed to convert current directory to string")?,
                "-d",
                "-t",
                &format!("={}", TMUX_SESSION_NAME),
                "-n",
                tmux_window_name,
                "./start.sh",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .with_context(|| "failed to execute tmux command")?;

        if !status.success() {
            let status = std::process::Command::new("tmux")
                .args([
                    "-L",
                    TMUX_SERVER_NAME,
                    "new-session",
                    "-c",
                    server
                        .path()
                        .to_str()
                        .with_context(|| "failed to convert current directory to string")?,
                    "-d",
                    "-s",
                    TMUX_SESSION_NAME,
                    "-n",
                    tmux_window_name,
                    "./start.sh",
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .with_context(|| "failed to execute tmux command")?;

            if !status.success() {
                match status.code() {
                    Some(code) => tracing::error!("command terminated with exit code: {code}"),
                    None => tracing::error!(
                        "command terminated via signal: {}",
                        status.signal().unwrap()
                    ),
                }

                crate::bail!("failed to create tmux session");
            }
        }

        let latest_log = server.logs().join("latest.log");

        let sleep_duration = std::time::Duration::from_secs(5);
        tracing::debug!(
            "sleeping for {:?} seconds to give the server a chance to create a new latest.log...",
            sleep_duration
        );
        std::thread::sleep(sleep_duration);

        let file = std::fs::File::open(&latest_log).with_context(|| "failed to open latest.log")?;
        let mut reader = std::io::BufReader::new(file);
        let mut position = 0;

        // A hint that might be helpful for debugging in the event an error occurs.
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

                if line.ends_with(r#"s)! For help, type "help""#) {
                    // TODO: Provide better output:
                    //
                    // Built {package.name()} in XX.XXs
                    // Starting the Minecraft server...
                    // ----------------------------------------------------------------------------
                    // Version: 1.21.6 (#44)
                    // Server IP: localhost
                    // Port: 25565
                    // ----------------------------------------------------------------------------
                    // Server is now online!
                    //
                    // Use `axiom --help` for a list of available commands.
                    writeln!(stderr, "🟢 server is now online!").ok();
                    return Ok(());
                } else if line.ends_with("Failed to start the minecraft server") {
                    let message = "An error occurred while starting the server".to_owned();
                    let err = anyhow::anyhow!(message);
                    return Err(crate::error::Error::new_with_hint(err, hint));
                } else {
                    position = reader
                        .stream_position()
                        .with_context(|| "failed to get cursor position")?;
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(5));
        }

        // Check if the window is still open as a last effort.
        // Ping the server to see if it disconnected as a last effort.

        let message = "Axiom timed out while waiting for the server to start".to_owned();
        Err(crate::error::Error::new_with_hint(
            anyhow::anyhow!(message),
            hint,
        ))
    }
}
