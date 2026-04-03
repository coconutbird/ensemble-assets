//! HW1 integration tests.
//!
//! Requires `HW1_GAME_DIR` to be set in the workspace `.env` file
//! (or as an environment variable) pointing to a Halo Wars DE installation.
//!
//! These tests are skipped when the variable is not set.

mod base;
mod scenario;

use pipeline::hw1::loader::load_game_dir;
use pipeline::hw1::validate::{FileOutcome, ValidateReport};
use pipeline::source::{AssetSource, StdFileProvider};

/// Load the `.env` from the workspace root and return `HW1_GAME_DIR` if set.
fn hw1_game_dir() -> Option<String> {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("could not find workspace root");
    let env_path = workspace_root.join(".env");
    let _ = dotenvy::from_path(&env_path);
    std::env::var("HW1_GAME_DIR").ok()
}

fn load_hw1(dir: &str) -> AssetSource<StdFileProvider> {
    load_game_dir(dir)
}

fn print_report(report: &ValidateReport) {
    for f in &report.files {
        match &f.outcome {
            FileOutcome::Ok { summary, warnings } => {
                if warnings.is_empty() {
                    println!("  OK    {:<14} {summary}", f.label);
                } else {
                    println!(
                        "  OK    {:<14} {summary}  ({} warnings)",
                        f.label,
                        warnings.len()
                    );
                }
            }
            FileOutcome::Failed(e) => println!("  FAIL  {:<14} {e}", f.label),
            FileOutcome::Missing => println!("  SKIP  {:<14} not found", f.label),
        }
    }
    println!(
        "\n  {} passed, {} failed, {} missing, {} warnings ({:.1}s)",
        report.passed(),
        report.failed(),
        report.missing(),
        report.total_warnings(),
        report.elapsed.as_secs_f64()
    );
}
