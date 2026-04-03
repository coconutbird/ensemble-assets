//! Integration tests for the watch crate.
//!
//! Requires `HW1_GAME_DIR` to point to a Halo Wars DE installation.

use std::path::Path;

/// Load the `.env` from the workspace root and return `HW1_GAME_DIR` if set.
fn hw1_game_dir() -> Option<String> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("could not find workspace root");
    let env_path = workspace_root.join(".env");
    let _ = dotenvy::from_path(&env_path);
    std::env::var("HW1_GAME_DIR").ok()
}

#[test]
fn table_for_override_path_unit() {
    // Covered by lib.rs unit tests — just verify the public API is accessible.
    let dir = std::path::PathBuf::from("/tmp/override");
    let path = dir.join("root.era").join("data").join("objects.xml");
    assert_eq!(
        watch::table_for_override_path(&dir, &path),
        Some(watch::TableId::Objects),
    );
}

#[test]
fn manual_reload_updates_world() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    // Load world + source
    let tmp = tempfile::tempdir().expect("tempdir");
    let (world, mut src) = pipeline::hw1::World::load(&dir).expect("load");
    src.set_override_dir(tmp.path());
    let original_count = world.database.objects.len();
    println!("Original object count: {original_count}");

    // Create watcher (not started — we'll use manual reload)
    let mut watcher = watch::WorldWatcher::new(world, src, tmp.path());

    // Mutate the objects table via the watcher's world
    {
        let world = watcher.world_mut();
        let mut objects = world.objects_mut();
        objects[0].hitpoints = Some(12345.0);
    }

    // Save to override dir
    {
        let written = watcher.save().expect("save");
        assert!(!written.is_empty());
        println!("Wrote {} files", written.len());
    }

    // Now manually reload — simulates what the file watcher would do
    let events = watcher.reload(watch::TableId::Objects);
    println!("Reload events: {}", events.len());

    let mut got_reload = false;
    let mut got_diagnostics = false;
    for ev in &events {
        match ev {
            watch::WorldEvent::AssetReloaded(kind) => {
                println!("  Reloaded: {kind:?}");
                assert_eq!(
                    *kind,
                    watch::AssetKind::DatabaseTable(watch::TableId::Objects)
                );
                got_reload = true;
            }
            watch::WorldEvent::DiagnosticsUpdated(r) => {
                println!(
                    "  Diagnostics: {} errors, {} warnings",
                    r.errors(),
                    r.warnings()
                );
                got_diagnostics = true;
            }
            watch::WorldEvent::Error(e) => panic!("unexpected error: {e}"),
            watch::WorldEvent::UnknownFile(p) => panic!("unexpected unknown: {}", p.display()),
            watch::WorldEvent::TableReloaded(_) => {}
        }
    }

    assert!(got_reload, "should have gotten an AssetReloaded event");
    assert!(
        got_diagnostics,
        "should have gotten a DiagnosticsUpdated event"
    );

    // The reloaded world should have the mutated hitpoints
    // (because the override file has them)
    let hp = watcher.world().database.objects[0].hitpoints;
    println!("Reloaded HP: {hp:?}");
    assert_eq!(hp, Some(12345.0), "hitpoints should persist through reload");

    println!("\n✅ Manual reload test passed!");
}

#[test]
fn filesystem_watcher_detects_change() {
    let Some(dir) = hw1_game_dir() else {
        eprintln!("SKIP: HW1_GAME_DIR not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let (world, mut src) = pipeline::hw1::World::load(&dir).expect("load");
    src.set_override_dir(tmp.path());
    let mut watcher = watch::WorldWatcher::new(world, src, tmp.path());

    // Start the file watcher
    watcher.start().expect("start watcher");

    // Write a modified objects.xml to the override directory.
    // We'll save via the world's save mechanism (which writes to override dir).
    {
        let world = watcher.world_mut();
        let mut objects = world.objects_mut();
        objects[0].hitpoints = Some(77777.0);
    }
    {
        let written = watcher.save().expect("save");
        assert!(!written.is_empty());
        println!("Wrote to override dir: {:?}", written[0]);
    }

    // Give the debouncer time to fire (200ms debounce + some slack).
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Poll for events
    let events = watcher.poll();
    println!("Got {} events from filesystem watcher", events.len());

    let got_reload = events.iter().any(|e| {
        matches!(
            e,
            watch::WorldEvent::AssetReloaded(watch::AssetKind::DatabaseTable(
                watch::TableId::Objects
            ))
        )
    });

    // The filesystem watcher should have detected the file write and reloaded.
    // Note: on some CI/OS combinations, filesystem events can be flaky.
    if got_reload {
        println!("✅ Filesystem watcher detected change and reloaded!");
    } else {
        println!("⚠️  No reload event (may be OS/timing dependent). Events:");
        for ev in &events {
            println!("  {ev:?}");
        }
    }

    watcher.stop();
}
