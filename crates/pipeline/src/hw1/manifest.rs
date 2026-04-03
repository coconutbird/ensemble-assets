//! Asset manifest — tracks all binary asset references discovered during
//! world loading.
//!
//! The manifest is a passive inventory: it records every file path referenced
//! by visual chains, scenario data, and preload lists. Call [`AssetManifest::verify`]
//! to check existence, or [`super::World::validate_binary_assets`] to parse each file.

use std::collections::BTreeSet;

use assets::AssetResolver;

use crate::source::AssetSource;

use super::resolve::ObjectAssets;
use super::scenario::{ScenarioData, ScenarioDescriptor, ScenarioList};

// ── Manifest ────────────────────────────────────────────────────────────

/// Manifest of all binary asset references discovered during resolution.
///
/// This is a passive inventory of every asset path referenced by the
/// resolved visual chains, scenario data, and preload lists. It does
/// **not** verify whether those files actually exist in the loaded
/// archives — call [`AssetManifest::verify`] for that.
#[derive(Debug, Clone, Default)]
pub struct AssetManifest {
    // ── Object visual chain refs ────────────────────────────────────
    /// Unique model file paths (.ugx) referenced by visuals.
    pub model_refs: BTreeSet<String>,
    /// Unique animation file paths (.uax) referenced by visuals.
    pub anim_refs: BTreeSet<String>,
    /// Unique damage model file paths (.ugx) referenced by visuals.
    pub damage_model_refs: BTreeSet<String>,
    /// Unique texture file paths (.ddx) extracted from UGX material chunks.
    pub texture_refs: BTreeSet<String>,

    // ── Preload lists (from scenario ERA) ───────────────────────────
    /// Visual files listed in `visFileList.txt`.
    pub preload_vis_refs: Vec<String>,
    /// Effect files listed in `tfxFileList.txt`.
    pub preload_tfx_refs: Vec<String>,
    /// Particle effect files listed in `pfxFileList.txt`.
    pub preload_pfx_refs: Vec<String>,

    // ── Scenario-level refs (from .scn) ────────────────────────────
    /// Lightset file paths (.gls/.fls) from the scenario.
    pub lightset_refs: Vec<String>,
    /// Cinematic file paths (.cin) from the scenario.
    pub cinematic_refs: Vec<String>,
    /// Talking head asset names from the scenario.
    pub talking_head_refs: Vec<String>,
    /// Terrain file paths (.xtd/.xtt) for the loaded scenario.
    pub terrain_refs: Vec<String>,
    /// Sky dome reference from the scenario.
    pub sky_ref: Option<String>,
    /// Terrain environment texture ref from the scenario.
    pub terrain_env_ref: Option<String>,
    /// Minimap texture path from the scenario.
    pub minimap_ref: Option<String>,
    /// Sound bank names from the scenario.
    pub sound_bank_refs: Vec<String>,
}

/// Result of verifying binary asset references against loaded archives.
#[derive(Debug, Clone, Default)]
pub struct VerifyResult {
    pub models_found: usize,
    pub models_missing: Vec<String>,
    pub anims_found: usize,
    pub anims_missing: Vec<String>,
    pub textures_found: usize,
    pub textures_missing: Vec<String>,
}

/// Result of deep-validating binary assets (parse every file, not just check existence).
#[derive(Debug, Clone, Default)]
pub struct BinaryValidation {
    pub models_ok: usize,
    pub models_failed: Vec<String>,
    pub models_missing: Vec<String>,
    pub anims_ok: usize,
    pub anims_failed: Vec<String>,
    pub anims_missing: Vec<String>,
    pub textures_ok: usize,
    pub textures_failed: Vec<String>,
    pub textures_missing: Vec<String>,
}

impl BinaryValidation {
    /// Total number of errors (failed + missing).
    pub fn total_errors(&self) -> usize {
        self.models_failed.len()
            + self.models_missing.len()
            + self.anims_failed.len()
            + self.anims_missing.len()
            + self.textures_failed.len()
            + self.textures_missing.len()
    }
}

impl AssetManifest {
    /// Check which referenced binary assets actually exist in `src`.
    ///
    /// This is cheap — just index lookups, no decompression.
    pub fn verify(&self, src: &AssetSource<impl assets::FileProvider>) -> VerifyResult {
        let mut result = VerifyResult::default();
        for path in &self.model_refs {
            if src.exists(path) {
                result.models_found += 1;
            } else {
                result.models_missing.push(path.clone());
            }
        }
        for path in &self.anim_refs {
            if src.exists(path) {
                result.anims_found += 1;
            } else {
                result.anims_missing.push(path.clone());
            }
        }
        for path in &self.texture_refs {
            if src.exists(path) {
                result.textures_found += 1;
            } else {
                result.textures_missing.push(path.clone());
            }
        }
        result
    }
}

// ── Visual chain collection ─────────────────────────────────────────

/// Collect all model/anim asset references from a parsed visual.
pub(crate) fn collect_visual_assets(vis: &database::hw1::Visual, manifest: &mut AssetManifest) {
    for model in &vis.models {
        if let Some(comp) = &model.component {
            for asset in &comp.assets {
                register_asset(asset, manifest);
            }

            if let Some(logic) = &comp.logic {
                for entry in &logic.entries {
                    if let Some(asset) = &entry.asset {
                        register_asset(asset, manifest);
                    }
                }
            }
        }

        for anim in &model.anims {
            for asset in &anim.assets {
                register_asset(asset, manifest);
            }
        }
    }
}

/// Register a single asset reference in the manifest.
fn register_asset(asset: &database::hw1::visual::Asset, manifest: &mut AssetManifest) {
    if let Some(file) = &asset.file {
        let normalized = file.replace('/', "\\");
        match asset.asset_type.as_str() {
            "Model" => {
                manifest.model_refs.insert(format!("art\\{normalized}.ugx"));
            }
            "Anim" => {
                manifest.anim_refs.insert(format!("art\\{normalized}.uax"));
            }
            _ => {}
        }
    }

    if let Some(dmg) = &asset.damage_file {
        manifest
            .damage_model_refs
            .insert(format!("art\\{}.ugx", dmg.replace('/', "\\")));
    }
}

/// Collect model/anim/damage paths from a visual into a per-object bundle.
pub(crate) fn collect_object_visual_assets(vis: &database::hw1::Visual, obj: &mut ObjectAssets) {
    for model in &vis.models {
        if let Some(comp) = &model.component {
            for asset in &comp.assets {
                register_object_asset(asset, obj);
            }

            if let Some(logic) = &comp.logic {
                for entry in &logic.entries {
                    if let Some(asset) = &entry.asset {
                        register_object_asset(asset, obj);
                    }
                }
            }
        }

        for anim in &model.anims {
            for asset in &anim.assets {
                register_object_asset(asset, obj);
            }
        }
    }
}

/// Register a single asset reference in a per-object bundle.
fn register_object_asset(asset: &database::hw1::visual::Asset, obj: &mut ObjectAssets) {
    if let Some(file) = &asset.file {
        let normalized = file.replace('/', "\\");
        match asset.asset_type.as_str() {
            "Model" => obj.models.push(format!("art\\{normalized}.ugx")),
            "Anim" => obj.anims.push(format!("art\\{normalized}.uax")),
            _ => {}
        }
    }
    if let Some(dmg) = &asset.damage_file {
        obj.damage_models
            .push(format!("art\\{}.ugx", dmg.replace('/', "\\")));
    }
}

// ── Preload lists ───────────────────────────────────────────────────

/// Parse a preload list file (e.g. `visFileList.txt`) from the asset source.
///
/// These are plain-text files with one asset path per line, found in
/// scenario ERAs. The engine reads them in `BScenario::preloadVisFiles`,
/// `BScenario::preloadTfxFiles`, and `BScenario::preloadPfxFiles`.
pub(crate) fn parse_preload_list(
    src: &mut AssetSource<impl assets::FileProvider>,
    filename: &str,
    out: &mut Vec<String>,
) {
    let data = match src.resolve_exact(filename) {
        Some(d) => d,
        None => return,
    };
    let text = match std::str::from_utf8(&data) {
        Ok(t) => t,
        Err(_) => return,
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            out.push(trimmed.to_string());
        }
    }
}

// ── Texture discovery ───────────────────────────────────────────────

/// Eagerly discover texture references from UGX material chunks.
///
/// For every model path in `model_refs`, this reads only the material
/// chunk (0x704) of each UGX file and extracts all texture map names,
/// normalising them to `art\...\name.ddx` form.
pub(crate) fn discover_textures(
    src: &mut AssetSource<impl assets::FileProvider>,
    model_refs: &BTreeSet<String>,
    texture_refs: &mut BTreeSet<String>,
) {
    for ugx_path in model_refs {
        let data = match src.resolve_exact(ugx_path) {
            Some(d) => d,
            None => continue,
        };

        let materials = match ugx::read_materials(&data) {
            Ok(m) => m,
            Err(_) => continue,
        };

        for mat in &materials {
            let legacy = match mat.legacy() {
                Some(l) => l,
                None => continue,
            };

            for maps in &legacy.maps {
                for map in maps {
                    if !map.name.is_empty() {
                        let tex_name = map.name.trim_start_matches(['\\', '/']);
                        let tex_name = tex_name.replace('/', "\\");
                        texture_refs.insert(format!("art\\{tex_name}.ddx"));
                    }
                }
            }
        }
    }
}

/// Read UGX materials for all models in an object and return the
/// deduplicated, sorted list of texture paths they reference.
///
/// Only decompresses the material chunk (0x704) of each UGX — geometry,
/// vertex buffers, and index buffers are never touched.
pub(crate) fn resolve_textures_for(
    obj: &ObjectAssets,
    src: &mut AssetSource<impl assets::FileProvider>,
) -> Vec<String> {
    let mut textures = BTreeSet::new();

    let ugx_paths: Vec<&str> = obj
        .models
        .iter()
        .chain(obj.damage_models.iter())
        .map(|s| s.as_str())
        .collect();

    for ugx_path in ugx_paths {
        let data = match src.resolve_exact(ugx_path) {
            Some(d) => d,
            None => continue,
        };
        let materials = match ugx::read_materials(&data) {
            Ok(m) => m,
            Err(_) => continue,
        };
        for mat in &materials {
            let legacy = match mat.legacy() {
                Some(l) => l,
                None => continue,
            };
            for maps in &legacy.maps {
                for map in maps {
                    if !map.name.is_empty() {
                        let tex_name = map.name.trim_start_matches(['\\', '/']);
                        let tex_name = tex_name.replace('/', "\\");
                        textures.insert(format!("art\\{tex_name}.ddx"));
                    }
                }
            }
        }
    }

    textures.into_iter().collect()
}

// ── XTT terrain texture discovery ───────────────────────────────────

/// Discover texture references from XTT terrain files.
///
/// XTT files contain three kinds of texture references:
///
/// 1. **Active textures** — splat/detail terrain textures (e.g. `arctic/snowdrift_01`).
///    The engine resolves these as `art\{filename}_df.ddx` (and `_nm`, `_sp`, etc).
/// 2. **Active decals** — projected decal textures baked into the terrain.
/// 3. **Foliage sets** — foliage billboard textures (e.g. `foliage\foliageset`),
///    resolved as `art\{filename}_df.ddx`, `_nm.ddx`, `_op.ddx`, `_sp.ddx`.
///
/// All discovered paths are inserted into `texture_refs`.
pub(crate) fn discover_terrain_textures(
    src: &mut AssetSource<impl assets::FileProvider>,
    terrain_refs: &[String],
    texture_refs: &mut BTreeSet<String>,
) {
    for path in terrain_refs {
        if !path.ends_with(".xtt") {
            continue;
        }
        let data = match src.resolve_exact(path) {
            Some(d) => d,
            None => continue,
        };
        let xtt_file = match xtt::Reader::read(&data) {
            Ok(f) => f,
            Err(_) => continue,
        };

        // Active textures — terrain splat layers
        // Engine resolves: art\{filename}_df.ddx (diffuse), _nm, _sp, _em, _env, etc.
        for tex in &xtt_file.active_textures {
            if !tex.filename.is_empty() {
                let name = tex.filename.replace('/', "\\");
                let name = name.trim_start_matches('\\');
                texture_refs.insert(format!("art\\{name}_df.ddx"));
                texture_refs.insert(format!("art\\{name}_nm.ddx"));
                texture_refs.insert(format!("art\\{name}_sp.ddx"));
                texture_refs.insert(format!("art\\{name}_em.ddx"));
                texture_refs.insert(format!("art\\{name}_env.ddx"));
            }
        }

        // Active decals — baked decal textures
        for decal in &xtt_file.active_decals {
            if !decal.filename.is_empty() {
                let name = decal.filename.replace('/', "\\");
                let name = name.trim_start_matches('\\');
                texture_refs.insert(format!("art\\{name}_df.ddx"));
                texture_refs.insert(format!("art\\{name}_nm.ddx"));
                texture_refs.insert(format!("art\\{name}_op.ddx"));
            }
        }

        // Foliage sets — billboard vegetation textures
        // Engine resolves as: art\{filename}_df.ddx, _nm.ddx, _op.ddx, _sp.ddx
        for set in &xtt_file.foliage.sets {
            if !set.filename.is_empty() {
                let name = set.filename.replace('/', "\\");
                let name = name.trim_start_matches('\\');
                texture_refs.insert(format!("art\\{name}_df.ddx"));
                texture_refs.insert(format!("art\\{name}_nm.ddx"));
                texture_refs.insert(format!("art\\{name}_op.ddx"));
                texture_refs.insert(format!("art\\{name}_sp.ddx"));
            }
        }
    }
}

// ── Stub processors ────────────────────────────────────────────────
//
// Placeholder functions for binary format processors that we track by
// path but have no HW1-compatible crate for yet. Each function panics
// with a descriptive message so callers know the feature is planned.

/// Parse a PFX (particle effect) file.
///
/// HW1 particle effects (`.pfx`) are referenced via `pfxFileList.txt`
/// preload lists. We track paths in the manifest but cannot parse the
/// binary content yet.
pub fn parse_pfx(_data: &[u8]) -> ! {
    unimplemented!("PFX particle effect parsing is not yet implemented for HW1")
}

/// Parse a TFX/trigger script file.
///
/// HW1 trigger/effect scripts (`.tfx`) are referenced via `tfxFileList.txt`
/// preload lists. The UFX crate targets HW2 shaders, not HW1 triggers.
pub fn parse_tfx(_data: &[u8]) -> ! {
    unimplemented!("TFX trigger/effect script parsing is not yet implemented for HW1")
}

/// Parse an FXB (effect bundle) file.
///
/// HW1 effect bundles (`.fxb`) are referenced by various gameplay systems.
/// No format crate exists yet.
pub fn parse_fxb(_data: &[u8]) -> ! {
    unimplemented!("FXB effect bundle parsing is not yet implemented for HW1")
}

/// Parse a GLS/FLS lightset file.
///
/// HW1 lightset files are referenced by scenario SCN data. We track paths
/// in the manifest but cannot parse the binary content yet.
pub fn parse_lightset(_data: &[u8]) -> ! {
    unimplemented!("GLS/FLS lightset parsing is not yet implemented for HW1")
}

/// Parse a cinematic file.
///
/// HW1 cinematic files (`.cin`) are referenced by scenario SCN data. We
/// track paths in the manifest but cannot parse the binary content yet.
pub fn parse_cinematic(_data: &[u8]) -> ! {
    unimplemented!("Cinematic file parsing is not yet implemented for HW1")
}

/// Parse a sound bank.
///
/// HW1 sound bank names are referenced by scenario SCN data. We track
/// names in the manifest but cannot parse the binary content yet.
pub fn parse_sound_bank(_name: &str) -> ! {
    unimplemented!("Sound bank parsing is not yet implemented for HW1")
}

// ── Scenario asset collection ───────────────────────────────────────

/// Try to find a scenario whose `.scn` file resolves from the loaded ERAs.
///
/// This is called automatically during `load_from_source`. When a scenario
/// ERA is loaded, at least one scenario descriptor's SCN path will become
/// resolvable. We find it, parse the SCN, and collect all scenario-level
/// asset references into the manifest.
pub(crate) fn resolve_scenario(
    src: &mut AssetSource<impl assets::FileProvider>,
    scenario_list: &ScenarioList,
    manifest: &mut AssetManifest,
) -> (Option<ScenarioDescriptor>, Option<ScenarioData>) {
    // Try each descriptor until we find one whose SCN resolves.
    // Sort by file path for deterministic selection.
    let mut candidates: Vec<&ScenarioDescriptor> = scenario_list.scenarios.values().collect();
    candidates.sort_by(|a, b| a.file.cmp(&b.file));

    for desc in candidates {
        if let Some(scn) = desc.read_scenario(src) {
            // Found a scenario with a resolvable SCN — collect its assets.
            collect_scenario_assets_into(desc, &scn, manifest);
            return (Some(desc.clone()), Some(scn));
        }
    }

    (None, None)
}

/// Collect all asset references from a parsed scenario into the manifest.
///
/// This is the internal implementation used by both `resolve_scenario`
/// (during auto-loading) and `World::collect_scenario_assets` (for manual use).
pub(crate) fn collect_scenario_assets_into(
    descriptor: &ScenarioDescriptor,
    scn: &ScenarioData,
    manifest: &mut AssetManifest,
) {
    // Terrain files (.xtd / .xtt)
    if let Some(xtd) = descriptor.xtd_path() {
        manifest.terrain_refs.push(xtd);
    }
    if let Some(xtt) = descriptor.xtt_path() {
        manifest.terrain_refs.push(xtt);
    }

    // Lightsets
    let ls = scn.lightset();
    if !ls.is_empty() {
        manifest.lightset_refs.push(ls.to_string());
    }

    for ls in scn.lightsets() {
        if !ls.is_empty() {
            manifest.lightset_refs.push(ls.clone());
        }
    }

    // Cinematics
    for cin in scn.cinematics() {
        if !cin.path.is_empty() {
            manifest.cinematic_refs.push(cin.path.clone());
        }
    }

    // Talking heads
    for th in scn.talking_heads() {
        if !th.name.is_empty() {
            manifest.talking_head_refs.push(th.name.clone());
        }
    }

    // Sky
    let sky = scn.sky();
    if !sky.is_empty() {
        manifest.sky_ref = Some(sky.to_string());
    }

    // Terrain environment texture
    let te = scn.terrain_env();
    if !te.is_empty() {
        manifest.terrain_env_ref = Some(te.to_string());
    }

    // Minimap
    let mm = scn.minimap_texture();
    if !mm.is_empty() {
        manifest.minimap_ref = Some(mm.to_string());
    }

    // Sound banks
    for sb in scn.sound_banks() {
        if !sb.is_empty() {
            manifest.sound_bank_refs.push(sb.clone());
        }
    }
}
