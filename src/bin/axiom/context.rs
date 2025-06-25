use std::rc::Rc;

use anyhow::Context as _;

#[derive(Debug, Clone, Default)]
pub struct Context {
    versions: Option<Rc<[axiom::paper::Version]>>,
    jars: Option<Rc<std::path::Path>>,
    package: Option<Rc<axiom::Package>>,
}

impl Context {
    pub fn versions(&mut self) -> Result<Rc<[axiom::paper::Version]>, anyhow::Error> {
        match &self.versions {
            Some(versions) => Ok(Rc::clone(versions)),
            None => {
                let versions = axiom::paper::versions()
                    .with_context(|| "failed to get supported Minecraft versions from PaperMC")?;
                self.versions = Some(versions.into());
                Ok(Rc::clone(self.versions.as_ref().unwrap()))
            }
        }
    }

    pub fn jars(&mut self) -> Result<Rc<std::path::Path>, anyhow::Error> {
        match &self.jars {
            Some(jars) => Ok(Rc::clone(jars)),
            None => {
                let jars = dirs::cache_dir()
                    .with_context(|| "failed to get cache directory")?
                    .join("axiom");
                self.jars = Some(jars.into());
                Ok(Rc::clone(self.jars.as_ref().unwrap()))
            }
        }
    }

    /// Get a reference to an Axiom package.
    ///
    /// A package contains the manifest (the `Axiom.toml` file) and all of its files.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - There is a problem getting the current directory.
    /// - There is a problem reading and parsing the manifest file.
    pub fn package(&mut self) -> Result<Rc<axiom::Package>, anyhow::Error> {
        match &self.package {
            Some(package) => Ok(Rc::clone(package)),
            None => {
                let path =
                    std::env::current_dir().with_context(|| "failed to get current directory")?;
                let manifest = axiom::Manifest::from_directory(&path)
                    .with_context(|| "failed to get package manifest")?;
                self.package = Some(Rc::new(axiom::Package::new(path, manifest)));
                Ok(Rc::clone(self.package.as_ref().unwrap()))
            }
        }
    }
}
