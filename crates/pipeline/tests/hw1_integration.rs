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


/// Round-trip test: load every database file, serialize back to XMB via
/// `Database::to_documents()`, re-parse, and compare counts.
///
/// This catches serialization regressions that would silently drop data.
#[test]
fn roundtrip_database_serialize() {
    let dir = match hw1_game_dir() {
        Some(d) => d,
        None => {
            eprintln!("HW1_GAME_DIR not set — skipping roundtrip test");
            return;
        }
    };

    let mut src = load_game_dir(&dir);

    // Parse original database
    let original = database::hw1::Database::load(&mut src).expect("failed to load database");
    println!("Original: {} objects, {} squads, {} techs, {} abilities, {} powers, {} civs, {} leaders, {} weapontypes, {} damagetypes",
        original.objects.len(), original.squads.len(), original.techs.len(),
        original.abilities.len(), original.powers.len(), original.civs.len(),
        original.leaders.len(), original.weapon_types.len(), original.damage_types.len());

    // Serialize to documents
    let docs = original
        .to_documents()
        .expect("failed to serialize database");
    println!("Serialized {} documents", docs.len());
    assert!(docs.len() >= 10, "expected at least 10 documents, got {}", docs.len());

    // Re-parse each document and compare counts
    for (path, doc) in &docs {
        match path.as_str() {
            "data\\objects.xml" => {
                let reparsed = database::hw1::objects::parse(doc)
                    .expect("failed to re-parse objects");
                assert_eq!(
                    reparsed.len(),
                    original.objects.len(),
                    "objects count mismatch after round-trip"
                );
                // Spot-check a few fields on the first object
                let orig = &original.objects[0];
                let rt = &reparsed[0];
                assert_eq!(orig.name, rt.name, "object name mismatch");
                assert_eq!(orig.hitpoints, rt.hitpoints, "hitpoints mismatch for {}", orig.name);
                println!("  objects: {} → {} ✓", original.objects.len(), reparsed.len());
            }
            "data\\squads.xml" => {
                let reparsed = database::hw1::squads::parse(doc)
                    .expect("failed to re-parse squads");
                assert_eq!(reparsed.len(), original.squads.len(), "squads count mismatch");
                println!("  squads: {} → {} ✓", original.squads.len(), reparsed.len());
            }
            "data\\techs.xml" => {
                let reparsed = database::hw1::techs::parse(doc)
                    .expect("failed to re-parse techs");
                assert_eq!(reparsed.len(), original.techs.len(), "techs count mismatch");
                println!("  techs: {} → {} ✓", original.techs.len(), reparsed.len());
            }
            "data\\abilities.xml" => {
                let reparsed = database::hw1::abilities::parse(doc)
                    .expect("failed to re-parse abilities");
                assert_eq!(reparsed.len(), original.abilities.len(), "abilities count mismatch");
                println!("  abilities: {} → {} ✓", original.abilities.len(), reparsed.len());
            }
            "data\\powers.xml" => {
                let reparsed = database::hw1::powers::parse(doc)
                    .expect("failed to re-parse powers");
                assert_eq!(reparsed.len(), original.powers.len(), "powers count mismatch");
                println!("  powers: {} → {} ✓", original.powers.len(), reparsed.len());
            }
            "data\\civs.xml" => {
                let reparsed = database::hw1::civs::parse(doc)
                    .expect("failed to re-parse civs");
                assert_eq!(reparsed.len(), original.civs.len(), "civs count mismatch");
                println!("  civs: {} → {} ✓", original.civs.len(), reparsed.len());
            }
            "data\\leaders.xml" => {
                let reparsed = database::hw1::leaders::parse(doc)
                    .expect("failed to re-parse leaders");
                assert_eq!(reparsed.len(), original.leaders.len(), "leaders count mismatch");
                println!("  leaders: {} → {} ✓", original.leaders.len(), reparsed.len());
            }
            "data\\weapontypes.xml" => {
                let reparsed = database::hw1::weapontypes::parse(doc)
                    .expect("failed to re-parse weapontypes");
                assert_eq!(reparsed.len(), original.weapon_types.len(), "weapontypes count mismatch");
                println!("  weapontypes: {} → {} ✓", original.weapon_types.len(), reparsed.len());
            }
            "data\\damagetypes.xml" => {
                let reparsed = database::hw1::damagetypes::parse(doc)
                    .expect("failed to re-parse damagetypes");
                assert_eq!(reparsed.len(), original.damage_types.len(), "damagetypes count mismatch");
                println!("  damagetypes: {} → {} ✓", original.damage_types.len(), reparsed.len());
            }
            "data\\gamedata.xml" => {
                let reparsed = database::hw1::gamedata::parse(doc)
                    .expect("failed to re-parse gamedata");
                let orig_gd = original.game_data.as_ref().unwrap();
                let rt_res = reparsed.resources.as_ref().map(|r| r.entries.len()).unwrap_or(0);
                let orig_res = orig_gd.resources.as_ref().map(|r| r.entries.len()).unwrap_or(0);
                let rt_pops = reparsed.pops.as_ref().map(|p| p.entries.len()).unwrap_or(0);
                let orig_pops = orig_gd.pops.as_ref().map(|p| p.entries.len()).unwrap_or(0);
                assert_eq!(rt_res, orig_res, "resources count mismatch");
                assert_eq!(rt_pops, orig_pops, "pops count mismatch");
                println!("  gamedata: {} resources, {} pops ✓", rt_res, rt_pops);
            }
            other => panic!("unexpected document path: {other}"),
        }
    }

    println!("Round-trip validation passed!");
}