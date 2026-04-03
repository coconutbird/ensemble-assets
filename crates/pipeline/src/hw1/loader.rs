//! HW1 ERA loading — mirrors the engine's `BArchiveManager` load order.

use crate::source::{AssetSource, StdFileProvider};

/// Base game ERA names loaded in the engine's confirmed load order.
///
/// Archives loaded later have **higher priority** (last loaded wins):
///
/// 1. `locale.era` / `locale_update.era` — localised strings (lowest priority)
/// 2. `root.era` / `root_update.era`     — base game data + patches
/// 3. `shader.era`                       — compiled shaders
/// 4. `miniloader.era` / `pregameUI.era` — loading & menu UI
/// 5. `ingameUI.era`                     — in-game UI
/// 6. `scenarioshared.era`               — shared scenario assets
/// 7. `dlc01.era` / `dlc02.era`          — DLC content (highest priority)
const BASE_ERAS: &[&str] = &[
    // Phase 1 — Early init (sub_140820B60)
    "locale.era",
    "locale_update.era",
    "root.era",
    "root_update.era",
    "shader.era",
    // Phase 2 — Game init (BArchiveManager::beginGameInit)
    "miniloader.era",
    "pregameUI.era",
    // Phase 3 — Scenario load (BArchiveManager::beginScenarioPrefetch)
    "ingameUI.era",
    "scenarioshared.era",
];

/// Build an [`AssetSource`] from a game directory, loading ERAs in the
/// engine's confirmed load order (from IDA `BArchiveManager`).
pub fn load_game_dir(dir: &str) -> AssetSource<StdFileProvider> {
    let mut src = AssetSource::with_provider(StdFileProvider);

    for name in BASE_ERAS {
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

/// Build an [`AssetSource`] with a scenario ERA layered on top.
///
/// `scenario_era` is the ERA filename (e.g. `"PHXscn01.era"` or
/// `"blood_gulch.era"`).
pub fn load_with_scenario(dir: &str, scenario_era: &str) -> AssetSource<StdFileProvider> {
    let mut src = load_game_dir(dir);
    load_scenario_era(&mut src, dir, scenario_era);
    src
}

/// Add a scenario ERA to an existing source. Returns `true` if loaded.
pub fn load_scenario_era(
    src: &mut AssetSource<StdFileProvider>,
    dir: &str,
    scenario_era: &str,
) -> bool {
    let path = format!("{dir}/{scenario_era}");
    if std::path::Path::new(&path).exists() {
        match src.add_era(&path) {
            Ok(n) => {
                println!("  Loaded {scenario_era:<24} ({n} entries)");
                return true;
            }
            Err(e) => eprintln!("  WARN  {scenario_era}: {e}"),
        }
    }
    false
}

/// Try to find the ERA filename for a scenario by map name.
///
/// The engine names scenario ERAs either after the scenario (e.g.
/// `blood_gulch.era` for skirmish maps) or with a campaign prefix
/// (e.g. `PHXscn01.era`). This scans the game directory for a matching
/// `.era` file.
///
/// Accepts:
/// - An exact ERA filename: `"PHXscn01.era"` → returned as-is
/// - A map name: `"blood_gulch"` → looks for `blood_gulch.era`
/// - A full SCN path: `"skirmish\design\blood_gulch\blood_gulch.scn"` → extracts stem
pub fn find_scenario_era(dir: &str, scenario: &str) -> Option<String> {
    // Already an ERA filename?
    if scenario.ends_with(".era") {
        let path = format!("{dir}/{scenario}");
        if std::path::Path::new(&path).exists() {
            return Some(scenario.to_string());
        }
        return None;
    }

    // Extract the map stem from a full SCN path or use as-is.
    let stem = scenario
        .rsplit(['\\', '/'])
        .next()
        .and_then(|s| s.strip_suffix(".scn"))
        .unwrap_or(scenario);

    // Look for `{stem}.era` in the game directory.
    let candidate = format!("{stem}.era");
    let path = format!("{dir}/{candidate}");
    if std::path::Path::new(&path).exists() {
        return Some(candidate);
    }

    // Scan all .era files for a case-insensitive match (handles
    // naming inconsistencies like `PHXscn01.era` vs `phxscn01`).
    let stem_lower = stem.to_lowercase();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.to_lowercase().ends_with(".era") {
                let file_stem = name.strip_suffix(".era").unwrap_or(&name);
                if file_stem.to_lowercase() == stem_lower {
                    return Some(name);
                }
            }
        }
    }

    None
}
