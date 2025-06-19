use axiom_core::Manifest;

#[derive(Debug, Clone)]
pub struct Context {
    cwd: std::path::PathBuf,
    jars: std::path::PathBuf,
    server: std::path::PathBuf,
    manifest: Option<Manifest>,
}

impl Context {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().expect("failed to get the current directory");
        let jars = cwd.join("jars");
        let server = cwd.join("server");
        let manifest = Manifest::from_directory(&cwd).ok();

        Self {
            cwd,
            jars,
            server,
            manifest,
        }
    }

    /// A reference to the path of the server's base directory.
    pub fn cwd(&self) -> &std::path::Path {
        self.cwd.as_ref()
    }

    /// A reference to the path of the server's `jars` directory.
    pub fn jars(&self) -> &std::path::Path {
        self.jars.as_ref()
    }

    /// A reference to the path of the server's `server` directory.
    pub fn server(&self) -> &std::path::Path {
        self.server.as_ref()
    }

    /// A reference to the project's manifest, or [`None`] if not available.
    pub fn manifest(&self) -> Option<&axiom_core::Manifest> {
        self.manifest.as_ref()
    }

    pub fn set_manifest(&mut self, manifest: axiom_core::Manifest) {
        self.manifest = Some(manifest);
    }
}
