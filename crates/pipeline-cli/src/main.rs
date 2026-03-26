//! Ensemble asset pipeline CLI.

use clap::{Parser, Subcommand};

use pipeline::hw1::loader::load_game_dir;
use pipeline::hw1::validate::FileOutcome;

#[derive(Parser)]
#[command(name = "pipeline")]
#[command(about = "Ensemble asset pipeline — load and validate game assets")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate all database XMB files parse correctly
    Validate {
        /// Path to the game directory containing ERA files
        #[arg(long)]
        game_dir: String,
        /// Additional ERA to layer on top (e.g. a scenario ERA)
        #[arg(long = "era")]
        era_paths: Vec<String>,
    },
    /// Load the full game world (database + all asset chains)
    Load {
        /// Path to the game directory containing ERA files
        #[arg(long)]
        game_dir: String,
        /// Scenario ERA to layer on top
        #[arg(long)]
        scenario: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Validate {
            game_dir,
            era_paths,
        } => {
            println!("Loading ERAs from {game_dir}:");
            let mut src = load_game_dir(&game_dir);
            for path in &era_paths {
                match src.add_era(path) {
                    Ok(n) => println!("  Loaded {path} ({n} entries)"),
                    Err(e) => {
                        eprintln!("Failed to load {path}: {e}");
                        std::process::exit(1);
                    }
                }
            }
            println!();

            let report = pipeline::hw1::validate(&mut src);

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
                            for w in warnings {
                                println!("        ⚠ {w}");
                            }
                        }
                    }
                    FileOutcome::Failed(e) => {
                        println!("  FAIL  {:<14} {e}", f.label);
                    }
                    FileOutcome::Missing => {
                        println!("  SKIP  {:<14} not found in archive", f.label);
                    }
                }
            }

            println!("\n--- Summary ---");
            println!(
                "{} passed, {} failed, {} missing, {} warnings ({:.1}s)",
                report.passed(),
                report.failed(),
                report.missing(),
                report.total_warnings(),
                report.elapsed.as_secs_f64()
            );

            if report.failed() > 0 {
                std::process::exit(1);
            }
        }
        Commands::Load { game_dir, scenario } => {
            println!("Loading HW1 world from {game_dir}...\n");
            let world =
                pipeline::hw1::World::load(&game_dir, scenario.as_deref()).unwrap_or_else(|e| {
                    eprintln!("Failed to load world: {e}");
                    std::process::exit(1);
                });
            println!();
            world.print_summary();
        }
    }
}
