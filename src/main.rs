use std::io::Write;

use axiom::{ExitCode, run};
use colored::Colorize;

/// The main entry point to the application.
fn main() -> ExitCode {
    run().unwrap_or_else(|err| {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{}", "An error occurred".bold().red()).ok();

        for cause in err.chain() {
            writeln!(stderr, "  {}: {}", "Cause".bold(), cause).ok();

            if let Some(hint) = err.downcast_ref::<axiom::Error>().and_then(|e| e.hint()) {
                writeln!(stderr, "  {}: {}", "Hint".bold().green(), hint).ok();
            }
        }

        ExitCode::Failure
    })
}
