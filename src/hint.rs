use colored::Colorize;

pub struct Hint(pub String);

impl std::fmt::Display for Hint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("  {}: {}", "Hint".bold(), self.0).green())
    }
}
