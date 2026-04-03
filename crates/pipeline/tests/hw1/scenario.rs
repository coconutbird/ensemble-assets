//! Scenario-specific integration tests — ERA layering, SCN parsing,
//! scenario-scoped manifest collection, and full end-to-end validation.

use super::hw1_game_dir;

// ── Validation with scenario ERA ─────────────────────────────────────

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

// ── World loading with scenario ──────────────────────────────────────

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
    let world = pipeline::hw1::World::load(&dir, Some("PHXscn01"))
        .expect("failed to load world with scenario");

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
    assert!(!scn.objects.is_empty(), "SCN should have placed objects");
    assert!(!scn.players.is_empty(), "SCN should have players");
    assert!(!scn.terrain.is_empty(), "SCN should reference terrain");
}

// ── SCN structure dump ───────────────────────────────────────────────

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

    for (_name, desc) in &list.scenarios {
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

// ── Scenario (SCN) parsing tests ─────────────────────────────────────

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
    println!("  Terrain:      '{}'", scene.terrain);
    println!("  TerrainEnv:   '{}'", scene.terrain_env);
    println!("  Lightset:     '{}'", scene.lightset);
    println!("  Pathing:      '{}'", scene.pathing);
    println!("  Minimap:      '{}'", scene.minimap_texture);
    println!("  Placement:    '{}'", scene.player_placement_type);
    println!("  Objects:      {}", scene.objects.len());
    println!("  Players:      {}", scene.players.len());
    println!("  Positions:    {}", scene.positions.len());
    println!("  Cinematics:   {}", scene.cinematics.len());
    println!("  TalkingHeads: {}", scene.talking_heads.len());
    println!("  Objectives:   {}", scene.objectives.len());

    assert!(
        !scene.terrain.is_empty(),
        "terrain name should not be empty"
    );
    assert!(!scene.objects.is_empty(), "should have placed objects");
    assert!(!scene.players.is_empty(), "should have players");
    assert!(!scene.positions.is_empty(), "should have positions");

    let first = &scene.objects[0];
    assert!(!first.proto_name.is_empty(), "object proto name empty");
    assert_ne!(first.id, 0, "object ID should not be zero");
    println!(
        "  First object: '{}' ID={} player={} pos={:?}",
        first.proto_name, first.id, first.player, first.position
    );

    let p1 = &scene.players[0];
    assert!(!p1.name.is_empty(), "player name empty");
    assert!(!p1.civ.is_empty(), "player civ empty");
    println!(
        "  First player: '{}' civ={} team={} supplies={}",
        p1.name, p1.civ, p1.team, p1.supplies
    );
}

// ── Scenario asset collection test ───────────────────────────────────

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

    let world = pipeline::hw1::World::load(&dir, Some("PHXscn01.era"))
        .expect("failed to load world with scenario");

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
    println!("  SCN sky:          '{}'", scn.sky);
    println!("  SCN lightsets:    {:?}", scn.lightsets);
    println!("  SCN sound_banks:  {:?}", scn.sound_banks);
    println!("  SCN sim_bounds:   {:?}", scn.sim_bounds);

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

// ── Full scenario validation test ────────────────────────────────────

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

    let world = pipeline::hw1::World::load(&dir, Some("PHXscn01"))
        .expect("failed to load world with scenario");

    world.print_summary();

    // ── 1. Scenario auto-detection ──
    let desc = world
        .scenario
        .as_ref()
        .expect("scenario descriptor missing");
    let scn = world.scenario_data.as_ref().expect("scenario data missing");
    println!("\n=== Full Scenario Validation ===");
    println!("Scenario: {} ({})", desc.name(), desc.file);

    // ── 2. SCN content ──
    assert!(!scn.objects.is_empty(), "SCN should have placed objects");
    assert!(!scn.players.is_empty(), "SCN should have players");
    assert!(!scn.terrain.is_empty(), "SCN should reference terrain");
    println!("  Placed objects: {}", scn.objects.len());
    println!("  Players:        {}", scn.players.len());
    println!("  Terrain:        '{}'", scn.terrain);

    // ── 3. Manifest: preload lists ──
    let total_preload = world.manifest.preload_vis_refs.len()
        + world.manifest.preload_pfx_refs.len()
        + world.manifest.preload_tfx_refs.len();
    println!("  Preload entries: {} total", total_preload);
    assert!(
        !world.manifest.preload_vis_refs.is_empty(),
        "vis preload list should have entries for campaign scenario"
    );

    // ── 4. Manifest: terrain ──
    assert!(
        world.manifest.terrain_refs.len() >= 2,
        "expected at least .xtd and .xtt terrain refs, got {}",
        world.manifest.terrain_refs.len()
    );
    for tr in &world.manifest.terrain_refs {
        println!("  Terrain ref: {tr}");
    }

    // ── 5. Manifest: SCN-level assets ──
    assert!(!world.manifest.lightset_refs.is_empty(), "lightset refs");
    assert!(!world.manifest.cinematic_refs.is_empty(), "cinematic refs");
    assert!(
        !world.manifest.talking_head_refs.is_empty(),
        "talking head refs"
    );

    // ── 6. Texture discovery still works ──
    assert!(
        world.manifest.texture_refs.len() > 600,
        "expected 600+ texture refs, got {}",
        world.manifest.texture_refs.len()
    );

    // ── 7. Scenario ERA should bring MORE assets than base-only ──
    let base_world = pipeline::hw1::World::load(&dir, None).expect("failed to load base world");

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

// ── Terrain texture discovery ───────────────────────────────────────

#[test]
fn terrain_textures_from_xtt() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let world = pipeline::hw1::World::load(&dir, Some("PHXscn01.era"))
        .expect("failed to load world with scenario");

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
