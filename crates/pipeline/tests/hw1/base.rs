//! Base game integration tests — ERA loading, database, validation,
//! models, animations, textures, and binary asset verification.

use super::{load_hw1, print_report};
use database::hw1::hpbars::HpBarUv;
use pipeline::hw1::loader::load_game_dir;
use test_utils::require_game_dir;

#[test]
#[ignore = "requires HW1_GAME_DIR"]
fn validate_base_game() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
#[ignore = "requires HW1_GAME_DIR"]
fn load_world_base_game() {
    let dir = require_game_dir("HW1_GAME_DIR");

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

/// The shipped `_update` overlays give the engine tables (`BDatabase_setupDependentData`
/// @ `0x1401ee690`): `sys_icon_53_01` moves to the end, `game_CTF_flag_01` gets the LOS of
/// `objects_update.xml`, and the 11 balance techs follow `techs.xml`.
#[test]
#[ignore = "requires HW1_GAME_DIR"]
fn database_applies_shipped_update_overlays() {
    let dir = require_game_dir("HW1_GAME_DIR");
    let mut src = load_game_dir(&dir);

    let db = database::hw1::Database::load(&mut src).expect("failed to load database");

    assert_eq!(db.objects.len(), 2496);
    assert_eq!(db.objects[2495].name, "sys_icon_53_01");
    let flag = db
        .objects
        .iter()
        .find(|object| object.name == "game_CTF_flag_01")
        .expect("game_CTF_flag_01");
    assert_eq!(flag.los, Some(14.0));
    assert_eq!(db.squads.len(), 125);
    assert_eq!(
        db.squads.last().map(|squad| squad.name.as_str()),
        Some("game_CTF_monitor_01")
    );
    assert_eq!(db.techs.len(), 208);
    assert_eq!(
        db.techs.last().map(|tech| tech.name.as_str()),
        Some("ScarabBuff")
    );

    // 53 objects list the undeclared type `Icon`; only `_Icon` is declared.
    let index = db.object_type_index();
    assert_eq!(db.object_types.len(), 96);
    assert_eq!(index.declared_type("Icon"), None);
    assert_eq!(index.declared_type("_Icon"), Some(1));
    let with_icon = db
        .objects
        .iter()
        .filter(|object| object.object_types.iter().any(|name| name == "Icon"))
        .count();
    assert_eq!(with_icon, 53);

    let modes: Vec<&str> = db
        .game_modes
        .iter()
        .map(|mode| mode.name.as_str())
        .collect();
    assert_eq!(
        modes,
        [
            "Skirmish",
            "Deathmatch",
            "CapFlag",
            "ScoreRace",
            "WaveAttack"
        ]
    );
    assert_eq!(db.game_modes[2].locked, Some(true));
}

/// `BDatabase_setupHPBars` @ `0x1401ece90` reads `hpbars.xml` (from `root_update.era`) and
/// `BDatabase_setupPlayerColors` @ `0x1401f9560` reads `playercolors.xml` (from `root.era`).
#[test]
#[ignore = "requires HW1_GAME_DIR"]
fn database_loads_shipped_hp_bars_and_player_colors() {
    let dir = require_game_dir("HW1_GAME_DIR");
    let mut src = load_game_dir(&dir);

    let db = database::hw1::Database::load(&mut src).expect("failed to load database");

    // The shipped `.xmb` files map without warnings (the retail `<skirmish>` and `<spc>` have
    // the text "`", which the `.xml` form of the files does not show).
    let report = pipeline::hw1::validate(&mut src);
    for file in &report.files {
        if file.label == "hpbars" || file.label == "playercolors" {
            match &file.outcome {
                pipeline::hw1::validate::FileOutcome::Ok { warnings, .. } => {
                    assert_eq!(warnings.len(), 0, "{} warnings", file.label);
                }
                _ => panic!("{} did not parse", file.label),
            }
        }
    }
    let hp_bars = db.hp_bars.as_ref().expect("hpbars.xml");
    assert_eq!(hp_bars.hp_bars.len(), 20);
    assert_eq!(hp_bars.color_stages.len(), 1);
    assert_eq!(hp_bars.veterancy_bars.len(), 6);
    assert_eq!(hp_bars.pie_progress.len(), 4);
    assert_eq!(hp_bars.bobble_heads.len(), 5);
    assert_eq!(hp_bars.building_strengths.len(), 1);
    let stages = hp_bars
        .default_color_stages()
        .expect("DEFAULT color stages");
    assert_eq!(stages.hitpoint.stages.len(), 3);
    assert_eq!(stages.ammo.stages.len(), 3);
    assert_eq!(stages.shield.stages.len(), 3);
    assert!((stages.hitpoint.stages[0].value - 0.38).abs() < 1.0e-6);
    assert_eq!(stages.hitpoint.stages[0].color.red, 208);
    let health = &hp_bars.hp_bars[hp_bars.hp_bar_id("UNSC_Health").expect("UNSC_Health")];
    assert!((health.size_x - 8.0).abs() < 1.0e-6);
    let rect = |uv: Option<HpBarUv>| uv.as_ref().and_then(HpBarUv::rect);
    assert!(rect(health.hitpoint_uv).is_some());
    assert_eq!(rect(health.shield_uv), None);
    assert!(hp_bars.pie_progress.iter().all(|pie| !pie.fill));

    let colors = db.player_colors.as_ref().expect("playercolors.xml");
    for set in [&colors.skirmish, &colors.spc] {
        let set = set.as_ref().expect("color set");
        assert_eq!(set.colors.len(), 16);
        assert!((0..16).all(|num| set.color(num).is_some()));
        assert!(set.friend_or_foe_self.is_some());
        assert!(set.friend_or_foe_ally.is_some());
        assert!(set.friend_or_foe_neutral.is_some());
        assert!(set.friend_or_foe_enemy.is_some());
    }
    let skirmish = colors.skirmish.as_ref().expect("skirmish");
    let player_one = skirmish.color(1).expect("color 1");
    assert_eq!(
        (
            player_one.selection.red,
            player_one.selection.green,
            player_one.selection.blue
        ),
        (69, 110, 24)
    );
    let civs: Vec<&str> = colors.civs.iter().map(|civ| civ.name.as_str()).collect();
    assert_eq!(civs, ["UNSC", "Covenant"]);
    assert_eq!(colors.civs[1].players.len(), 9);
}

/// `BUIGame_loadFlashResources` @ `0x1404c07c0` gives `hud_decal.xml` to the UNSC and
/// `hud_decal_cov.xml` to the Covenant. `BSquad_drawSelectionDecal` @ `0x1403abea0` sizes the
/// decal of a squad from its first bound icon.
#[test]
#[ignore = "requires HW1_GAME_DIR"]
fn database_loads_shipped_decal_lists() {
    let dir = require_game_dir("HW1_GAME_DIR");
    let mut src = load_game_dir(&dir);

    let db = database::hw1::Database::load(&mut src).expect("failed to load database");

    let report = pipeline::hw1::validate(&mut src);
    for label in ["hud_decal", "hud_decal_cov"] {
        let file = report
            .files
            .iter()
            .find(|file| file.label == label)
            .expect("decal list in the report");
        match &file.outcome {
            pipeline::hw1::validate::FileOutcome::Ok { warnings, .. } => {
                let warnings: Vec<String> = warnings.iter().map(ToString::to_string).collect();
                assert!(warnings.is_empty(), "{label} warnings: {warnings:?}");
            }
            _ => panic!("{label} did not parse"),
        }
    }
    let civs: Vec<&str> = db
        .decal_lists
        .iter()
        .map(|entry| entry.civ.as_str())
        .collect();
    assert_eq!(civs, ["UNSC", "Covenant"]);

    let unsc = db.decal_list("unsc").expect("UNSC decal list");
    assert_eq!(unsc.icons.len(), 354);
    // The `.xmb` root has the text "a", which the `.xml` form of the file does not show.
    assert_eq!(unsc.text.as_deref(), Some("a"));
    let warthog = unsc
        .owner_icon("unsc_veh_warthog_01")
        .expect("Warthog icon");
    assert_eq!(warthog.size(), [6.0, 0.0, 6.0]);
    let command = unsc.owner_icon("UNSC_BLDG_COMMAND_03").expect("HQ icon");
    assert_eq!(command.size(), [37.0, 2.0, 37.0]);
    let offset = command.offset();
    assert_eq!([offset.x, offset.y, offset.z], [3.0, -1.5, 0.0]);
    // The engine reads `Intenstiy1`, so the shipped `Intensity1` gives the default 1.
    assert_eq!(warthog.intensities(), [1.0, 28.0, 18.0, 1.0]);

    let covenant = db.decal_list("Covenant").expect("Covenant decal list");
    assert_eq!(covenant.icons.len(), 334);
    let command = covenant
        .owner_icon("unsc_bldg_command_03")
        .expect("HQ icon in the Covenant list");
    assert_eq!(command.size(), [37.0, 2.0, 37.0]);
}

/// Round-trip test: load every database file, serialize back to XMB via
/// `Database::to_documents()`, re-parse, and compare counts.
///
/// This catches serialization regressions that would silently drop data.
#[test]
#[ignore = "requires HW1_GAME_DIR"]
fn roundtrip_database_serialize() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
        assert_document_round_trip(&original, path, doc);
    }

    println!("Round-trip validation passed!");
}

/// Re-parse one serialized document and compare it with the loaded database.
fn assert_document_round_trip(original: &database::hw1::Database, path: &str, doc: &xmb::Document) {
    match path {
        "data\\objects.xml" => {
            let reparsed = database::hw1::objects::parse(doc).expect("failed to re-parse objects");
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
            let reparsed = database::hw1::squads::parse(doc).expect("failed to re-parse squads");
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
            let reparsed = database::hw1::powers::parse(doc).expect("failed to re-parse powers");
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
            let reparsed = database::hw1::leaders::parse(doc).expect("failed to re-parse leaders");
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
            let rt_res = reparsed.resources.as_ref().map_or(0, |r| r.entries.len());
            let orig_res = orig_gd.resources.as_ref().map_or(0, |r| r.entries.len());
            let rt_pops = reparsed.pops.as_ref().map_or(0, |p| p.entries.len());
            let orig_pops = orig_gd.pops.as_ref().map_or(0, |p| p.entries.len());
            assert_eq!(rt_res, orig_res, "resources count mismatch");
            assert_eq!(rt_pops, orig_pops, "pops count mismatch");
            println!("  gamedata: {rt_res} resources, {rt_pops} pops ✓");
        }
        "data\\objects_update.xml" => {
            let reparsed = database::hw1::objects::parse(doc).expect("objects_update");
            assert!(reparsed.is_empty(), "objects_update must be empty");
        }
        "data\\squads_update.xml" => {
            let reparsed = database::hw1::squads::parse(doc).expect("squads_update");
            assert!(reparsed.is_empty(), "squads_update must be empty");
        }
        "data\\techs_update.xml" => {
            let reparsed = database::hw1::techs::parse(doc).expect("techs_update");
            assert!(reparsed.is_empty(), "techs_update must be empty");
        }
        "data\\objecttypes.xml" => {
            let reparsed =
                database::hw1::objecttypes::parse(doc).expect("failed to re-parse objecttypes");
            assert_eq!(reparsed.len(), original.object_types.len());
            println!("  objecttypes: {} ✓", reparsed.len());
        }
        "data\\gamemodes.xml" => {
            let reparsed =
                database::hw1::gamemodes::parse(doc).expect("failed to re-parse gamemodes");
            assert_eq!(reparsed.len(), original.game_modes.len());
            println!("  gamemodes: {} ✓", reparsed.len());
        }
        "data\\aidifficultysettings.xml" => {
            let reparsed = database::hw1::aidifficultysettings::parse(doc)
                .expect("failed to re-parse aidifficultysettings");
            assert_eq!(reparsed.len(), original.ai_difficulty_settings.len());
            println!("  aidifficultysettings: {} ✓", reparsed.len());
        }
        "data\\hpbars.xml" => {
            let reparsed = database::hw1::hpbars::parse(doc).expect("failed to re-parse hpbars");
            let orig = original.hp_bars.as_ref().expect("hp bars");
            assert_eq!(reparsed.hp_bars.len(), orig.hp_bars.len());
            assert_eq!(reparsed.color_stages.len(), orig.color_stages.len());
            println!("  hpbars: {} ✓", reparsed.hp_bars.len());
        }
        "data\\playercolors.xml" => {
            let reparsed =
                database::hw1::playercolors::parse(doc).expect("failed to re-parse playercolors");
            let orig = original.player_colors.as_ref().expect("player colors");
            assert_eq!(
                reparsed.skirmish.map(|set| set.colors),
                orig.skirmish.as_ref().map(|set| set.colors.clone())
            );
            println!("  playercolors ✓");
        }
        decal if decal.starts_with(DECAL_DIR) => {
            assert_decal_list_round_trip(original, decal, doc);
        }
        other => panic!("unexpected document path: {other}"),
    }
}

/// The game directory of the Flash decal lists.
const DECAL_DIR: &str = "art\\ui\\flash\\hud\\hud_decal\\";

/// Re-parse a serialized decal list and compare its icons with the loaded list.
fn assert_decal_list_round_trip(
    original: &database::hw1::Database,
    path: &str,
    doc: &xmb::Document,
) {
    let reparsed = database::hw1::flashdecals::parse(doc).expect("failed to re-parse a decal list");
    let orig = original
        .decal_lists
        .iter()
        .find(|entry| entry.path == path)
        .expect("decal list");
    assert_eq!(reparsed.icons, orig.list.icons);
    println!("  {path}: {} icons ✓", reparsed.icons.len());
}

#[test]
#[ignore = "requires HW1_GAME_DIR"]
fn read_model_ugx() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
#[ignore = "requires HW1_GAME_DIR"]
fn read_animation_uax() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
#[ignore = "requires HW1_GAME_DIR"]
fn read_texture_ddx() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
        if let Some(tex) = world.read_texture(path, &mut src) {
            println!(
                "  OK  {path}: {}x{} {:?}",
                tex.info.width, tex.info.height, tex.info.data_format
            );
            loaded += 1;
        } else {
            println!("  MISS {path}");
            failed += 1;
        }
    }

    println!("  Loaded {loaded}, missing {failed}");
    assert!(loaded > 0, "should have loaded at least one texture");
}

#[test]
#[ignore = "requires HW1_GAME_DIR"]
fn validate_binary_assets_sample() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
#[ignore = "requires HW1_GAME_DIR"]
fn texture_refs_populated_eagerly() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
#[ignore = "requires HW1_GAME_DIR"]
fn cross_reference_diagnostics() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
#[ignore = "requires HW1_GAME_DIR"]
fn inject_bad_reference_produces_diagnostic() {
    let dir = require_game_dir("HW1_GAME_DIR");

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
