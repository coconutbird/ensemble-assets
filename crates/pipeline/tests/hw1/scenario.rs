//! Scenario-specific integration tests — ERA layering, SCN parsing,
//! scenario-scoped manifest collection, and full end-to-end validation.

use super::hw1_game_dir;

#[test]
fn validate_with_scenario_era() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found");
        return;
    }

    let mut src = pipeline::hw1::loader::load_with_scenario(&dir, "PHXscn01.era");
    let report = pipeline::hw1::validate(&mut src);

    super::print_report(&report);

    assert_eq!(report.missing(), 0, "some database files were not found");
    assert!(
        report.passed() >= 7,
        "expected at least 7 files to pass, got {}",
        report.passed()
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

    // The refactored pipeline auto-wires scenario loading.
    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");
    world.swap_scenario(&mut src, "PHXscn01");

    world.print_summary();

    // Scenario descriptor should be auto-detected
    assert!(
        world.scenario.is_some(),
        "scenario descriptor should be auto-populated"
    );
    // Scenario data (parsed SCN) should be populated
    assert!(
        world.scenario_data.is_some(),
        "scenario_data should be auto-populated"
    );

    let scn = world.scenario_data.as_ref().unwrap();
    assert!(!scn.objects().is_empty(), "SCN should have placed objects");
    assert!(!scn.players().is_empty(), "SCN should have players");
    assert!(!scn.terrain().is_empty(), "SCN should reference terrain");
}

#[test]
fn dump_scn_structure() {
    let dir = match hw1_game_dir() {
        Some(d) => d,
        None => {
            eprintln!("SKIP");
            return;
        }
    };
    let mut src = pipeline::hw1::loader::load_with_scenario(&dir, "PHXscn01.era");
    let list = pipeline::hw1::scenario::ScenarioList::load(&mut src);

    for desc in list.scenarios.values() {
        let candidates = [format!("scenario\\{}", desc.file), desc.file.clone()];
        for scn_path in &candidates {
            if let Some(data) = src.resolve_data(scn_path) {
                println!(
                    "\n=== {} === ({scn_path}, {} bytes)",
                    desc.name(),
                    data.len()
                );
                if let Ok(doc) = xmb::Reader::read(&data) {
                    if let Some(root) = doc.root() {
                        dump_node(root, 0);
                    }
                } else {
                    println!(
                        "  (not XMB, first 16 bytes: {:02x?})",
                        &data[..16.min(data.len())]
                    );
                }
                return;
            }
        }
    }
    println!("No .scn found in ERA");
}

fn dump_node(node: &bdt::Node, depth: usize) {
    let indent = "  ".repeat(depth);
    print!("{indent}<{}", node.name);
    for attr in &node.attributes {
        let v = attr.value_string();
        let v = if v.len() > 80 {
            format!("{}...", &v[..77])
        } else {
            v
        };
        print!(" {}=\"{}\"", attr.name, v);
    }
    let has_text = !matches!(node.text, bdt::Variant::Null);
    if node.children.is_empty() && !has_text {
        println!("/>");
    } else {
        println!(">");
        if has_text {
            println!("{indent}  {}", node.text.to_string_value());
        }
        let max = if depth == 0 {
            200
        } else if depth == 1 {
            3
        } else {
            2
        };
        for (i, c) in node.children.iter().enumerate() {
            if i >= max {
                println!("{indent}  ... +{} more", node.children.len() - max);
                break;
            }
            dump_node(c, depth + 1);
        }
        println!("{indent}</{}>", node.name);
    }
}

#[test]
fn read_scenario_scn() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let mut src = pipeline::hw1::loader::load_with_scenario(&dir, "PHXscn01.era");
    let list = pipeline::hw1::scenario::ScenarioList::load(&mut src);

    let desc = list
        .scenarios
        .values()
        .find(|d| d.name() == "PHXscn01")
        .expect("PHXscn01 not found in scenario list");

    let scene = desc
        .read_scenario(&mut src)
        .expect("failed to read scenario");

    println!("Scenario: {}", desc.name());
    println!("  Terrain:      '{}'", scene.terrain());
    println!("  TerrainEnv:   '{}'", scene.terrain_env());
    println!("  Lightset:     '{}'", scene.lightset());
    println!("  Pathing:      '{}'", scene.pathing());
    println!("  Minimap:      '{}'", scene.minimap_texture());
    println!("  Placement:    '{}'", scene.player_placement_type());
    println!("  Objects:      {}", scene.objects().len());
    println!("  Players:      {}", scene.players().len());
    println!("  Positions:    {}", scene.positions().len());
    println!("  Cinematics:   {}", scene.cinematics().len());
    println!("  TalkingHeads: {}", scene.talking_heads().len());
    println!("  Objectives:   {}", scene.objectives().len());

    assert!(
        !scene.terrain().is_empty(),
        "terrain name should not be empty"
    );
    assert!(!scene.objects().is_empty(), "should have placed objects");
    assert!(!scene.players().is_empty(), "should have players");
    assert!(!scene.positions().is_empty(), "should have positions");

    let first = &scene.objects()[0];
    assert!(!first.proto_name.is_empty(), "object proto name empty");
    assert_ne!(first.id, 0, "object ID should not be zero");
    println!(
        "  First object: '{}' ID={} player={} pos={:?}",
        first.proto_name, first.id, first.player, first.position
    );

    let p1 = &scene.players()[0];
    assert!(!p1.name.is_empty(), "player name empty");
    assert!(!p1.civ.is_empty(), "player civ empty");
    println!(
        "  First player: '{}' civ={} team={} supplies={}",
        p1.name, p1.civ, p1.team, p1.supplies
    );
}

#[test]
fn scenario_manifest_collection() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found");
        return;
    }

    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");
    world.swap_scenario(&mut src, "PHXscn01.era");

    let scn = world
        .scenario_data
        .as_ref()
        .expect("scenario_data should be auto-populated");

    println!("Scenario manifest (auto-collected):");
    println!("  Terrain refs:     {}", world.manifest.terrain_refs.len());
    println!("  Lightset refs:    {}", world.manifest.lightset_refs.len());
    println!(
        "  Cinematic refs:   {}",
        world.manifest.cinematic_refs.len()
    );
    println!(
        "  TalkingHead refs: {}",
        world.manifest.talking_head_refs.len()
    );
    println!(
        "  Sound bank refs:  {}",
        world.manifest.sound_bank_refs.len()
    );
    println!("  Sky ref:          {:?}", world.manifest.sky_ref);
    println!("  TerrainEnv ref:   {:?}", world.manifest.terrain_env_ref);
    println!("  Minimap ref:      {:?}", world.manifest.minimap_ref);
    println!("  SCN sky:          '{}'", scn.sky());
    println!("  SCN lightsets:    {:?}", scn.lightsets());
    println!("  SCN sound_banks:  {:?}", scn.sound_banks());
    println!("  SCN sim_bounds:   {:?}", scn.sim_bounds());

    assert!(
        world.manifest.terrain_refs.len() >= 2,
        "expected at least .xtd and .xtt terrain refs"
    );
    assert!(
        !world.manifest.lightset_refs.is_empty(),
        "expected at least one lightset ref"
    );
    assert!(
        !world.manifest.cinematic_refs.is_empty(),
        "expected cinematic refs for campaign scenario"
    );
    assert!(
        !world.manifest.talking_head_refs.is_empty(),
        "expected talking head refs for campaign scenario"
    );
}

#[test]
fn full_scenario_validation() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found");
        return;
    }

    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");
    world.swap_scenario(&mut src, "PHXscn01");

    world.print_summary();

    let desc = world
        .scenario
        .as_ref()
        .expect("scenario descriptor missing");
    let scn = world.scenario_data.as_ref().expect("scenario data missing");
    println!("\n=== Full Scenario Validation ===");
    println!("Scenario: {} ({})", desc.name(), desc.file);

    assert!(!scn.objects().is_empty(), "SCN should have placed objects");
    assert!(!scn.players().is_empty(), "SCN should have players");
    assert!(!scn.terrain().is_empty(), "SCN should reference terrain");
    println!("  Placed objects: {}", scn.objects().len());
    println!("  Players:        {}", scn.players().len());
    println!("  Terrain:        '{}'", scn.terrain());

    let total_preload = world.manifest.preload_vis_refs.len()
        + world.manifest.preload_pfx_refs.len()
        + world.manifest.preload_tfx_refs.len();
    println!("  Preload entries: {} total", total_preload);
    assert!(
        !world.manifest.preload_vis_refs.is_empty(),
        "vis preload list should have entries for campaign scenario"
    );

    assert!(
        world.manifest.terrain_refs.len() >= 2,
        "expected at least .xtd and .xtt terrain refs, got {}",
        world.manifest.terrain_refs.len()
    );
    for tr in &world.manifest.terrain_refs {
        println!("  Terrain ref: {tr}");
    }

    assert!(!world.manifest.lightset_refs.is_empty(), "lightset refs");
    assert!(!world.manifest.cinematic_refs.is_empty(), "cinematic refs");
    assert!(
        !world.manifest.talking_head_refs.is_empty(),
        "talking head refs"
    );

    assert!(
        world.manifest.texture_refs.len() > 600,
        "expected 600+ texture refs, got {}",
        world.manifest.texture_refs.len()
    );

    let (base_world, _base_src) =
        pipeline::hw1::World::load(&dir).expect("failed to load base world");

    let scenario_models = world.manifest.model_refs.len();
    let base_models = base_world.manifest.model_refs.len();
    println!(
        "\n  Models: {} (scenario) vs {} (base)",
        scenario_models, base_models
    );
    assert!(
        scenario_models >= base_models,
        "scenario should have at least as many models as base"
    );

    let scenario_textures = world.manifest.texture_refs.len();
    let base_textures = base_world.manifest.texture_refs.len();
    println!(
        "  Textures: {} (scenario) vs {} (base)",
        scenario_textures, base_textures
    );
    assert!(
        scenario_textures >= base_textures,
        "scenario should have at least as many textures as base"
    );

    println!("\n✅ Full scenario validation passed!");
}

#[test]
fn terrain_textures_from_xtt() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");
    world.swap_scenario(&mut src, "PHXscn01.era");

    // Terrain refs should include at least one .xtt file
    let xtt_refs: Vec<_> = world
        .manifest
        .terrain_refs
        .iter()
        .filter(|p| p.ends_with(".xtt"))
        .collect();
    assert!(
        !xtt_refs.is_empty(),
        "expected at least one .xtt terrain ref"
    );
    println!("  XTT refs: {:?}", xtt_refs);

    // Terrain textures should be discovered — look for typical splat patterns
    let terrain_textures: Vec<_> = world
        .manifest
        .texture_refs
        .iter()
        .filter(|t| t.contains("_df.ddx") || t.contains("_nm.ddx") || t.contains("_sp.ddx"))
        .filter(|t| {
            // Terrain textures typically live in paths without unit/building names
            !t.contains("\\unit\\") && !t.contains("\\building\\")
        })
        .collect();

    println!(
        "  Terrain-style textures discovered: {}",
        terrain_textures.len()
    );
    for t in terrain_textures.iter().take(10) {
        println!("    {t}");
    }

    // With XTT parsing, we should discover splat/foliage textures
    assert!(
        terrain_textures.len() > 5,
        "expected terrain textures from XTT, got {}",
        terrain_textures.len()
    );

    println!("\n✅ Terrain texture discovery passed!");
}

#[test]
fn edit_save_round_trip() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found");
        return;
    }

    // 1. Load world
    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");
    world.swap_scenario(&mut src, "PHXscn01");

    // Set up a temp override directory
    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    src.set_override_dir(tmp.path());

    // Should start clean
    assert!(!world.is_dirty(), "world should not be dirty after load");

    // 2. Mutate database — change an object's hitpoints
    let original_hp = world.database.objects[0].hitpoints;
    let original_name = world.database.objects[0].name.clone();
    {
        let mut objects = world.objects_mut();
        objects[0].hitpoints = Some(99999.0);
    }
    assert!(world.is_dirty(), "world should be dirty after mutation");
    assert_eq!(
        world.dirty_tables(),
        vec![pipeline::hw1::TableId::Objects],
        "only Objects should be dirty"
    );

    // 3. Mutate scenario — change a placed object position
    let original_scn_obj_count = world.scenario_data.as_ref().unwrap().objects().len();
    {
        let mut scn = world.scenario_data_mut();
        if let Some(ref mut scn_data) = *scn
            && let Some(ref mut objects_wrapper) = scn_data.objects
        {
            objects_wrapper.entries[0].position = "100.0,200.0,300.0".to_string();
        }
    }
    let dirty = world.dirty_tables();
    assert!(dirty.contains(&pipeline::hw1::TableId::Objects));
    assert!(dirty.contains(&pipeline::hw1::TableId::Scenario));

    // 4. Save — should write only Objects + Scenario
    let written = world.save(&src).expect("save failed");
    assert!(!written.is_empty(), "should have written files");
    println!("Written {} files:", written.len());
    for p in &written {
        println!("  {}", p.display());
    }

    // Should be clean after save
    assert!(!world.is_dirty(), "world should be clean after save");

    // 5. Verify the override files exist on disk
    for p in &written {
        assert!(p.exists(), "written file should exist: {}", p.display());
    }

    // 6. Reload from the same source (override dir has priority)
    let world2 = pipeline::hw1::World::load_from_source(&mut src).expect("failed to reload world");

    // Verify the mutated hitpoints persisted
    let reloaded_obj = world2
        .database
        .objects
        .iter()
        .find(|o| o.name == original_name)
        .expect("original object should exist in reloaded world");
    assert_eq!(
        reloaded_obj.hitpoints,
        Some(99999.0),
        "hitpoints should be 99999 after reload (was {original_hp:?})"
    );

    // Verify the scenario mutation persisted
    let reloaded_scn = world2
        .scenario_data
        .as_ref()
        .expect("reloaded scenario_data");
    assert_eq!(
        reloaded_scn.objects().len(),
        original_scn_obj_count,
        "SCN object count should be preserved"
    );
    assert_eq!(
        reloaded_scn.objects()[0].position,
        "100.0,200.0,300.0",
        "SCN object position should be updated"
    );

    println!("\n✅ Edit-save round-trip passed!");
}

#[test]
fn terrain_edit_save_round_trip() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found");
        return;
    }

    // 1. Load world with scenario — terrain should be eagerly loaded
    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");
    world.swap_scenario(&mut src, "PHXscn01");

    let tmp = tempfile::tempdir().expect("failed to create tempdir");
    src.set_override_dir(tmp.path());

    assert!(
        world.terrain_data.is_some(),
        "terrain data should be eagerly loaded for scenario"
    );
    assert!(
        world.terrain_textures.is_some(),
        "terrain textures should be eagerly loaded for scenario"
    );
    assert!(!world.is_dirty(), "world should start clean");

    // 2. Record original XTD header values
    let original_tile_scale = world.terrain_data.as_ref().unwrap().header.tile_scale;
    let original_world_min_y = world.terrain_data.as_ref().unwrap().header.world_min[1];
    println!("  Original tile_scale: {original_tile_scale}, world_min.y: {original_world_min_y}");

    // 3. Mutate terrain data via DirtyGuard
    {
        let mut xtd = world.terrain_data_mut();
        if let Some(ref mut xtd_file) = *xtd {
            xtd_file.header.tile_scale = 42.0;
            xtd_file.header.world_min[1] = -999.0;
        }
    }
    assert!(world.is_dirty(), "world should be dirty after XTD mutation");
    assert!(
        world
            .dirty_tables()
            .contains(&pipeline::hw1::TableId::TerrainData),
        "TerrainData should be in dirty tables"
    );

    // 4. Mutate terrain textures — change a texture scale
    let original_tex_count = world
        .terrain_textures
        .as_ref()
        .unwrap()
        .active_textures
        .len();
    println!("  Active textures: {original_tex_count}");

    if original_tex_count > 0 {
        let original_u_scale = world.terrain_textures.as_ref().unwrap().active_textures[0].u_scale;
        {
            let mut xtt = world.terrain_textures_mut();
            if let Some(ref mut xtt_file) = *xtt {
                xtt_file.active_textures[0].u_scale = 777;
            }
        }
        assert!(
            world
                .dirty_tables()
                .contains(&pipeline::hw1::TableId::TerrainTextures),
            "TerrainTextures should be in dirty tables"
        );
        println!("  Changed u_scale from {original_u_scale} to 777");
    }

    // 5. Save via save() — should write both XTD and XTT
    let written = world.save(&src).expect("save failed");
    assert!(!written.is_empty(), "should have written terrain files");
    println!("  Written {} files:", written.len());
    for p in &written {
        println!("    {}", p.display());
    }
    assert!(!world.is_dirty(), "world should be clean after save");

    // 6. Verify files exist on disk
    let xtd_files: Vec<_> = written
        .iter()
        .filter(|p| p.to_string_lossy().contains(".xtd"))
        .collect();
    let xtt_files: Vec<_> = written
        .iter()
        .filter(|p| p.to_string_lossy().contains(".xtt"))
        .collect();
    assert_eq!(xtd_files.len(), 1, "should have written exactly one .xtd");
    assert!(
        xtd_files[0].exists(),
        "XTD file should exist: {}",
        xtd_files[0].display()
    );

    if original_tex_count > 0 {
        assert_eq!(xtt_files.len(), 1, "should have written exactly one .xtt");
        assert!(
            xtt_files[0].exists(),
            "XTT file should exist: {}",
            xtt_files[0].display()
        );
    }

    // 7. Re-read the saved XTD and verify mutations persisted
    let saved_xtd_bytes = std::fs::read(xtd_files[0]).expect("read saved XTD");
    let saved_xtd = xtd::Reader::read(&saved_xtd_bytes).expect("parse saved XTD");
    assert_eq!(
        saved_xtd.header.tile_scale, 42.0,
        "saved XTD tile_scale should be 42.0"
    );
    assert_eq!(
        saved_xtd.header.world_min[1], -999.0,
        "saved XTD world_min.y should be -999.0"
    );

    // 8. Per-file save: mutate + save_terrain_data individually
    {
        let mut xtd = world.terrain_data_mut();
        if let Some(ref mut xtd_file) = *xtd {
            xtd_file.header.tile_scale = 123.0;
        }
    }
    let xtd_path = world
        .save_terrain_data(&src)
        .expect("save_terrain_data failed");
    assert!(xtd_path.exists(), "per-file XTD save should create file");
    assert!(!world.is_dirty(), "should be clean after per-file save");

    let re_read_bytes = std::fs::read(&xtd_path).expect("read re-saved XTD");
    let re_read = xtd::Reader::read(&re_read_bytes).expect("parse re-saved XTD");
    assert_eq!(
        re_read.header.tile_scale, 123.0,
        "per-file saved XTD tile_scale should be 123.0"
    );

    println!("\n✅ Terrain edit-save round-trip passed!");
}

#[test]
fn clear_scenario_unloads_state() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found");
        return;
    }

    // 1. Load world with scenario
    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");
    world.swap_scenario(&mut src, "PHXscn01");

    // Verify scenario is loaded
    assert!(world.scenario.is_some(), "scenario should be loaded");
    assert!(
        world.scenario_data.is_some(),
        "scenario data should be loaded"
    );
    assert!(
        world.terrain_data.is_some(),
        "terrain data should be loaded"
    );
    assert!(
        !world.manifest.terrain_refs.is_empty(),
        "terrain refs should be populated"
    );

    let era_count_before = src.era_count();
    let db_objects = world.database.objects.len();
    let visuals_count = world.visuals.len();
    let tactics_count = world.tactics.len();

    // 2. Clear the scenario (pop ERA)
    world.clear_scenario(&mut src);

    // 3. Scenario state should be gone
    assert!(world.scenario.is_none(), "scenario should be cleared");
    assert!(
        world.scenario_data.is_none(),
        "scenario data should be cleared"
    );
    assert!(
        world.terrain_data.is_none(),
        "terrain data should be cleared"
    );
    assert!(
        world.terrain_textures.is_none(),
        "terrain textures should be cleared"
    );
    assert!(
        world.manifest.terrain_refs.is_empty(),
        "terrain refs should be cleared"
    );
    assert!(
        world.manifest.lightset_refs.is_empty(),
        "lightset refs should be cleared"
    );

    // 4. Base-game state should be preserved
    assert_eq!(
        world.database.objects.len(),
        db_objects,
        "database should be unchanged"
    );
    assert_eq!(
        world.visuals.len(),
        visuals_count,
        "visuals should be unchanged"
    );
    assert_eq!(
        world.tactics.len(),
        tactics_count,
        "tactics should be unchanged"
    );

    // 5. ERA count should have decreased by 1
    assert_eq!(
        src.era_count(),
        era_count_before - 1,
        "should have popped one ERA"
    );

    println!("\n✅ clear_scenario passed!");
}

#[test]
fn swap_scenario_reloads_state() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let scenario_path = format!("{dir}/PHXscn01.era");
    if !std::path::Path::new(&scenario_path).exists() {
        eprintln!("SKIP: PHXscn01.era not found");
        return;
    }

    // 1. Load world with scenario
    let (mut world, mut src) = pipeline::hw1::World::load(&dir).expect("failed to load world");
    world.swap_scenario(&mut src, "PHXscn01");

    let era_count_before = src.era_count();
    let db_objects = world.database.objects.len();
    let visuals_count = world.visuals.len();

    // Record original scenario name
    let orig_scenario_name = world
        .scenario
        .as_ref()
        .expect("scenario should exist")
        .name()
        .to_string();

    // 2. Swap to the same scenario (only one guaranteed to exist)
    world.swap_scenario(&mut src, "PHXscn01");

    // 3. Scenario should be re-loaded
    assert!(
        world.scenario.is_some(),
        "scenario should be re-loaded after swap"
    );
    assert!(
        world.scenario_data.is_some(),
        "scenario data should be re-loaded after swap"
    );
    assert!(
        world.terrain_data.is_some(),
        "terrain data should be re-loaded after swap"
    );
    assert_eq!(
        world.scenario.as_ref().unwrap().name(),
        orig_scenario_name,
        "scenario name should match after swap"
    );

    // 4. Manifest should be re-populated
    assert!(
        !world.manifest.terrain_refs.is_empty(),
        "terrain refs should be re-populated after swap"
    );

    // 5. Base-game state should still be preserved
    assert_eq!(
        world.database.objects.len(),
        db_objects,
        "database should be unchanged after swap"
    );
    assert_eq!(
        world.visuals.len(),
        visuals_count,
        "visuals should be unchanged after swap"
    );

    // 6. ERA count should be the same (popped old, pushed new)
    assert_eq!(
        src.era_count(),
        era_count_before,
        "ERA count should be unchanged after swap"
    );

    println!("\n✅ swap_scenario passed!");
}
