//! HW1 integration tests.
//!
//! Each test requires `HW1_GAME_DIR` in the workspace `.env` file or in the environment. The
//! variable names a Halo Wars DE installation. The tests are ignored by default; run them with
//! `cargo test --workspace --all-features -- --ignored`.

mod base;
mod save;
mod scenario;
mod scenario_setup;

use pipeline::hw1::loader::load_game_dir;
use pipeline::hw1::validate::{FileOutcome, ValidateReport};
use pipeline::source::{AssetSource, StdFileProvider};

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
