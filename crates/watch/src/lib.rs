//! Incremental file watcher for the Ensemble asset pipeline.
//!
//! Watches the override directory for changes, maps filesystem paths back to
//! [`AssetKind`]s, incrementally reloads affected assets, re-validates,
//! and emits [`WorldEvent`]s via an [`std::sync::mpsc`] channel.
//!
//! Designed to serve both:
//! - **Language server** — `recv()` / async bridge for LSP diagnostics
//! - **Engine hot-reload** — `try_recv()` polling each frame

pub mod watcher;

pub use pipeline::hw1::{
    AssetKind, Diagnostic, DiagnosticCode, DiagnosticReport, Location, Severity, TableId,
};
pub use watcher::{WorldEvent, WorldWatcher};

use std::path::Path;

/// Extract a game path from a filesystem path inside the override directory.
///
/// Override layout: `{override_dir}/{era_label}/{game_path}`.
/// Strips the override prefix, skips the first path component (era label),
/// and returns the backslash-joined remainder.
pub fn game_path_from_override(override_dir: &Path, fs_path: &Path) -> Option<String> {
    // Canonicalize both paths to handle macOS /var → /private/var symlinks.
    let canon_dir = override_dir
        .canonicalize()
        .unwrap_or_else(|_| override_dir.to_path_buf());
    let canon_path = fs_path
        .canonicalize()
        .unwrap_or_else(|_| fs_path.to_path_buf());

    let relative = canon_path.strip_prefix(&canon_dir).ok()?;

    // Skip the first component (era label like "root.era").
    let mut components = relative.components();
    components.next()?; // era_label

    let game_path: String = components
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\\");

    if game_path.is_empty() {
        return None;
    }

    Some(game_path)
}

/// Map a filesystem path inside the override directory to an [`AssetKind`].
///
/// Convenience wrapper around [`game_path_from_override`] +
/// [`AssetKind::from_game_path`].
pub fn asset_for_override_path(override_dir: &Path, fs_path: &Path) -> Option<AssetKind> {
    let gp = game_path_from_override(override_dir, fs_path)?;
    AssetKind::from_game_path(&gp)
}

/// Legacy alias — maps to a [`TableId`] only (database XML tables).
///
/// Prefer [`asset_for_override_path`] for full coverage.
pub fn table_for_override_path(override_dir: &Path, fs_path: &Path) -> Option<TableId> {
    let gp = game_path_from_override(override_dir, fs_path)?;
    TableId::from_game_path(&gp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_table_for_override_path() {
        let override_dir = PathBuf::from("/tmp/override");

        let path = override_dir
            .join("root.era")
            .join("data")
            .join("objects.xml");
        assert_eq!(
            table_for_override_path(&override_dir, &path),
            Some(TableId::Objects)
        );

        let path = override_dir
            .join("root.era")
            .join("data")
            .join("squads.xml.xmb");
        assert_eq!(
            table_for_override_path(&override_dir, &path),
            Some(TableId::Squads)
        );

        let path = override_dir
            .join("PHXscn01.era")
            .join("scenario")
            .join("skirmish")
            .join("design")
            .join("blood_gulch.scn");
        assert_eq!(
            table_for_override_path(&override_dir, &path),
            Some(TableId::Scenario)
        );

        let path = PathBuf::from("/other/dir/data/objects.xml");
        assert_eq!(table_for_override_path(&override_dir, &path), None);
    }

    #[test]
    fn test_asset_for_override_path() {
        let override_dir = PathBuf::from("/tmp/override");

        // Database table
        let path = override_dir
            .join("root.era")
            .join("data")
            .join("objects.xml");
        assert_eq!(
            asset_for_override_path(&override_dir, &path),
            Some(AssetKind::DatabaseTable(TableId::Objects)),
        );

        // Visual
        let path = override_dir
            .join("root.era")
            .join("art")
            .join("unsc_inf_marine_01.vis");
        assert_eq!(
            asset_for_override_path(&override_dir, &path),
            Some(AssetKind::Visual("art\\unsc_inf_marine_01.vis".into())),
        );

        // Model (.ugx)
        let path = override_dir
            .join("root.era")
            .join("art")
            .join("unsc_inf_marine_01.ugx");
        assert_eq!(
            asset_for_override_path(&override_dir, &path),
            Some(AssetKind::Model("art\\unsc_inf_marine_01.ugx".into())),
        );

        // Texture (.ddx)
        let path = override_dir
            .join("root.era")
            .join("art")
            .join("unsc_inf_marine_01_df.ddx");
        assert_eq!(
            asset_for_override_path(&override_dir, &path),
            Some(AssetKind::Texture("art\\unsc_inf_marine_01_df.ddx".into())),
        );

        // Tactics
        let path = override_dir
            .join("root.era")
            .join("data")
            .join("tactics")
            .join("unsc_inf_marine_01_tactics.xml.xmb");
        assert_eq!(
            asset_for_override_path(&override_dir, &path),
            Some(AssetKind::Tactics(
                "data\\tactics\\unsc_inf_marine_01_tactics.xml".into()
            )),
        );

        // Animation (.uax)
        let path = override_dir
            .join("root.era")
            .join("art")
            .join("anim_idle_01.uax");
        assert_eq!(
            asset_for_override_path(&override_dir, &path),
            Some(AssetKind::Animation("art\\anim_idle_01.uax".into())),
        );

        // Terrain (.xtd)
        let path = override_dir
            .join("PHXscn01.era")
            .join("scenario")
            .join("skirmish")
            .join("blood_gulch.xtd");
        assert_eq!(
            asset_for_override_path(&override_dir, &path),
            Some(AssetKind::TerrainData(
                "scenario\\skirmish\\blood_gulch.xtd".into()
            )),
        );

        // Physics
        let path = override_dir
            .join("root.era")
            .join("physics")
            .join("warthog.physics");
        assert_eq!(
            asset_for_override_path(&override_dir, &path),
            Some(AssetKind::Physics("physics\\warthog.physics".into())),
        );

        // Unknown extension
        let path = override_dir
            .join("root.era")
            .join("data")
            .join("random.foo");
        assert_eq!(asset_for_override_path(&override_dir, &path), None);
    }
}
