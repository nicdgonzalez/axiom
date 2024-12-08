// TODO: Organize this file better...

use std::io::BufRead;

use anyhow::anyhow;

pub fn init() -> anyhow::Result<()> {
    let backups = get_backups_path()?;
    let jars = get_jars_path()?;
    let servers = get_servers_path()?;

    for dir in [backups, jars, servers].iter() {
        std::fs::create_dir_all(dir)?;
    }

    Ok(())
}

pub fn normalize_server_name(name: &str) -> String {
    static MAX_LENGTH: u8 = 255; // Max filename length on Windows and Linux
    let normalized = name
        .trim()
        .chars()
        .take(MAX_LENGTH as usize)
        .map(|c| {
            // Invalid characters are replaced with a dash.
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase();

    normalized
}

pub fn get_axiom_path() -> anyhow::Result<std::path::PathBuf> {
    let axiom = dirs::data_dir()
        .ok_or_else(|| anyhow!("unable to get the data directory"))?
        .join("axiom");

    Ok(axiom)
}

pub fn get_backups_path() -> anyhow::Result<std::path::PathBuf> {
    let backups = get_axiom_path()?.join("backups");
    Ok(backups)
}

pub fn get_jars_path() -> anyhow::Result<std::path::PathBuf> {
    let jars = get_axiom_path()?.join("jars");
    Ok(jars)
}

pub fn get_servers_path() -> anyhow::Result<std::path::PathBuf> {
    let servers = get_axiom_path()?.join("servers");
    Ok(servers)
}

pub fn get_servers_dirs() -> anyhow::Result<Vec<std::fs::DirEntry>> {
    let servers: Vec<std::fs::DirEntry> = get_servers_path()?
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_dir()))
        .collect();

    Ok(servers)
}

pub fn get_server_path(name: &str) -> anyhow::Result<std::path::PathBuf> {
    let server = get_servers_path()?.join(name);
    Ok(server)
}

pub fn get_server_backups_path(name: &str) -> anyhow::Result<std::path::PathBuf> {
    let backups = get_backups_path()?.join(name);
    Ok(backups)
}

pub fn validate_server_exists(name: &str) -> anyhow::Result<(String, std::path::PathBuf)> {
    let name = normalize_server_name(name);
    let server = get_server_path(&name)?;

    if !server.try_exists()? {
        return Err(anyhow!("server with name '{name}' not found"));
    }

    Ok((name, server))
}

pub fn validate_server_not_exists(name: &str) -> anyhow::Result<(String, std::path::PathBuf)> {
    let name = normalize_server_name(name);
    let server = get_server_path(&name)?;

    if server.try_exists()? {
        return Err(anyhow!("server with name '{name}' already exists"));
    }

    Ok((name, server))
}

static BASE_URL: &str = "https://api.papermc.io/v2";

pub fn get_paper_server_versions() -> anyhow::Result<Vec<String>> {
    let url = format!("{}/projects/paper", BASE_URL);

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;

    let data: serde_json::Value = serde_json::from_str(&response)?;
    let data = data.as_object().expect("expected JSON object");

    let versions: Vec<String> = data
        .get("versions")
        .expect("expected field 'versions'")
        .as_array()
        .expect("expected 'versions' to be an array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("expected 'versions' to be an array of strings")
                .to_string()
        })
        .collect();

    Ok(versions)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelKind {
    Default,
    Experimental,
}

pub struct Build {
    pub version: String,
    pub build: i64,
    pub channel: ChannelKind,
    pub filename: String,
}

pub fn get_paper_build_latest(version: &str) -> anyhow::Result<Build> {
    let url = format!("{}/projects/paper/versions/{}/builds", BASE_URL, version);

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;

    let data: serde_json::Value = serde_json::from_str(&response)?;
    let data = data.as_object().expect("expected JSON object");

    let builds = data
        .get("builds")
        .expect("expected field 'builds'")
        .as_array()
        .expect("expected 'builds' to be an array")
        .iter()
        .last()
        .map(|entry| {
            let build = entry
                .get("build")
                .expect("expected field 'build'")
                .as_i64()
                .expect("expected 'builds' to be an array of i64");
            let channel = match entry
                .get("channel")
                .expect("expected field 'channel'")
                .as_str()
                .expect("expected field 'channel' to be a string")
            {
                "default" => ChannelKind::Default,
                "experimental" => ChannelKind::Experimental,
                _ => unreachable!(),
            };
            let filename = entry
                .get("downloads")
                .expect("expected field 'downloads'")
                .as_object()
                .expect("expected 'downloads' to be a JSON object")
                .get("application")
                .expect("expected field 'application'")
                .as_object()
                .expect("expected 'application' to be a JSON object")
                .get("name")
                .expect("expected field 'name'")
                .as_str()
                .expect("expected 'name' to be a string")
                .to_string();

            Build {
                version: version.to_owned(),
                build,
                channel,
                filename,
            }
        })
        .expect("no builds available");

    Ok(builds)
}

pub struct ServerJar {
    pub filename: String,
    pub data: Vec<u8>,
}

pub fn get_paper_server_jar(build: &Build) -> anyhow::Result<ServerJar> {
    let parts: Vec<&str> = build.filename.split('-').collect();
    let version = parts
        .get(1)
        .expect("expected filename format: paper-{version}-{build}.jar");
    let build_number = parts
        .get(2)
        .expect("expected filename format: paper-{version}-{build}.jar")
        .split_once('.')
        .expect("expected filename to end with `.jar`")
        .0;

    let url = format!(
        "{}/projects/paper/versions/{}/builds/{}/downloads/{}",
        BASE_URL, &version, &build_number, &build.filename
    );

    let client = reqwest::blocking::Client::new();
    let data = client
        .get(url)
        .timeout(std::time::Duration::from_secs(60 * 2))
        .send()?
        .bytes()?
        .to_vec();

    Ok(ServerJar {
        filename: build.filename.to_owned(),
        data,
    })
}

pub fn get_version_installed(server_jar: &std::path::PathBuf) -> Option<String> {
    let file = server_jar
        .exists()
        .then(|| {
            server_jar.is_symlink().then(|| {
                server_jar
                    .read_link()
                    .expect("failed to follow server.jar symlink")
            })
        })
        .flatten()
        .or_else(|| Some(server_jar.to_path_buf()));

    let version = file.and_then(|f| {
        f.file_name()
            .and_then(|name| name.to_str())
            .and_then(|name_str| name_str.split('-').nth(1))
            .map(|version| version.to_string())
    });

    version
}

// NOTE: This method is slow... maybe lock behind a flag in case you
// *really* want to know which version the server is using?
pub fn get_server_version(server_jar: &std::path::PathBuf) -> Option<String> {
    let output = std::process::Command::new("java")
        .args([
            "-jar",
            server_jar
                .to_str()
                .expect("expected path to be valid unicode"),
            "--version",
        ])
        .current_dir(server_jar.parent().unwrap())
        .output()
        .expect("failed to execute java command");

    output
        .stdout
        .lines()
        .last()
        .and_then(|line| line.ok()?.split('-').nth(0).map(String::from))
}
