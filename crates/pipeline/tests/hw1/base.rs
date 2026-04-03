//! Base game integration tests — ERA loading, database, validation,
//! models, animations, textures, and binary asset verification.

use super::{hw1_game_dir, load_hw1, print_report};
use pipeline::hw1::loader::load_game_dir;

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
    println!(
        "Original: {} objects, {} squads, {} techs, {} abilities, {} powers, {} civs, {} leaders, {} weapontypes, {} damagetypes",
        original.objects.len(),
        original.squads.len(),
        original.techs.len(),
        original.abilities.len(),
        original.powers.len(),
        original.civs.len(),
        original.leaders.len(),
        original.weapon_types.len(),
        original.damage_types.len()
    );

    // Serialize to documents
    let docs = original
        .to_documents()
        .expect("failed to serialize database");
    println!("Serialized {} documents", docs.len());
    assert!(
        docs.len() >= 10,
        "expected at least 10 documents, got {}",
        docs.len()
    );

    // Re-parse each document and compare counts
    for (path, doc) in &docs {
        match path.as_str() {
            "data\\objects.xml" => {
                let reparsed =
                    database::hw1::objects::parse(doc).expect("failed to re-parse objects");
                assert_eq!(
                    reparsed.len(),
                    original.objects.len(),
                    "objects count mismatch after round-trip"
                );
                let orig = &original.objects[0];
                let rt = &reparsed[0];
                assert_eq!(orig.name, rt.name, "object name mismatch");
                assert_eq!(
                    orig.hitpoints, rt.hitpoints,
                    "hitpoints mismatch for {}",
                    orig.name
                );
                println!(
                    "  objects: {} → {} ✓",
                    original.objects.len(),
                    reparsed.len()
                );
            }
            "data\\squads.xml" => {
                let reparsed =
                    database::hw1::squads::parse(doc).expect("failed to re-parse squads");
                assert_eq!(
                    reparsed.len(),
                    original.squads.len(),
                    "squads count mismatch"
                );
                println!("  squads: {} → {} ✓", original.squads.len(), reparsed.len());
            }
            "data\\techs.xml" => {
                let reparsed = database::hw1::techs::parse(doc).expect("failed to re-parse techs");
                assert_eq!(reparsed.len(), original.techs.len(), "techs count mismatch");
                println!("  techs: {} → {} ✓", original.techs.len(), reparsed.len());
            }

            "data\\abilities.xml" => {
                let reparsed =
                    database::hw1::abilities::parse(doc).expect("failed to re-parse abilities");
                assert_eq!(
                    reparsed.len(),
                    original.abilities.len(),
                    "abilities count mismatch"
                );
                println!(
                    "  abilities: {} → {} ✓",
                    original.abilities.len(),
                    reparsed.len()
                );
            }
            "data\\powers.xml" => {
                let reparsed =
                    database::hw1::powers::parse(doc).expect("failed to re-parse powers");
                assert_eq!(
                    reparsed.len(),
                    original.powers.len(),
                    "powers count mismatch"
                );
                println!("  powers: {} → {} ✓", original.powers.len(), reparsed.len());
            }
            "data\\civs.xml" => {
                let reparsed = database::hw1::civs::parse(doc).expect("failed to re-parse civs");
                assert_eq!(reparsed.len(), original.civs.len(), "civs count mismatch");
                println!("  civs: {} → {} ✓", original.civs.len(), reparsed.len());
            }
            "data\\leaders.xml" => {
                let reparsed =
                    database::hw1::leaders::parse(doc).expect("failed to re-parse leaders");
                assert_eq!(
                    reparsed.len(),
                    original.leaders.len(),
                    "leaders count mismatch"
                );
                println!(
                    "  leaders: {} → {} ✓",
                    original.leaders.len(),
                    reparsed.len()
                );
            }
            "data\\weapontypes.xml" => {
                let reparsed =
                    database::hw1::weapontypes::parse(doc).expect("failed to re-parse weapontypes");
                assert_eq!(
                    reparsed.len(),
                    original.weapon_types.len(),
                    "weapontypes count mismatch"
                );
                println!(
                    "  weapontypes: {} → {} ✓",
                    original.weapon_types.len(),
                    reparsed.len()
                );
            }
            "data\\damagetypes.xml" => {
                let reparsed =
                    database::hw1::damagetypes::parse(doc).expect("failed to re-parse damagetypes");
                assert_eq!(
                    reparsed.len(),
                    original.damage_types.len(),
                    "damagetypes count mismatch"
                );
                println!(
                    "  damagetypes: {} → {} ✓",
                    original.damage_types.len(),
                    reparsed.len()
                );
            }
            "data\\gamedata.xml" => {
                let reparsed =
                    database::hw1::gamedata::parse(doc).expect("failed to re-parse gamedata");
                let orig_gd = original.game_data.as_ref().unwrap();
                let rt_res = reparsed
                    .resources
                    .as_ref()
                    .map(|r| r.entries.len())
                    .unwrap_or(0);
                let orig_res = orig_gd
                    .resources
                    .as_ref()
                    .map(|r| r.entries.len())
                    .unwrap_or(0);
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

#[test]
fn read_model_ugx() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let world = pipeline::hw1::World::load(&dir, None).expect("failed to load world");
    let mut src = load_hw1(&dir);

    let obj = world
        .assets
        .values()
        .find(|a| !a.models.is_empty() && a.models.iter().any(|m| src.resolve_exact(m).is_some()))
        .expect("no object with resolvable models found");

    println!(
        "Testing UGX loading for '{}' ({} models)",
        obj.name,
        obj.models.len()
    );

    let models = world.read_object_models(&obj.name, &mut src);
    assert!(
        !models.is_empty(),
        "should have parsed at least one model for '{}'",
        obj.name
    );

    for (path, geom) in &models {
        println!(
            "  {path}: {} sections, {} bones, {} total tris",
            geom.sections.len(),
            geom.bones.len(),
            geom.total_triangles()
        );
        assert!(!geom.sections.is_empty(), "model should have sections");
    }
}

#[test]
fn read_animation_uax() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let world = pipeline::hw1::World::load(&dir, None).expect("failed to load world");
    let mut src = load_hw1(&dir);

    let with_anims = world
        .assets
        .values()
        .find(|a| !a.anims.is_empty() && a.anims.iter().any(|p| src.resolve_exact(p).is_some()))
        .expect("no object with resolvable animations found");

    println!(
        "Testing UAX loading for '{}' ({} anims)",
        with_anims.name,
        with_anims.anims.len()
    );

    let anims = world.read_object_animations(&with_anims.name, &mut src);
    println!(
        "  Parsed {} / {} animations",
        anims.len(),
        with_anims.anims.len()
    );

    assert!(
        !anims.is_empty(),
        "should have parsed at least one animation"
    );

    for (path, anim) in &anims {
        let name = anim.animation_name().ok().flatten().unwrap_or_default();
        let duration = anim.duration().unwrap_or(0.0);
        println!("  {path}: name='{name}' duration={duration:.3}s");
    }
}

#[test]
fn read_texture_ddx() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let world = pipeline::hw1::World::load(&dir, None).expect("failed to load world");
    let mut src = load_hw1(&dir);

    let mut tex_paths = Vec::new();
    let mut obj_name = String::new();
    for obj in world.assets.values() {
        if obj.models.is_empty() {
            continue;
        }
        if !obj.models.iter().any(|m| src.resolve_exact(m).is_some()) {
            continue;
        }
        let paths = world.resolve_textures_for_obj(obj, &mut src);
        if !paths.is_empty() {
            tex_paths = paths;
            obj_name = obj.name.clone();
            break;
        }
    }
    assert!(
        !tex_paths.is_empty(),
        "no object with resolvable textures found"
    );
    println!("'{}' references {} textures", obj_name, tex_paths.len());

    let mut loaded = 0;
    let mut failed = 0;
    for path in &tex_paths {
        match world.read_texture(path, &mut src) {
            Some(tex) => {
                println!(
                    "  OK  {path}: {}x{} {:?}",
                    tex.info.width, tex.info.height, tex.info.data_format
                );
                loaded += 1;
            }
            None => {
                println!("  MISS {path}");
                failed += 1;
            }
        }
    }

    println!("  Loaded {loaded}, missing {failed}");
    assert!(loaded > 0, "should have loaded at least one texture");
}

#[test]
fn validate_binary_assets_sample() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let world = pipeline::hw1::World::load(&dir, None).expect("failed to load world");
    let mut src = load_hw1(&dir);

    println!("Manifest:");
    println!("  Model refs:   {}", world.manifest.model_refs.len());
    println!("  Anim refs:    {}", world.manifest.anim_refs.len());
    println!("  Texture refs: {}", world.manifest.texture_refs.len());

    let verify = world.manifest.verify(&src);
    println!("\nVerify (existence):");
    println!(
        "  Models:   {} found, {} missing",
        verify.models_found,
        verify.models_missing.len()
    );
    println!(
        "  Anims:    {} found, {} missing",
        verify.anims_found,
        verify.anims_missing.len()
    );
    println!(
        "  Textures: {} found, {} missing",
        verify.textures_found,
        verify.textures_missing.len()
    );

    let validation = world.validate_binary_assets(&mut src);
    println!("\nDeep validation (parse):");
    println!(
        "  Models:   {} ok, {} failed, {} missing",
        validation.models_ok,
        validation.models_failed.len(),
        validation.models_missing.len()
    );
    println!(
        "  Anims:    {} ok, {} failed, {} missing",
        validation.anims_ok,
        validation.anims_failed.len(),
        validation.anims_missing.len()
    );
    println!(
        "  Textures: {} ok, {} failed, {} missing",
        validation.textures_ok,
        validation.textures_failed.len(),
        validation.textures_missing.len()
    );

    for f in &validation.models_failed {
        println!("  MODEL FAIL: {f}");
    }
    for f in &validation.anims_failed {
        println!("  ANIM FAIL: {f}");
    }
    for f in &validation.textures_failed {
        println!("  TEX FAIL: {f}");
    }

    assert_eq!(
        validation.models_failed.len(),
        0,
        "some models failed to parse"
    );
    assert_eq!(
        validation.anims_failed.len(),
        0,
        "some animations failed to parse"
    );
}

#[test]
fn texture_refs_populated_eagerly() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let world = pipeline::hw1::World::load(&dir, None).expect("failed to load world");

    println!(
        "Texture refs: {} unique .ddx (eagerly discovered from UGX material chunks)",
        world.manifest.texture_refs.len()
    );

    assert!(
        world.manifest.texture_refs.len() > 100,
        "expected >100 texture refs from UGX materials, got {}",
        world.manifest.texture_refs.len()
    );

    for (i, path) in world.manifest.texture_refs.iter().enumerate() {
        if i >= 5 {
            break;
        }
        println!("  {path}");
    }
}

#[test]
fn cross_reference_diagnostics() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let world = pipeline::hw1::World::load(&dir, None).expect("failed to load world");
    let report = pipeline::hw1::validate_world(&world);

    println!("Cross-reference diagnostics:");
    report.print_summary();

    // Real game data should have very few (if any) errors
    let errors = report.errors();
    assert!(
        errors == 0,
        "expected 0 errors on clean game data, got {errors}"
    );
}

#[test]
fn inject_bad_reference_produces_diagnostic() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let mut world = pipeline::hw1::World::load(&dir, None).expect("failed to load world");

    // Inject a bad visual reference
    {
        let mut objects = world.objects_mut();
        objects[0].visual = Some("nonexistent_visual_12345".to_string());
    }

    // Inject a bad squad → object reference
    {
        let mut squads = world.squads_mut();
        if let Some(ref mut units) = squads[0].units {
            units.entries[0].proto_object = "nonexistent_object_12345".to_string();
        }
    }

    let report = pipeline::hw1::validate_world(&world);

    println!("Injected-error diagnostics:");
    for d in &report.diagnostics {
        println!("  {d}");
    }

    // Should find our injected bad references
    let missing_refs: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.code == pipeline::hw1::DiagnosticCode::MissingReference)
        .collect();

    let has_visual_diag = missing_refs
        .iter()
        .any(|d| d.message.contains("nonexistent_visual_12345"));
    let has_squad_diag = missing_refs
        .iter()
        .any(|d| d.message.contains("nonexistent_object_12345"));

    assert!(has_visual_diag, "should detect bad visual reference");
    assert!(has_squad_diag, "should detect bad squad→object reference");

    println!("\n✅ Injected bad reference diagnostics detected!");
}
