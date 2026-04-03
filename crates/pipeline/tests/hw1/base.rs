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

    let (world, _src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

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

    let (world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

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

    let (world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

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

    let (world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

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

    let (world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

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

    let (world, _src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

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

    let (world, _src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

    // In-memory cross-reference checks
    let report = pipeline::hw1::validate_world(&world);
    println!("Cross-reference diagnostics:");
    report.print_summary();

    let errors = report.errors();
    assert!(
        errors == 0,
        "expected 0 errors on clean game data, got {errors}"
    );

    // Print a few representative warnings/info so we can see the new checks working
    let by_code = |code: pipeline::hw1::DiagnosticCode| {
        report.diagnostics.iter().filter(|d| d.code == code).count()
    };
    println!(
        "  DuplicateName:     {}",
        by_code(pipeline::hw1::DiagnosticCode::DuplicateName)
    );
    println!(
        "  MissingReference:  {}",
        by_code(pipeline::hw1::DiagnosticCode::MissingReference)
    );

    // Asset existence checks (needs AssetSource)
    let src = load_hw1(&dir);
    let asset_report = pipeline::hw1::validate_world_assets(&world, &src);
    println!("\nAsset existence diagnostics:");
    asset_report.print_summary();

    let asset_errors = asset_report.errors();
    assert!(
        asset_errors == 0,
        "expected 0 asset errors on clean game data, got {asset_errors}"
    );
    println!(
        "  MissingAsset:      {}",
        asset_report
            .diagnostics
            .iter()
            .filter(|d| d.code == pipeline::hw1::DiagnosticCode::MissingAsset)
            .count()
    );
}

#[test]
fn inject_bad_reference_produces_diagnostic() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let (mut world, _src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

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

    // Inject a bad leader → civ reference
    {
        let mut leaders = world.leaders_mut();
        if !leaders.is_empty() {
            leaders[0].civ = Some("nonexistent_civ_99999".to_string());
        }
    }

    // Inject a bad tactics weapon → weapon_type reference
    {
        let mut tactics = world.tactics_mut();
        if let Some((_key, tac)) = tactics.iter_mut().next()
            && !tac.weapons.is_empty()
        {
            tac.weapons[0].weapon_type = Some("nonexistent_wt_99999".to_string());
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
    let has_leader_civ_diag = missing_refs
        .iter()
        .any(|d| d.message.contains("nonexistent_civ_99999"));
    let has_tactics_wt_diag = missing_refs
        .iter()
        .any(|d| d.message.contains("nonexistent_wt_99999"));

    assert!(has_visual_diag, "should detect bad visual reference");
    assert!(has_squad_diag, "should detect bad squad→object reference");
    assert!(
        has_leader_civ_diag,
        "should detect bad leader→civ reference"
    );
    assert!(
        has_tactics_wt_diag,
        "should detect bad tactics weapon→weapon_type reference"
    );

    println!("\n✅ All injected bad reference diagnostics detected!");
}

#[test]
fn per_file_save_visual_tactics_physics() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    // 1. Load world
    let (mut world, src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

    // Set up a temp override directory
    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    let src = {
        let mut s = src;
        s.set_override_dir(tmp.path());
        s
    };

    assert!(!world.is_dirty(), "world should start clean");

    // 2. Find an object that has a visual, tactics, and physics.
    let test_obj = world
        .assets
        .iter()
        .find(|(name, oa)| {
            oa.visual.is_some()
                && oa.tactics.is_some()
                && oa.physics.is_some()
                && world.visuals.contains_key(name.as_str())
                && world.tactics.contains_key(name.as_str())
                && world.physics.contains_key(name.as_str())
        })
        .map(|(name, _)| name.clone());

    let obj_name = match test_obj {
        Some(n) => n,
        None => {
            eprintln!("SKIP: no object with visual+tactics+physics found");
            return;
        }
    };
    println!("Test object: {obj_name}");

    // Record original values for round-trip verification
    let vis_path = world.assets[&obj_name].visual.clone().unwrap();
    let tac_path = world.assets[&obj_name].tactics.clone().unwrap();
    let phys_path = world.assets[&obj_name].physics.clone().unwrap();
    println!("  visual:  {vis_path}");
    println!("  tactics: {tac_path}");
    println!("  physics: {phys_path}");

    // --- Test per-key dirty tracking ---

    // 3a. Mutate a single visual via visual_mut()
    {
        let mut vis = world
            .visual_mut(&obj_name)
            .expect("visual_mut should succeed");
        // Just touch it — the KeyDirtyGuard marks the key on drop.
        vis.default_model = vis.default_model.clone();
    }
    assert!(world.is_dirty(), "should be dirty after visual_mut");
    let dirty = world.dirty_tables();
    assert_eq!(
        dirty,
        vec![pipeline::hw1::TableId::Visuals],
        "only Visuals should be dirty"
    );

    // 3b. save_visual should write just that one file
    let written_path = world
        .save_visual(&obj_name, &src)
        .expect("save_visual failed");
    assert!(
        written_path.exists(),
        "saved visual should exist: {}",
        written_path.display()
    );
    println!("  wrote visual: {}", written_path.display());

    // After save_visual, should be clean (no other dirty keys)
    assert!(
        !world.is_dirty(),
        "should be clean after save_visual (single key)"
    );

    // 4. Mutate tactics via tactic_mut()
    {
        let mut tac = world
            .tactic_mut(&obj_name)
            .expect("tactic_mut should succeed");
        tac.weapons = tac.weapons.clone();
    }
    assert!(world.is_dirty(), "should be dirty after tactic_mut");

    let written_path = world
        .save_tactic(&obj_name, &src)
        .expect("save_tactic failed");
    assert!(
        written_path.exists(),
        "saved tactics should exist: {}",
        written_path.display()
    );
    println!("  wrote tactics: {}", written_path.display());
    assert!(!world.is_dirty(), "should be clean after save_tactic");

    // 5. Mutate physics via physics_entry_mut()
    {
        let mut phys = world
            .physics_entry_mut(&obj_name)
            .expect("physics_entry_mut should succeed");
        phys.physics.vehicle = phys.physics.vehicle.clone();
    }
    assert!(world.is_dirty(), "should be dirty after physics_entry_mut");

    let written_paths = world
        .save_physics(&obj_name, &src)
        .expect("save_physics failed");
    assert!(
        !written_paths.is_empty(),
        "save_physics should write at least one file"
    );
    for p in &written_paths {
        assert!(
            p.exists(),
            "saved physics file should exist: {}",
            p.display()
        );
        println!("  wrote physics: {}", p.display());
    }
    assert!(!world.is_dirty(), "should be clean after save_physics");

    // --- Test selective save() with per-key tracking ---

    // 6. Dirty two specific visuals (if a second one exists)
    let second_obj = world
        .assets
        .iter()
        .find(|(name, oa)| {
            *name != &obj_name && oa.visual.is_some() && world.visuals.contains_key(name.as_str())
        })
        .map(|(name, _)| name.clone());

    {
        let mut vis = world.visual_mut(&obj_name).unwrap();
        vis.default_model = vis.default_model.clone();
    }
    if let Some(ref second) = second_obj {
        let mut vis = world.visual_mut(second).unwrap();
        vis.default_model = vis.default_model.clone();
    }

    // save() should only write dirty keys, not all visuals
    let written = world.save(&src).expect("save failed");
    let expected_count = if second_obj.is_some() { 2 } else { 1 };
    assert_eq!(
        written.len(),
        expected_count,
        "save() should write only {} dirty visual(s), not all {}",
        expected_count,
        world.visuals.len()
    );
    for p in &written {
        assert!(p.exists(), "written file should exist: {}", p.display());
    }
    assert!(!world.is_dirty(), "should be clean after save()");

    println!("\n✅ Per-file asset save passed!");
}

#[test]
fn string_table_edit_save_round_trip() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

    // Load strings if not already present
    if world.strings.is_none() {
        world.strings = pipeline::hw1::stringtable::StringTable::load("en", &mut src);
    }
    let st = world
        .strings
        .as_ref()
        .expect("string table should be loaded");
    let original_count = st.len();
    assert!(original_count > 0, "should have strings");
    println!("Loaded {original_count} strings");

    // Pick a string to modify
    let test_id = *st.strings.keys().next().expect("no string entries");
    let original_text = st.strings[&test_id].text.clone();
    println!("  locID {test_id}: {original_text:?}");

    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    src.set_override_dir(tmp.path());

    assert!(!world.is_dirty(), "world should start clean");

    // Mutate via strings_mut()
    {
        let mut guard = world.strings_mut();
        let st = guard.as_mut().expect("should be Some");
        st.strings.get_mut(&test_id).unwrap().text = "ROUND_TRIP_TEST".to_string();
    }
    assert!(world.is_dirty(), "should be dirty after strings_mut");
    assert!(
        world
            .dirty_tables()
            .contains(&pipeline::hw1::TableId::Strings),
        "Strings should be in dirty tables"
    );

    // Per-file save
    let written_path = world.save_strings(&src).expect("save_strings failed");
    assert!(
        written_path.exists(),
        "saved string table should exist: {}",
        written_path.display()
    );
    assert!(!world.is_dirty(), "should be clean after save_strings");
    println!("  wrote: {}", written_path.display());

    // Re-read the saved XMB and verify the mutation persisted
    let saved_bytes = std::fs::read(&written_path).expect("read saved file");
    let doc = pipeline::xmb::Document::from_bytes(&saved_bytes).expect("parse saved XMB");
    let root = doc.root().expect("should have root node");
    // Find the Language node, then the String node with our locID
    let lang_node = root
        .children
        .iter()
        .find(|n| n.name == "Language")
        .expect("should have Language node");
    assert_eq!(
        lang_node.children.len(),
        original_count,
        "string count should match"
    );
    let target = lang_node
        .children
        .iter()
        .find(|n| {
            n.attributes
                .iter()
                .any(|a| a.name == "_locID" && a.value.to_string_value() == test_id.to_string())
        })
        .expect("should find mutated string node");
    assert_eq!(
        target.text.to_string_value(),
        "ROUND_TRIP_TEST",
        "mutated string should persist in saved XMB"
    );

    // Also verify that read_xmb resolves the override file
    // (unpacked .xml path should fall back to .xml.xmb in override dir)
    let reloaded_doc = src
        .read_xmb("data\\stringtable-en.xml")
        .expect("read_xmb should find the override .xmb file");
    let reloaded_root = reloaded_doc.root().expect("reloaded root");
    let reloaded_lang = reloaded_root
        .children
        .iter()
        .find(|n| n.name == "Language")
        .expect("reloaded Language node");
    let reloaded_target = reloaded_lang
        .children
        .iter()
        .find(|n| {
            n.attributes
                .iter()
                .any(|a| a.name == "_locID" && a.value.to_string_value() == test_id.to_string())
        })
        .expect("should find mutated string in reloaded XMB");
    assert_eq!(
        reloaded_target.text.to_string_value(),
        "ROUND_TRIP_TEST",
        "read_xmb override resolution should return the mutated string"
    );

    println!("\n✅ String table edit-save round-trip passed!");
}

#[test]
fn model_edit_save_round_trip() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

    // Find an object with a resolvable model
    let model_path = world
        .assets
        .values()
        .flat_map(|a| a.models.iter())
        .find(|m| src.resolve_exact(m).is_some())
        .cloned();

    let model_path = match model_path {
        Some(p) => p,
        None => {
            eprintln!("SKIP: no resolvable model found");
            return;
        }
    };
    println!("Testing model: {model_path}");

    // Load into cache
    let geom = world
        .load_model(&model_path, &mut src)
        .expect("load_model failed");
    let original_sections = geom.sections.len();
    let original_bones = geom.bones.len();
    println!("  sections: {original_sections}, bones: {original_bones}");

    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    src.set_override_dir(tmp.path());

    assert!(!world.is_dirty(), "world should start clean");

    // Mutate via model_mut() — just touch it to trigger dirty
    {
        let mut guard = world
            .model_mut(&model_path)
            .expect("model_mut should succeed");
        // Touch the sections to trigger the dirty guard
        guard.sections = guard.sections.clone();
    }
    assert!(world.is_dirty(), "should be dirty after model_mut");
    assert!(
        world
            .dirty_tables()
            .contains(&pipeline::hw1::TableId::Models),
        "Models should be in dirty tables"
    );

    // Per-file save
    let written_path = world
        .save_model(&model_path, &src)
        .expect("save_model failed");
    assert!(
        written_path.exists(),
        "saved model should exist: {}",
        written_path.display()
    );
    assert!(!world.is_dirty(), "should be clean after save_model");
    println!("  wrote: {}", written_path.display());

    // Re-read and verify structure preserved
    let saved_bytes = std::fs::read(&written_path).expect("read saved model");
    let reloaded = pipeline::ugx::UgxGeom::from_bytes(&saved_bytes).expect("parse saved UGX");
    assert_eq!(
        reloaded.sections.len(),
        original_sections,
        "section count should survive round-trip"
    );
    assert_eq!(
        reloaded.bones.len(),
        original_bones,
        "bone count should survive round-trip"
    );

    println!("\n✅ Model (UGX) edit-save round-trip passed!");
}

#[test]
fn texture_edit_save_round_trip() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

    // Find a resolvable texture by iterating objects
    let mut tex_path = None;
    for obj in world.assets.values() {
        let paths = world.resolve_textures_for_obj(obj, &mut src);
        if let Some(p) = paths.into_iter().next() {
            tex_path = Some(p);
            break;
        }
    }

    let tex_path = match tex_path {
        Some(p) => p,
        None => {
            eprintln!("SKIP: no resolvable texture found");
            return;
        }
    };
    println!("Testing texture: {tex_path}");

    // Load into cache
    let tex = world
        .load_texture(&tex_path, &mut src)
        .expect("load_texture failed");
    let original_width = tex.info.width;
    let original_height = tex.info.height;
    println!("  {}x{}", original_width, original_height);

    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    src.set_override_dir(tmp.path());

    assert!(!world.is_dirty(), "world should start clean");

    // Mutate via texture_mut()
    {
        let _guard = world
            .texture_mut(&tex_path)
            .expect("texture_mut should succeed");
        // Just touching triggers dirty on drop
    }
    assert!(world.is_dirty(), "should be dirty after texture_mut");

    // Per-file save
    let written_path = world
        .save_texture(&tex_path, &src)
        .expect("save_texture failed");
    assert!(
        written_path.exists(),
        "saved texture should exist: {}",
        written_path.display()
    );
    assert!(!world.is_dirty(), "should be clean after save_texture");
    println!("  wrote: {}", written_path.display());

    // Re-read and verify dimensions preserved
    let saved_bytes = std::fs::read(&written_path).expect("read saved texture");
    let reloaded = pipeline::ddx::DdxTexture::from_bytes(&saved_bytes).expect("parse saved DDX");
    assert_eq!(
        reloaded.info.width, original_width,
        "width should survive round-trip"
    );
    assert_eq!(
        reloaded.info.height, original_height,
        "height should survive round-trip"
    );

    println!("\n✅ Texture (DDX) edit-save round-trip passed!");
}

#[test]
fn animation_edit_save_round_trip() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");

    // Find an object with a resolvable animation
    let anim_path = world
        .assets
        .values()
        .flat_map(|a| a.anims.iter())
        .find(|p| src.resolve_exact(p).is_some())
        .cloned();

    let anim_path = match anim_path {
        Some(p) => p,
        None => {
            eprintln!("SKIP: no resolvable animation found");
            return;
        }
    };
    println!("Testing animation: {anim_path}");

    // Load into cache
    let anim = world
        .load_animation(&anim_path, &mut src)
        .expect("load_animation failed");
    let original_anim_count = anim.animation_count().unwrap_or(0);
    let original_duration = anim.duration().unwrap_or(0.0);
    println!("  animations: {original_anim_count}, duration: {original_duration:.3}s");

    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    src.set_override_dir(tmp.path());

    assert!(!world.is_dirty(), "world should start clean");

    // Mutate via animation_mut()
    {
        let _guard = world
            .animation_mut(&anim_path)
            .expect("animation_mut should succeed");
        // Just touching triggers dirty on drop
    }
    assert!(world.is_dirty(), "should be dirty after animation_mut");

    // Per-file save
    let written_path = world
        .save_animation(&anim_path, &src)
        .expect("save_animation failed");
    assert!(
        written_path.exists(),
        "saved animation should exist: {}",
        written_path.display()
    );
    assert!(!world.is_dirty(), "should be clean after save_animation");
    println!("  wrote: {}", written_path.display());

    // Re-read and verify
    let saved_bytes = std::fs::read(&written_path).expect("read saved animation");
    let reloaded = pipeline::uax::UaxFile::from_bytes(&saved_bytes).expect("parse saved UAX");
    assert_eq!(
        reloaded.animation_count().unwrap_or(0),
        original_anim_count,
        "animation count should survive round-trip"
    );
    assert!(
        (reloaded.duration().unwrap_or(0.0) - original_duration).abs() < 0.001,
        "duration should survive round-trip"
    );

    println!("\n✅ Animation (UAX) edit-save round-trip passed!");
}
