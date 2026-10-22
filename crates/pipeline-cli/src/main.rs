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

/// Shipped asset source that the CLI loads from a game directory.
type Source = pipeline::source::AssetSource<pipeline::source::StdFileProvider>;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Validate {
            game_dir,
            era_paths,
        } => run_validate(&game_dir, &era_paths),
        Commands::Load { game_dir, scenario } => run_load(&game_dir, scenario.as_deref()),
        Commands::Save {
            game_dir,
            out,
            mode,
            scenario,
        } => run_save(&game_dir, &out, mode, scenario.as_deref()),
    }
}

/// Runs the `validate` command and exits with status 1 if a file fails.
fn run_validate(game_dir: &str, era_paths: &[String]) {
    println!("Loading ERAs from {game_dir}:");
    let mut src = load_game_dir(game_dir);
    for path in era_paths {
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

/// Runs the `load` command and prints the world summary.
fn run_load(game_dir: &str, scenario: Option<&str>) {
    println!("Loading HW1 world from {game_dir}...\n");
    let (mut world, mut src) = pipeline::hw1::World::load(game_dir).unwrap_or_else(|e| {
        eprintln!("Failed to load world: {e}");
        std::process::exit(1);
    });
    if let Some(scen) = scenario
        && let Err(e) = world.swap_scenario(&mut src, scen)
    {
        eprintln!("Failed to load scenario {scen}: {e}");
        std::process::exit(1);
    }
    println!();
    world.print_summary();
}

/// Writes documents to the override directory and counts the results.
struct SaveWriter<'a> {
    /// Asset source with the override directory set.
    src: &'a Source,
    /// Output format of each document.
    mode: SaveMode,
    /// Number of files written.
    written: u32,
    /// Number of files that failed to serialize or write.
    failed: u32,
}

impl SaveWriter<'_> {
    /// Writes one serialized document to `path`.
    ///
    /// `what` names the item in the error message if serialization failed.
    fn write<E: std::fmt::Display>(
        &mut self,
        path: &str,
        doc: Result<pipeline::xmb::Document, E>,
        what: &str,
    ) {
        let doc = match doc {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("  FAIL {what}: {e}");
                self.failed += 1;
                return;
            }
        };
        let result = match self.mode {
            SaveMode::Unpacked => self.src.write_xml(path, &doc),
            SaveMode::Packed => self.src.write_xmb(path, &doc),
        };
        match result {
            Ok(disk_path) => {
                println!("  wrote {}", disk_path.display());
                self.written += 1;
            }
            Err(e) => {
                eprintln!("  FAIL {path}: {e}");
                self.failed += 1;
            }
        }
    }
}

/// Runs the `save` command and exits with status 1 if a file fails.
fn run_save(game_dir: &str, out: &str, mode: SaveMode, scenario: Option<&str>) {
    let mode_label = match mode {
        SaveMode::Unpacked => "unpacked (XML)",
        SaveMode::Packed => "packed (XMB)",
    };

    println!("Loading HW1 world from {game_dir}...");
    let mut src = match scenario {
        Some(era) => pipeline::hw1::loader::load_with_scenario(game_dir, era),
        None => load_game_dir(game_dir),
    };
    src.set_override_dir(out);

    let world = pipeline::hw1::World::load_from_source(&mut src).unwrap_or_else(|e| {
        eprintln!("Failed to load world: {e}");
        std::process::exit(1);
    });

    let mut writer = SaveWriter {
        src: &src,
        mode,
        written: 0,
        failed: 0,
    };

    // 1. Database files (10 documents)
    let docs = world.database.to_documents().unwrap_or_else(|e| {
        eprintln!("Failed to serialize database: {e}");
        std::process::exit(1);
    });
    println!(
        "\nWriting {} database files as {mode_label} to {out}...",
        docs.len()
    );
    for (path, doc) in docs {
        writer.write(path.as_str(), Ok::<_, String>(doc), path.as_str());
    }

    // 1b. Scenario descriptions
    if !world.scenario_list.scenarios.is_empty() {
        println!(
            "\nWriting scenario descriptions ({} scenarios)...",
            world.scenario_list.scenarios.len()
        );
        writer.write(
            pipeline::hw1::scenario::ScenarioList::GAME_PATH,
            world.scenario_list.to_document(),
            "scenariodescriptions",
        );
    }

    save_asset_chains(&mut writer, &world, mode_label);

    println!("\n--- Summary ---");
    println!("{} written, {} failed", writer.written, writer.failed);
    if writer.failed > 0 {
        std::process::exit(1);
    }
}

/// Writes the visuals, tactics and physics chains of `world`.
fn save_asset_chains(writer: &mut SaveWriter<'_>, world: &pipeline::hw1::World, mode_label: &str) {
    use pipeline::database::hw1::{physics, tactics, visual};

    // 2. Visuals
    println!(
        "\nWriting {} visuals as {mode_label}...",
        world.visuals.len()
    );
    for (obj_name, vis) in &world.visuals {
        if let Some(vis_path) = world.assets.get(obj_name).and_then(|a| a.visual.as_ref()) {
            let what = format!("{obj_name} serialize");
            writer.write(vis_path, visual::to_document(vis), &what);
        }
    }

    // 3. Tactics
    println!(
        "\nWriting {} tactics as {mode_label}...",
        world.tactics.len()
    );
    for (obj_name, tac) in &world.tactics {
        if let Some(tac_path) = world.assets.get(obj_name).and_then(|a| a.tactics.as_ref()) {
            let what = format!("{obj_name} serialize");
            writer.write(tac_path, tactics::to_document(tac), &what);
        }
    }

    // 4. Physics chains (physics + blueprint + shape)
    println!(
        "\nWriting {} physics chains as {mode_label}...",
        world.physics.len()
    );
    for (obj_name, chain) in &world.physics {
        let Some(obj_assets) = world.assets.get(obj_name) else {
            continue;
        };
        if let Some(phys_path) = &obj_assets.physics {
            let what = format!("{obj_name} physics serialize");
            writer.write(
                phys_path,
                physics::physics_to_document(&chain.physics),
                &what,
            );
        }
        if let (Some(bp), Some(bp_path)) = (&chain.blueprint, &obj_assets.blueprint) {
            let what = format!("{obj_name} blueprint serialize");
            writer.write(bp_path, physics::blueprint_to_document(bp), &what);
        }
        if let (Some(shp), Some(shp_path)) = (&chain.shape, &obj_assets.shape) {
            let what = format!("{obj_name} shape serialize");
            writer.write(shp_path, physics::shape_to_document(shp), &what);
        }
    }
}
