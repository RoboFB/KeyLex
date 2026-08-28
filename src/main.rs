//! The `keylex` binary. Everything it does lives in the library so the
//! integration tests can reach it too; this only picks up the exit code.

use std::process::ExitCode;

fn main() -> ExitCode {
    keylex::cli::run()
}
