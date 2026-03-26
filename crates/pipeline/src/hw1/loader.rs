//! HW1 ERA loading — mirrors the engine's `BArchiveManager` load order.

use crate::source::{AssetSource, StdFileProvider};

/// Build an [`AssetSource`] from a game directory, loading ERAs in the
/// engine's confirmed load order (from IDA `BArchiveManager`).
///
/// The engine loads archives across several init phases.  Archives loaded
/// later have **higher priority** (last loaded wins):
///
/// 1. `locale.era` / `locale_update.era` — localised strings (lowest priority)
/// 2. `root.era` / `root_update.era`     — base game data + patches
/// 3. `shader.era`                       — compiled shaders
/// 4. `miniloader.era` / `pregameUI.era` — loading & menu UI
/// 5. `ingameUI.era`                     — in-game UI
/// 6. `scenarioshared.era`               — shared scenario assets
/// 7. `dlc01.era` / `dlc02.era`          — DLC content (highest priority)
pub fn load_game_dir(dir: &str) -> AssetSource<StdFileProvider> {
    let mut src = AssetSource::with_provider(StdFileProvider);

    // Phase 1 — Early init (sub_140820B60)
    let phase1 = [
        "locale.era",
        "locale_update.era",
        "root.era",
        "root_update.era",
        "shader.era",
    ];

    // Phase 2 — Game init (BArchiveManager::beginGameInit)
    let phase2 = ["miniloader.era", "pregameUI.era"];

    // Phase 3 — Scenario load (BArchiveManager::beginScenarioPrefetch)
    let phase3 = ["ingameUI.era", "scenarioshared.era"];

    for name in phase1.iter().chain(phase2.iter()).chain(phase3.iter()) {
        let path = format!("{dir}/{name}");
        if std::path::Path::new(&path).exists() {
            match src.add_era(&path) {
                Ok(n) => println!("  Loaded {name:<24} ({n} entries)"),
                Err(e) => eprintln!("  WARN  {name}: {e}"),
            }
        }
    }

    // Phase 4 — DLC (BArchiveManager::loadDLCArchives)
    for i in 1..=10 {
        let name = format!("dlc{i:02}.era");
        let path = format!("{dir}/{name}");
        if std::path::Path::new(&path).exists() {
            match src.add_era(&path) {
                Ok(n) => println!("  Loaded {name:<24} ({n} entries)"),
                Err(e) => eprintln!("  WARN  {name}: {e}"),
            }
        }
    }
    src
}

/// Build an [`AssetSource`] with an optional scenario ERA layered on top.
pub fn load_with_scenario(dir: &str, scenario_era: &str) -> AssetSource<StdFileProvider> {
    let mut src = load_game_dir(dir);
    let path = format!("{dir}/{scenario_era}");
    if std::path::Path::new(&path).exists() {
        match src.add_era(&path) {
            Ok(n) => println!("  Loaded {scenario_era:<24} ({n} entries)"),
            Err(e) => eprintln!("  WARN  {scenario_era}: {e}"),
        }
    }
    src
}
