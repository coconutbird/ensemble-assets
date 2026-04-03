//! Ensemble asset pipeline CLI.

use clap::{Parser, Subcommand, ValueEnum};

use pipeline::hw1::loader::load_game_dir;
use pipeline::hw1::validate::FileOutcome;

/// Output format for the `save` command.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum SaveMode {
    /// Human-readable XML files (.xml)
    #[default]
    Unpacked,
    /// Binary XMB files (.xmb) — game-native format
    Packed,
}

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
    /// Save all database files to an override directory
    Save {
        /// Path to the game directory containing ERA files
        #[arg(long)]
        game_dir: String,
        /// Output directory for override files
        #[arg(long, short)]
        out: String,
        /// Output format: unpacked (XML) or packed (XMB)
        #[arg(long, short, value_enum, default_value_t = SaveMode::Unpacked)]
        mode: SaveMode,
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
            let (mut world, mut src) = pipeline::hw1::World::load(&game_dir).unwrap_or_else(|e| {
                eprintln!("Failed to load world: {e}");
                std::process::exit(1);
            });
            if let Some(scen) = &scenario {
                world.swap_scenario(&mut src, scen);
            }
            println!();
            world.print_summary();
        }
        Commands::Save {
            game_dir,
            out,
            mode,
            scenario,
        } => {
            let mode_label = match mode {
                SaveMode::Unpacked => "unpacked (XML)",
                SaveMode::Packed => "packed (XMB)",
            };

            println!("Loading HW1 world from {game_dir}...");
            let mut src = match &scenario {
                Some(era) => pipeline::hw1::loader::load_with_scenario(&game_dir, era),
                None => load_game_dir(&game_dir),
            };
            src.set_override_dir(&out);

            let world = pipeline::hw1::World::load_from_source(&mut src).unwrap_or_else(|e| {
                eprintln!("Failed to load world: {e}");
                std::process::exit(1);
            });

            // Helper: write a document in the chosen mode
            let write_doc = |src: &pipeline::source::AssetSource<_>,
                             path: &str,
                             doc: &pipeline::xmb::Document|
             -> Result<std::path::PathBuf, String> {
                match mode {
                    SaveMode::Unpacked => src.write_xml(path, doc),
                    SaveMode::Packed => src.write_xmb(path, doc),
                }
            };

            let mut written = 0u32;
            let mut failed = 0u32;

            // 1. Database files (10 documents)
            let docs = world.database.to_documents().unwrap_or_else(|e| {
                eprintln!("Failed to serialize database: {e}");
                std::process::exit(1);
            });
            println!(
                "\nWriting {} database files as {mode_label} to {out}...",
                docs.len()
            );
            for (path, doc) in &docs {
                match write_doc(&src, path, doc) {
                    Ok(disk_path) => {
                        println!("  wrote {}", disk_path.display());
                        written += 1;
                    }
                    Err(e) => {
                        eprintln!("  FAIL {path}: {e}");
                        failed += 1;
                    }
                }
            }

            // 1b. Scenario descriptions
            if !world.scenario_list.scenarios.is_empty() {
                println!(
                    "\nWriting scenario descriptions ({} scenarios)...",
                    world.scenario_list.scenarios.len()
                );
                match world.scenario_list.to_document() {
                    Ok(doc) => {
                        let path = pipeline::hw1::scenario::ScenarioList::GAME_PATH;
                        match write_doc(&src, path, &doc) {
                            Ok(disk_path) => {
                                println!("  wrote {}", disk_path.display());
                                written += 1;
                            }
                            Err(e) => {
                                eprintln!("  FAIL {path}: {e}");
                                failed += 1;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("  FAIL scenariodescriptions: {e}");
                        failed += 1;
                    }
                }
            }

            // 2. Visuals
            println!(
                "\nWriting {} visuals as {mode_label}...",
                world.visuals.len()
            );
            for (obj_name, vis) in &world.visuals {
                if let Some(vis_path) = world.assets.get(obj_name).and_then(|a| a.visual.as_ref()) {
                    match pipeline::database::hw1::visual::to_document(vis) {
                        Ok(doc) => match write_doc(&src, vis_path, &doc) {
                            Ok(disk_path) => {
                                written += 1;
                                println!("  wrote {}", disk_path.display());
                            }
                            Err(e) => {
                                eprintln!("  FAIL {vis_path}: {e}");
                                failed += 1;
                            }
                        },
                        Err(e) => {
                            eprintln!("  FAIL {obj_name} serialize: {e}");
                            failed += 1;
                        }
                    }
                }
            }

            // 3. Tactics
            println!(
                "\nWriting {} tactics as {mode_label}...",
                world.tactics.len()
            );
            for (obj_name, tac) in &world.tactics {
                if let Some(tac_path) = world.assets.get(obj_name).and_then(|a| a.tactics.as_ref())
                {
                    match pipeline::database::hw1::tactics::to_document(tac) {
                        Ok(doc) => match write_doc(&src, tac_path, &doc) {
                            Ok(disk_path) => {
                                written += 1;
                                println!("  wrote {}", disk_path.display());
                            }
                            Err(e) => {
                                eprintln!("  FAIL {tac_path}: {e}");
                                failed += 1;
                            }
                        },
                        Err(e) => {
                            eprintln!("  FAIL {obj_name} serialize: {e}");
                            failed += 1;
                        }
                    }
                }
            }

            // 4. Physics chains (physics + blueprint + shape)
            println!(
                "\nWriting {} physics chains as {mode_label}...",
                world.physics.len()
            );
            for (obj_name, chain) in &world.physics {
                let obj_assets = match world.assets.get(obj_name) {
                    Some(a) => a,
                    None => continue,
                };

                // .physics
                if let Some(phys_path) = &obj_assets.physics {
                    match pipeline::database::hw1::physics::physics_to_document(&chain.physics) {
                        Ok(doc) => match write_doc(&src, phys_path, &doc) {
                            Ok(dp) => {
                                written += 1;
                                println!("  wrote {}", dp.display());
                            }
                            Err(e) => {
                                eprintln!("  FAIL {phys_path}: {e}");
                                failed += 1;
                            }
                        },
                        Err(e) => {
                            eprintln!("  FAIL {obj_name} physics serialize: {e}");
                            failed += 1;
                        }
                    }
                }

                // .blueprint
                if let (Some(bp), Some(bp_path)) = (&chain.blueprint, &obj_assets.blueprint) {
                    match pipeline::database::hw1::physics::blueprint_to_document(bp) {
                        Ok(doc) => match write_doc(&src, bp_path, &doc) {
                            Ok(dp) => {
                                written += 1;
                                println!("  wrote {}", dp.display());
                            }
                            Err(e) => {
                                eprintln!("  FAIL {bp_path}: {e}");
                                failed += 1;
                            }
                        },
                        Err(e) => {
                            eprintln!("  FAIL {obj_name} blueprint serialize: {e}");
                            failed += 1;
                        }
                    }
                }

                // .shp
                if let (Some(shp), Some(shp_path)) = (&chain.shape, &obj_assets.shape) {
                    match pipeline::database::hw1::physics::shape_to_document(shp) {
                        Ok(doc) => match write_doc(&src, shp_path, &doc) {
                            Ok(dp) => {
                                written += 1;
                                println!("  wrote {}", dp.display());
                            }
                            Err(e) => {
                                eprintln!("  FAIL {shp_path}: {e}");
                                failed += 1;
                            }
                        },
                        Err(e) => {
                            eprintln!("  FAIL {obj_name} shape serialize: {e}");
                            failed += 1;
                        }
                    }
                }
            }

            println!("\n--- Summary ---");
            println!("{written} written, {failed} failed");
            if failed > 0 {
                std::process::exit(1);
            }
        }
    }
}
