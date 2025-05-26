use std::io::Write;

use colored::Colorize;

use axiom::{ExitCode, run};

fn main() -> ExitCode {
    run().unwrap_or_else(|err| {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{}", "axiom failed".bold().red()).ok();

        for cause in err.chain() {
            writeln!(stderr, "  {}: {}", "Cause".bold(), cause).ok();
        }

        ExitCode::Failure
    })
}
