//! This module implements the `list` command, which displays active Minecraft servers.

use std::io::{BufRead, Write};

use anyhow::Context;

use super::{TMUX_SERVER_NAME, TMUX_SESSION_NAME};

#[derive(clap::Args)]
pub struct List;

impl crate::commands::Run for List {
    fn run(&self, _: &mut crate::context::Context) -> Result<(), crate::error::Error> {
        let output = std::process::Command::new("tmux")
            .args([
                "-L",
                TMUX_SERVER_NAME,
                "list-panes",
                "-t",
                &format!("={}", TMUX_SESSION_NAME),
                "-s",
                "-F",
                "#{pane_current_path}",
            ])
            .output()
            .with_context(|| "failed to execute command 'tmux'")?;

        let mut stdout = std::io::stdout().lock();

        for line in output.stdout.lines() {
            let line = line.with_context(|| "failed to read line")?;
            // The pane's path should end up in the package's server directory, so `parent()`
            // should lead to the package's path.
            let package_path = std::path::Path::new(&line)
                .parent()
                .expect("expected tmux to return an absolute path");
            let manifest = axiom::Manifest::from_directory(package_path)
                .with_context(|| "failed to get package manifest")?;
            let package = axiom::Package::new(package_path.to_path_buf(), manifest);

            // XXX: This is noticeably slow. maybe follow the server.jar symlink back to its
            // original and parse the file name instead, falling back to `build_info()` only if
            // we need to.
            let build_info = package
                .server()
                .build_info()
                .with_context(|| "failed to get build information for current server JAR")?;

            writeln!(
                stdout,
                "{package_name} {version}#{build} {package_path}",
                package_name = package.name(),
                version = build_info.version(),
                build = build_info.build(),
                package_path = package.path().display()
            )
            .ok();
        }

        Ok(())
    }
}
