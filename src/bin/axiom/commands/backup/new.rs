//! Implementation for the `backup new` command.

use anyhow::Context;
use colored::Colorize;
use flate2::{write::GzEncoder, Compression};

#[derive(clap::Args)]
pub struct Args {
    /// A unique name used to identify the server.
    pub name: String,
    /// Block the current process until the backup is complete.
    #[arg(long)]
    wait: bool,
}

/// Compress a server's files into a tarball.
pub fn run(args: &Args) -> Result<(), anyhow::Error> {
    let (name, server) = axiom::validate_server_exists(&args.name)?;
    let server_backups = prepare_backup_directory(&name)?;
    run_backup_in_thread(name.clone(), server, server_backups, args.wait)?;
    Ok(())
}

fn prepare_backup_directory(name: &str) -> anyhow::Result<std::path::PathBuf> {
    let server_backups = axiom::get_server_backups_path(name)?;

    if !server_backups.try_exists()? {
        std::fs::create_dir_all(&server_backups)?;
    }

    Ok(server_backups)
}

fn generate_backup_filename(name: &str) -> String {
    // NOTE: Designed for daily backups. If you need to backup more
    // frequently, consider adding the time to the filename as needed.
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    format!("{}_{}.tar.gz", today, name)
}

fn handle_backup_process(
    name: String,
    server: std::path::PathBuf,
    server_backups: std::path::PathBuf,
    filename: String,
) {
    let session_name = format!("axiom_{}", &name);

    // Temporarily disable auto-save if the server is running.
    if let Ok(true) = axiom::tmux::exists(&session_name) {
        axiom::tmux::send_command(&session_name, "save-off")
            .expect("failed to send save-off command");
        axiom::tmux::send_command(
            &session_name,
            "say Server backup in progress. Auto-save has been disabled!",
        )
        .expect("failed to send say command");
    }

    // Create the backup file
    let file = std::fs::File::create(&filename).expect("failed to create backup file");
    let encoder = GzEncoder::new(file, Compression::best());

    // Compress the directory into a tarball
    let mut tar = tar::Builder::new(encoder);
    if let Err(why) = tar.append_dir_all("", &server) {
        std::fs::remove_file(server_backups.join(&filename))
            .expect("failed to remove file after failed backup operation");
        panic!("failed to compress server directory: {why}");
    }

    // Backup complete; turn auto-save back on.
    if let Ok(true) = axiom::tmux::exists(&session_name) {
        axiom::tmux::send_command(&session_name, "save-on")
            .expect("failed to send save-on command");
        axiom::tmux::send_command(
            &session_name,
            "say Server backup complete. Auto-save has been re-enabled!",
        )
        .expect("failed to send say command");
    }
}

fn run_backup_in_thread(
    name: String,
    server: std::path::PathBuf,
    server_backups: std::path::PathBuf,
    wait: bool,
) -> anyhow::Result<()> {
    std::env::set_current_dir(&server_backups)
        .with_context(|| "failed to change into server's backup directory")?;

    let filename = generate_backup_filename(&name);
    let handle = std::thread::Builder::new()
        .spawn(move || handle_backup_process(name, server, server_backups, filename))
        .with_context(|| "failed to start server backup in a separate thread")?;

    println!("{}", "Backup started! Please wait a few minutes.".yellow());

    if wait {
        handle
            .join()
            .expect("unable to join on the associated thread");
    } else {
        eprintln!(
            "{}: use '--wait' to block until the backup is complete.",
            "hint".bold().green()
        );
    }

    Ok(())
}
