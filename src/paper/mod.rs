//! # Paper
//!
//! This module provides functionality for making calls to and operating on data from
//! the PaperMC API.
//!
//! # Examples
//!
//! To install the `server.jar` file for the latest version of Minecraft from PaperMC:
//!
//! ```no_run
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let build = axiom::paper::versions()?
//!         .last()
//!         .expect("no versions available")
//!         .builds()?
//!         .pop()
//!         .expect("no builds available");
//!     let path = std::env::current_dir()?.join(&build.download_name());
//!     let bytes = build.download(std::time::Duration::from_secs(60))?;
//!     assert!(std::fs::write(&path, &bytes).is_ok());
//!     Ok(())
//! }
//! ```

mod build;
mod error;
mod version;

pub use build::{Build, Channel};
pub use error::RequestError;
pub use version::Version;

pub(crate) const BASE_URL: &str = "https://api.papermc.io/v2";

/// Get all of the Minecraft versions that PaperMC supports.
pub fn versions() -> Result<Vec<Version>, RequestError> {
    let url = format!("{}/projects/paper", BASE_URL);
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(RequestError::request_failed)?;

    debug_assert!(response.status().is_success());

    let text = response.text().map_err(RequestError::response_timed_out)?;

    #[derive(serde::Deserialize)]
    struct Response {
        versions: Vec<String>,
    }

    let versions = serde_json::from_str::<Response>(&text)
        .map_err(RequestError::parse_response_failed)?
        .versions
        .into_iter()
        .map(Version::new)
        .collect();

    Ok(versions)
}
