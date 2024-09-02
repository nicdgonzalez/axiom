//! # Paper

static BASE_URL: &str = "https://api.papermc.io/v2";

pub struct ServerJar {
    pub filename: String,
    pub data: Vec<u8>,
}

pub fn get_versions() -> anyhow::Result<Vec<String>> {
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
        .map(|v| v.to_string())
        .collect();

    Ok(versions)
}

pub fn get_builds(version: &str) -> anyhow::Result<Vec<i64>> {
    let url = format!("{}/projects/paper/versions/{}", BASE_URL, version);

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;

    let data: serde_json::Value = serde_json::from_str(&response)?;
    let data = data.as_object().expect("expected JSON object");

    let builds: Vec<i64> = data
        .get("builds")
        .expect("expected field 'builds'")
        .as_array()
        .expect("expected 'builds' to be an array")
        .iter()
        .map(|v| v.as_i64().expect("expected 'builds' to be an array of i64"))
        .collect();

    Ok(builds)
}

pub fn get_filename(version: &str, build: &i64) -> anyhow::Result<String> {
    let url = format!(
        "{}/projects/paper/versions/{}/builds/{}",
        BASE_URL, &version, &build
    );

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;

    let data: serde_json::Value = serde_json::from_str(&response)?;
    let data = data.as_object().expect("expected JSON object");

    let filename = data
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

    Ok(filename)
}

pub fn get_filename_unchecked(version: &str, build: &i64) -> String {
    format!("paper-{}-{}.jar", &version, &build)
}

pub fn get_server_jar(filename: &str) -> anyhow::Result<ServerJar> {
    let parts: Vec<&str> = filename.split('-').collect();
    let version = parts
        .get(1)
        .expect("expected filename like: `paper-{version}-{build}.jar`");
    let build = parts
        .get(2)
        .expect("expected filename like: `paper-{version}-{build}.jar`")
        .split_once('.')
        .expect("expected filename to end with `.jar`")
        .0;

    let url = format!(
        "{}/projects/paper/versions/{}/builds/{}/downloads/{}",
        BASE_URL, &version, &build, &filename
    );

    let client = reqwest::blocking::Client::new();
    let data = client.get(url).send()?.bytes()?.to_vec();

    Ok(ServerJar {
        filename: filename.to_string(),
        data: data,
    })
}
