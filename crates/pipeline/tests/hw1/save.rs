//! Base game edit and save tests: round trips for visuals, tactics, physics,
//! string tables, models, textures and animations.

use super::hw1_game_dir;

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

    let Some(obj_name) = test_obj else {
        eprintln!("SKIP: no object with visual+tactics+physics found");
        return;
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

    let Some(model_path) = model_path else {
        eprintln!("SKIP: no resolvable model found");
        return;
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

    let Some(tex_path) = tex_path else {
        eprintln!("SKIP: no resolvable texture found");
        return;
    };
    println!("Testing texture: {tex_path}");

    // Load into cache
    let tex = world
        .load_texture(&tex_path, &mut src)
        .expect("load_texture failed");
    let original_width = tex.info.width;
    let original_height = tex.info.height;
    println!("  {original_width}x{original_height}");

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

    let Some(anim_path) = anim_path else {
        eprintln!("SKIP: no resolvable animation found");
        return;
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
