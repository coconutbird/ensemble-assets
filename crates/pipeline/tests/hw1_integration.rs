//! Integration tests for the HW1 asset pipeline.
//!
//! Requires `HW1_GAME_DIR` to be set in the workspace `.env` file
//! (or as an environment variable) pointing to a Halo Wars DE installation.
//!
//! These tests are skipped when the variable is not set.

use pipeline::hw1::loader::load_game_dir;
use pipeline::hw1::validate::{FileOutcome, ValidateReport};
use pipeline::source::AssetSource;
use pipeline::source::StdFileProvider;

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

// ── Validation tests ────────────────────────────────────────────────────

#[test]
fn validate_base_game() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let mut src = load_hw1(&dir);
    let report = pipeline::hw1::validate(&mut src);

    print_report(&report);

    assert_eq!(report.missing(), 0, "some database files were not found");
    assert!(
        report.passed() >= 7,
        "expected at least 7 files to pass, got {}",
        report.passed()
    );
}

#[test]
fn validate_with_scenario_era() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let mut src = load_hw1(&dir);

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found at {scenario_path}");
        return;
    }
    src.add_era(&scenario_path)
        .expect("failed to load PHXscn01.era");

    let report = pipeline::hw1::validate(&mut src);

    print_report(&report);

    assert_eq!(report.missing(), 0, "some database files were not found");
    assert!(
        report.passed() >= 7,
        "expected at least 7 files to pass, got {}",
        report.passed()
    );
}

// ── World loading tests ─────────────────────────────────────────────────

#[test]
fn load_world_base_game() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let world = pipeline::hw1::World::load(&dir, None).expect("failed to load world");

    world.print_summary();

    // Basic sanity checks
    assert!(!world.database.objects.is_empty(), "no objects loaded");
    assert!(!world.database.squads.is_empty(), "no squads loaded");
    assert!(!world.database.techs.is_empty(), "no techs loaded");
    assert!(!world.visuals.is_empty(), "no visuals resolved");
    assert!(!world.tactics.is_empty(), "no tactics resolved");
    assert!(!world.physics.is_empty(), "no physics resolved");
    assert!(
        world.stats.visuals_resolved > 100,
        "expected >100 visuals, got {}",
        world.stats.visuals_resolved
    );
}

#[test]
fn load_world_with_scenario() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found");
        return;
    }

    let world = pipeline::hw1::World::load(&dir, Some("PHXscn01.era"))
        .expect("failed to load world with scenario");

    world.print_summary();

    assert!(!world.database.objects.is_empty());
    assert!(
        !world.scenario_list.scenarios.is_empty(),
        "no scenarios loaded"
    );
}
