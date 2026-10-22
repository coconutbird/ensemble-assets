//! Asset manifest — tracks all binary asset references discovered during world loading.
//!
//! The [`AssetManifest`] is a passive inventory built during
//! [`World::load`](super::World::load). It collects every file path
//! referenced by visuals, tactics, physics chains, scenario data, and
//! preload lists (`.ugx`, `.uax`, `.ddx`, `.gls`, `.cin`, …).
//!
//! Use [`AssetManifest::verify`] to check which references actually
//! resolve in the loaded ERAs. This is the basis for the "missing assets"
//! diagnostics.

use std::collections::BTreeSet;

use assets::AssetResolver;

use crate::source::AssetSource;

use super::resolve::ObjectAssets;
use super::scenario::{ScenarioData, ScenarioDescriptor, ScenarioList};
use super::terrain::terrain_texture_path;

/// Manifest of all binary asset references discovered during resolution.
///
/// This is a passive inventory of every asset path referenced by the
/// resolved visual chains, scenario data, and preload lists. It does
/// **not** verify whether those files actually exist in the loaded
/// archives — call [`AssetManifest::verify`] for that.
#[derive(Debug, Clone, Default)]
pub struct AssetManifest {
    /// Unique model file paths (.ugx) referenced by visuals.
    pub model_refs: BTreeSet<String>,
    /// Unique animation file paths (.uax) referenced by visuals.
    pub anim_refs: BTreeSet<String>,
    /// Unique damage model file paths (.ugx) referenced by visuals.
    pub damage_model_refs: BTreeSet<String>,
    /// Unique texture file paths (.ddx) extracted from UGX material chunks.
    pub texture_refs: BTreeSet<String>,

    /// Visual files listed in `visFileList.txt`.
    pub preload_vis_refs: Vec<String>,
    /// Effect files listed in `tfxFileList.txt`.
    pub preload_tfx_refs: Vec<String>,
    /// Particle effect files listed in `pfxFileList.txt`.
    pub preload_pfx_refs: Vec<String>,

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
    /// Number of model (`.ugx`) references found in the archives.
    pub models_found: usize,
    /// Paths of model references not found in the archives.
    pub models_missing: Vec<String>,
    /// Number of animation (`.uax`) references found in the archives.
    pub anims_found: usize,
    /// Paths of animation references not found in the archives.
    pub anims_missing: Vec<String>,
    /// Number of texture (`.ddx`) references found in the archives.
    pub textures_found: usize,
    /// Paths of texture references not found in the archives.
    pub textures_missing: Vec<String>,
}

/// Result of deep-validating binary assets (parse every file, not just check existence).
#[derive(Debug, Clone, Default)]
pub struct BinaryValidation {
    /// Number of models that parse without error.
    pub models_ok: usize,
    /// Models that failed to parse, as `path: error`.
    pub models_failed: Vec<String>,
    /// Paths of models not found in the archives.
    pub models_missing: Vec<String>,
    /// Number of animations that parse without error.
    pub anims_ok: usize,
    /// Animations that failed to parse, as `path: error`.
    pub anims_failed: Vec<String>,
    /// Paths of animations not found in the archives.
    pub anims_missing: Vec<String>,
    /// Number of textures that parse without error.
    pub textures_ok: usize,
    /// Textures that failed to parse, as `path: error`.
    pub textures_failed: Vec<String>,
    /// Paths of textures not found in the archives.
    pub textures_missing: Vec<String>,
}

impl BinaryValidation {
    /// Total number of errors (failed + missing).
    #[must_use]
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

    /// Clear all scenario-specific asset references.
    ///
    /// This resets lightsets, cinematics, terrain refs, sky, terrain env,
    /// minimap, sound banks, and preload lists — everything populated by
    /// `collect_scenario_assets_into` and `parse_preload_list`.
    /// Model/anim/texture refs from the base-game visual chains are preserved.
    pub fn clear_scenario_refs(&mut self) {
        self.lightset_refs.clear();
        self.cinematic_refs.clear();
        self.talking_head_refs.clear();
        self.terrain_refs.clear();
        self.sky_ref = None;
        self.terrain_env_ref = None;
        self.minimap_ref = None;
        self.sound_bank_refs.clear();
        self.preload_vis_refs.clear();
        self.preload_tfx_refs.clear();
        self.preload_pfx_refs.clear();
    }
}

/// A typed asset path that a visual references.
enum VisualAssetRef {
    /// A model file (`.ugx`).
    Model(String),
    /// An animation file (`.uax`).
    Anim(String),
    /// A damage model file (`.ugx`).
    DamageModel(String),
}

/// Visit each model, animation and damage model path in a visual.
///
/// The visit order is the order of the assets in the visual. For each asset, the model or
/// animation path comes before the damage model path.
fn for_each_visual_asset_ref(vis: &database::hw1::Visual, mut visit: impl FnMut(VisualAssetRef)) {
    for model in &vis.models {
        if let Some(comp) = &model.component {
            for asset in &comp.assets {
                visit_asset_refs(asset, &mut visit);
            }

            if let Some(logic) = &comp.logic {
                for entry in &logic.entries {
                    if let Some(asset) = &entry.asset {
                        visit_asset_refs(asset, &mut visit);
                    }
                }
            }
        }

        for anim in &model.anims {
            for asset in &anim.assets {
                visit_asset_refs(asset, &mut visit);
            }
        }
    }
}

/// Visit the typed paths of a single visual asset.
fn visit_asset_refs(asset: &database::hw1::visual::Asset, visit: &mut impl FnMut(VisualAssetRef)) {
    if let Some(file) = &asset.file {
        let normalized = file.replace('/', "\\");
        match asset.asset_type.as_str() {
            "Model" => visit(VisualAssetRef::Model(format!("art\\{normalized}.ugx"))),
            "Anim" => visit(VisualAssetRef::Anim(format!("art\\{normalized}.uax"))),
            _ => {}
        }
    }

    if let Some(dmg) = &asset.damage_file {
        visit(VisualAssetRef::DamageModel(format!(
            "art\\{}.ugx",
            dmg.replace('/', "\\")
        )));
    }
}

/// Collect all model/anim asset references from a parsed visual.
pub(crate) fn collect_visual_assets(vis: &database::hw1::Visual, manifest: &mut AssetManifest) {
    for_each_visual_asset_ref(vis, |asset| {
        match asset {
            VisualAssetRef::Model(path) => manifest.model_refs.insert(path),
            VisualAssetRef::Anim(path) => manifest.anim_refs.insert(path),
            VisualAssetRef::DamageModel(path) => manifest.damage_model_refs.insert(path),
        };
    });
}

/// Collect model/anim/damage paths from a visual into a per-object bundle.
pub(crate) fn collect_object_visual_assets(vis: &database::hw1::Visual, obj: &mut ObjectAssets) {
    for_each_visual_asset_ref(vis, |asset| match asset {
        VisualAssetRef::Model(path) => obj.models.push(path),
        VisualAssetRef::Anim(path) => obj.anims.push(path),
        VisualAssetRef::DamageModel(path) => obj.damage_models.push(path),
    });
}

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
    let Some(data) = src.resolve_exact(filename) else {
        return;
    };
    let Ok(text) = std::str::from_utf8(&data) else {
        return;
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            out.push(trimmed.to_string());
        }
    }
}

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
    collect_model_texture_refs(src, model_refs, texture_refs);
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
    collect_model_texture_refs(
        src,
        obj.models.iter().chain(&obj.damage_models),
        &mut textures,
    );
    textures.into_iter().collect()
}

/// Add the texture paths of the legacy UGX materials of each model to `texture_refs`.
///
/// The function reads only the material chunk (0x704) of each UGX file. It skips models that
/// do not resolve or do not parse, and map entries with an empty name. It normalises each
/// texture name to the `art\...\name.ddx` form.
fn collect_model_texture_refs<'a>(
    src: &mut AssetSource<impl assets::FileProvider>,
    model_paths: impl IntoIterator<Item = &'a String>,
    texture_refs: &mut BTreeSet<String>,
) {
    for ugx_path in model_paths {
        let Some(data) = src.resolve_exact(ugx_path) else {
            continue;
        };

        let Ok(materials) = ugx::read_materials(&data) else {
            continue;
        };

        for mat in &materials {
            let Some(legacy) = mat.legacy() else {
                continue;
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

/// Discover texture references from XTT terrain files.
///
/// XTT files contain three kinds of texture references:
///
/// 1. **Active textures** — splat/detail terrain textures (e.g. `arctic/snowdrift_01`).
///    The engine resolves these as `art\terrain\{filename}_df.ddx` (and `_nm`, `_sp`, etc).
/// 2. **Active decals** — projected decal textures baked into the terrain, also below
///    `art\terrain\`.
/// 3. **Foliage sets** — foliage billboard textures (e.g. `foliage\foliageset`),
///    resolved as `art\{filename}_df.ddx`, `_nm.ddx`, `_op.ddx`, `_sp.ddx`.
///
/// [`super::terrain::terrain_texture_path`] makes the active texture and decal paths.
///
/// All discovered paths are inserted into `texture_refs`.
pub(crate) fn discover_terrain_textures(
    src: &mut AssetSource<impl assets::FileProvider>,
    terrain_refs: &[String],
    texture_refs: &mut BTreeSet<String>,
) {
    for path in terrain_refs {
        if !crate::source::has_extension(path, "xtt") {
            continue;
        }
        let Some(data) = src.resolve_exact(path) else {
            continue;
        };
        let Ok(xtt_file) = xtt::Reader::read(&data) else {
            continue;
        };

        // Active textures — terrain splat layers
        for tex in &xtt_file.active_textures {
            if !tex.filename.is_empty() {
                for suffix in ["df", "nm", "sp", "em", "env"] {
                    texture_refs.insert(terrain_texture_path(&tex.filename, suffix));
                }
            }
        }

        // Active decals — baked decal textures
        for decal in &xtt_file.active_decals {
            if !decal.filename.is_empty() {
                for suffix in ["df", "nm", "op"] {
                    texture_refs.insert(terrain_texture_path(&decal.filename, suffix));
                }
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
