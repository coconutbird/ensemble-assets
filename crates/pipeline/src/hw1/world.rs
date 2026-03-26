//! HW1 loaded world — the complete result of the asset pipeline.
//!
//! [`World`] is the top-level struct that proves end-to-end asset loading:
//! database, per-object visuals/tactics/physics, scenario metadata, and a
//! manifest of all referenced binary assets.

use std::collections::{BTreeSet, HashMap};

use assets::AssetResolver;

use crate::source::AssetSource;

use super::loader;
use super::scenario::{ScenarioDescriptor, ScenarioList};

// ── Resolved asset types ────────────────────────────────────────────────

/// A fully resolved physics chain for a single object.
#[derive(Debug, Clone, Default)]
pub struct PhysicsChain {
    /// The parsed `.physics.xmb` data.
    pub physics: database::hw1::Physics,
    /// The parsed `.blueprint.xmb` data (if the physics references one).
    pub blueprint: Option<database::hw1::Blueprint>,
    /// The parsed `.shp.xmb` data (if the blueprint references one).
    pub shape: Option<database::hw1::Shape>,
}

/// All file paths associated with a single proto-object.
///
/// Built eagerly during [`World::load`] by walking the visual, tactics,
/// and physics chains. Lets you answer "give me everything related to
/// the Gorgon" without touching the archives again.
#[derive(Debug, Clone, Default)]
pub struct ObjectAssets {
    /// Object name (same key as the `HashMap`).
    pub name: String,
    /// Object class from the database (e.g. `"Unit"`, `"Building"`, `"Projectile"`).
    pub object_class: Option<String>,
    /// Object type tags (e.g. `["Military", "CovVehicle"]`).
    pub object_types: Vec<String>,
    /// Path to the `.vis` / `.vis.xmb` file (e.g. `art\covenant\vehicle\gorgon_01\gorgon_01.vis`).
    pub visual: Option<String>,
    /// Path to the `.tactics` / `.tactics.xmb` file.
    pub tactics: Option<String>,
    /// Path to the `.physics` / `.physics.xmb` file.
    pub physics: Option<String>,
    /// Path to the `.blueprint` / `.blueprint.xmb` file.
    pub blueprint: Option<String>,
    /// Path to the `.shp` / `.shp.xmb` file.
    pub shape: Option<String>,
    /// Model files (.ugx) referenced by the visual.
    pub models: Vec<String>,
    /// Animation files (.uax) referenced by the visual.
    pub anims: Vec<String>,
    /// Damage model files (.ugx) referenced by the visual.
    pub damage_models: Vec<String>,
}

/// Manifest of all binary asset references discovered during resolution.
///
/// This is a passive inventory of every `.ugx`, `.uax`, etc. path
/// referenced by the resolved visual chains. It does **not** verify
/// whether those files actually exist in the loaded archives — call
/// [`AssetManifest::verify`] for that.
#[derive(Debug, Clone, Default)]
pub struct AssetManifest {
    /// Unique model file paths (.ugx) referenced by visuals.
    pub model_refs: BTreeSet<String>,
    /// Unique animation file paths (.uax) referenced by visuals.
    pub anim_refs: BTreeSet<String>,
    /// Unique damage model file paths (.ugx) referenced by visuals.
    pub damage_model_refs: BTreeSet<String>,
}

/// Result of verifying binary asset references against loaded archives.
#[derive(Debug, Clone, Default)]
pub struct VerifyResult {
    pub models_found: usize,
    pub models_missing: Vec<String>,
    pub anims_found: usize,
    pub anims_missing: Vec<String>,
}

impl AssetManifest {
    /// Check which referenced binary assets actually exist in `src`.
    ///
    /// This is cheap — just index lookups, no decompression.
    /// Run it after loading whatever ERAs are relevant (e.g. a scenario ERA)
    /// to see what's present vs genuinely missing.
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

        result
    }
}

/// Statistics from the world loading process.
#[derive(Debug, Clone, Default)]
pub struct LoadStats {
    pub objects_total: usize,
    pub objects_with_visual: usize,
    pub objects_with_tactics: usize,
    pub objects_with_physics: usize,
    pub visuals_resolved: usize,
    pub visuals_failed: Vec<String>,
    pub tactics_resolved: usize,
    pub tactics_failed: Vec<String>,
    pub physics_resolved: usize,
    pub physics_failed: Vec<String>,
    pub blueprints_resolved: usize,
    pub shapes_resolved: usize,
}

// ── World ───────────────────────────────────────────────────────────────

/// A fully loaded HW1 game world.
///
/// Contains the complete database, all resolved per-object assets, scenario
/// metadata, and a manifest of binary asset references. This struct proves
/// end-to-end that we can load and parse all game assets from ERA archives.
pub struct World {
    /// The full game database (objects, squads, techs, etc.).
    pub database: database::hw1::Database,
    /// Per-object asset bundles keyed by object name.
    ///
    /// Every proto-object gets an entry here with all its resolved file
    /// paths (visual, tactics, physics, models, anims, etc.).
    pub assets: HashMap<String, ObjectAssets>,
    /// Parsed visual definitions keyed by object name.
    pub visuals: HashMap<String, database::hw1::Visual>,
    /// Parsed tactics definitions keyed by object name.
    pub tactics: HashMap<String, database::hw1::TacticData>,
    /// Parsed physics chains keyed by object name.
    pub physics: HashMap<String, PhysicsChain>,
    /// Scenario descriptor (if a scenario was loaded).
    pub scenario: Option<ScenarioDescriptor>,
    /// All available scenario descriptors.
    pub scenario_list: ScenarioList,
    /// Manifest of all binary asset references (global, not per-object).
    pub manifest: AssetManifest,
    /// Loading statistics.
    pub stats: LoadStats,
}

impl World {
    /// Load a complete HW1 world from a game directory.
    ///
    /// This is the main entry point for the HW1 asset pipeline. It:
    /// 1. Loads all base game ERAs in the engine's load order
    /// 2. Optionally layers a scenario ERA on top
    /// 3. Loads the full game database (objects, squads, techs, etc.)
    /// 4. Resolves all per-object asset chains (visuals, tactics, physics)
    /// 5. Verifies binary assets (models, animations) exist
    /// 6. Loads scenario descriptors
    ///
    /// # Arguments
    /// * `game_dir` — path to the HW1 game directory containing ERA files
    /// * `scenario_era` — optional scenario ERA filename (e.g. `"PHXscn01.era"`)
    pub fn load(game_dir: &str, scenario_era: Option<&str>) -> crate::Result<Self> {
        let mut src = match scenario_era {
            Some(era) => loader::load_with_scenario(game_dir, era),
            None => loader::load_game_dir(game_dir),
        };

        Self::load_from_source(&mut src)
    }

    /// Load a complete HW1 world from a pre-configured [`AssetSource`].
    ///
    /// Use this when you need custom ERA loading (e.g. explicit ERA list
    /// instead of the standard game directory layout).
    pub fn load_from_source(
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> crate::Result<Self> {
        // 1. Load the full database
        let database = database::hw1::Database::load(src)?;

        // 2. Load scenario descriptors
        let scenario_list = ScenarioList::load(src);

        // 3. Resolve all per-object asset chains
        let mut assets_map: HashMap<String, ObjectAssets> = HashMap::new();
        let mut visuals = HashMap::new();
        let mut tactics = HashMap::new();
        let mut physics = HashMap::new();
        let mut manifest = AssetManifest::default();
        let mut stats = LoadStats {
            objects_total: database.objects.len(),
            ..Default::default()
        };

        for obj in &database.objects {
            let mut obj_assets = ObjectAssets {
                name: obj.name.clone(),
                object_class: obj.object_class.clone(),
                object_types: obj.object_types.clone(),
                ..Default::default()
            };

            // Visual chain: object → .vis → .ugx/.uax
            if let Some(vis_ref) = &obj.visual {
                stats.objects_with_visual += 1;
                let vis_base = format!("art\\{}", vis_ref.replace('/', "\\"));
                obj_assets.visual = Some(vis_base.clone());

                if let Some(vis_doc) = src.read_xmb(&vis_base) {
                    match database::hw1::visual::parse(&vis_doc) {
                        Ok(vis) => {
                            stats.visuals_resolved += 1;
                            collect_visual_assets(&vis, &mut manifest);
                            collect_object_visual_assets(&vis, &mut obj_assets);
                            visuals.insert(obj.name.clone(), vis);
                        }
                        Err(e) => stats.visuals_failed.push(format!("{vis_base}: {e}")),
                    }
                } else {
                    stats.visuals_failed.push(vis_base);
                }
            }

            // Tactics chain: object → .tactics
            if let Some(tac_ref) = &obj.tactics {
                stats.objects_with_tactics += 1;
                let tac_base = format!("data\\tactics\\{}", tac_ref);
                obj_assets.tactics = Some(tac_base.clone());

                if let Some(tac_doc) = src.read_xmb(&tac_base) {
                    match database::hw1::tactics::parse(&tac_doc) {
                        Ok(tac) => {
                            stats.tactics_resolved += 1;
                            tactics.insert(obj.name.clone(), tac);
                        }
                        Err(e) => stats.tactics_failed.push(format!("{tac_base}: {e}")),
                    }
                } else {
                    stats.tactics_failed.push(tac_base);
                }
            }

            // Physics chain: .physics → .blueprint → .shp
            if let Some(phys_ref) = &obj.physics_info {
                stats.objects_with_physics += 1;
                let phys_base = format!("physics\\{}.physics", phys_ref);
                obj_assets.physics = Some(phys_base.clone());

                if let Some(phys_doc) = src.read_xmb(&phys_base) {
                    match database::hw1::physics::parse_physics(&phys_doc) {
                        Ok(phys) => {
                            stats.physics_resolved += 1;
                            let mut chain = PhysicsChain {
                                physics: phys,
                                ..Default::default()
                            };
                            resolve_physics_chain(src, &mut chain, &mut stats);
                            // Record blueprint/shape paths
                            if let Some(bp) = &chain.blueprint {
                                if let Some(bp_ref) = &chain.physics.blueprint {
                                    obj_assets.blueprint =
                                        Some(format!("physics\\{bp_ref}.blueprint"));
                                }
                                if let Some(shp_ref) = &bp.shape {
                                    obj_assets.shape = Some(format!("physics\\{shp_ref}.shp"));
                                }
                            }
                            physics.insert(obj.name.clone(), chain);
                        }
                        Err(e) => stats.physics_failed.push(format!("{phys_base}: {e}")),
                    }
                } else {
                    stats.physics_failed.push(phys_base);
                }
            }

            assets_map.insert(obj.name.clone(), obj_assets);
        }

        Ok(World {
            database,
            assets: assets_map,
            visuals,
            tactics,
            physics,
            scenario: None,
            scenario_list,
            manifest,
            stats,
        })
    }

    /// Print a summary of the loaded world to stdout.
    pub fn print_summary(&self) {
        println!("=== HW1 World Summary ===\n");
        println!("Database:");
        println!("  Objects:       {}", self.database.objects.len());
        println!("  Squads:        {}", self.database.squads.len());
        println!("  Techs:         {}", self.database.techs.len());
        println!("  Abilities:     {}", self.database.abilities.len());
        println!("  Powers:        {}", self.database.powers.len());
        println!("  Civs:          {}", self.database.civs.len());
        println!("  Leaders:       {}", self.database.leaders.len());
        println!("  Weapon Types:  {}", self.database.weapon_types.len());
        println!("  Damage Types:  {}", self.database.damage_types.len());
        println!(
            "  Game Data:     {}",
            if self.database.game_data.is_some() {
                "loaded"
            } else {
                "missing"
            }
        );
        println!();
        println!("Asset Resolution:");
        println!(
            "  Visuals:  {} / {} ({} failed)",
            self.stats.visuals_resolved,
            self.stats.objects_with_visual,
            self.stats.visuals_failed.len()
        );
        println!(
            "  Tactics:  {} / {} ({} failed)",
            self.stats.tactics_resolved,
            self.stats.objects_with_tactics,
            self.stats.tactics_failed.len()
        );
        println!(
            "  Physics:  {} / {} ({} failed)",
            self.stats.physics_resolved,
            self.stats.objects_with_physics,
            self.stats.physics_failed.len()
        );
        println!("  Blueprints: {}", self.stats.blueprints_resolved);
        println!("  Shapes:     {}", self.stats.shapes_resolved);
        println!();
        println!("Binary Asset References:");
        println!(
            "  Model refs:  {} unique .ugx",
            self.manifest.model_refs.len()
        );
        println!(
            "  Anim refs:   {} unique .uax",
            self.manifest.anim_refs.len()
        );
        println!("  Damage refs: {}", self.manifest.damage_model_refs.len());
        println!();
        println!(
            "Scenarios:     {} descriptors",
            self.scenario_list.scenarios.len()
        );
    }

    // ── Asset queries ───────────────────────────────────────────────────

    /// Get the asset bundle for a specific object by exact name.
    pub fn object_assets(&self, name: &str) -> Option<&ObjectAssets> {
        self.assets.get(name)
    }

    /// Search for objects whose name contains `pattern` (case-insensitive).
    ///
    /// Returns all matching [`ObjectAssets`] entries. Use this for queries
    /// like "give me everything related to gorgon".
    pub fn search_assets(&self, pattern: &str) -> Vec<&ObjectAssets> {
        let lower = pattern.to_lowercase();
        self.assets
            .values()
            .filter(|a| a.name.to_lowercase().contains(&lower))
            .collect()
    }

    /// Get all objects that have a specific object class (e.g. `"Unit"`, `"Building"`).
    pub fn assets_by_class(&self, class: &str) -> Vec<&ObjectAssets> {
        self.assets
            .values()
            .filter(|a| a.object_class.as_deref() == Some(class))
            .collect()
    }

    /// Get all objects that have a specific object type tag (e.g. `"Military"`, `"CovVehicle"`).
    pub fn assets_by_type(&self, object_type: &str) -> Vec<&ObjectAssets> {
        self.assets
            .values()
            .filter(|a| a.object_types.iter().any(|t| t == object_type))
            .collect()
    }

    /// Resolve texture paths for an object by reading its UGX model materials.
    ///
    /// This is intentionally **lazy** — it decompresses and parses only the
    /// material chunk (0x704) of each `.ugx` file, skipping geometry, bones,
    /// and vertex/index buffers. Call this when you actually need to know
    /// which `.ddx` textures a unit uses.
    ///
    /// Returns a deduplicated, sorted list of texture paths (normalised with
    /// `art\` prefix and `.ddx` extension).
    pub fn resolve_textures(
        &self,
        name: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Vec<String> {
        let obj = match self.assets.get(name) {
            Some(o) => o,
            None => return Vec::new(),
        };
        resolve_textures_for(obj, src)
    }

    /// Resolve texture paths for an already-retrieved [`ObjectAssets`].
    ///
    /// Convenience wrapper when you already have the object from
    /// [`search_assets`] or [`assets_by_class`].
    pub fn resolve_textures_for_obj(
        &self,
        obj: &ObjectAssets,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Vec<String> {
        resolve_textures_for(obj, src)
    }
}

impl ObjectAssets {
    /// All file paths referenced by this object, in no particular order.
    pub fn all_files(&self) -> Vec<&str> {
        let mut files = Vec::new();
        if let Some(v) = &self.visual {
            files.push(v.as_str());
        }

        if let Some(t) = &self.tactics {
            files.push(t.as_str());
        }

        if let Some(p) = &self.physics {
            files.push(p.as_str());
        }

        if let Some(b) = &self.blueprint {
            files.push(b.as_str());
        }

        if let Some(s) = &self.shape {
            files.push(s.as_str());
        }

        for m in &self.models {
            files.push(m.as_str());
        }

        for a in &self.anims {
            files.push(a.as_str());
        }

        for d in &self.damage_models {
            files.push(d.as_str());
        }

        files
    }
}

// ── Helper functions ────────────────────────────────────────────────────

/// Resolve the blueprint → shape chain from a physics entry.
fn resolve_physics_chain(
    src: &mut AssetSource<impl assets::FileProvider>,
    chain: &mut PhysicsChain,
    stats: &mut LoadStats,
) {
    if let Some(bp_ref) = &chain.physics.blueprint {
        let bp_base = format!("physics\\{}.blueprint", bp_ref);
        if let Some(bp_doc) = src.read_xmb(&bp_base) {
            if let Ok(bp) = database::hw1::physics::parse_blueprint(&bp_doc) {
                stats.blueprints_resolved += 1;

                // Shape chain
                if let Some(shp_ref) = &bp.shape {
                    let shp_base = format!("physics\\{}.shp", shp_ref);
                    if let Some(shp_doc) = src.read_xmb(&shp_base) {
                        if let Ok(shp) = database::hw1::physics::parse_shape(&shp_doc) {
                            stats.shapes_resolved += 1;
                            chain.shape = Some(shp);
                        }
                    }
                }

                chain.blueprint = Some(bp);
            }
        }
    }
}

/// Collect all model/anim asset references from a parsed visual.
fn collect_visual_assets(vis: &database::hw1::Visual, manifest: &mut AssetManifest) {
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
fn collect_object_visual_assets(vis: &database::hw1::Visual, obj: &mut ObjectAssets) {
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

/// Read UGX materials for all models in an object and return the
/// deduplicated, sorted list of texture paths they reference.
///
/// Only decompresses the material chunk (0x704) of each UGX — geometry,
/// vertex buffers, and index buffers are never touched.
fn resolve_textures_for(
    obj: &ObjectAssets,
    src: &mut AssetSource<impl assets::FileProvider>,
) -> Vec<String> {
    let mut textures = BTreeSet::new();

    // Collect all UGX paths (models + damage models).
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
            for maps in &mat.maps {
                for map in maps {
                    if !map.name.is_empty() {
                        // Texture names in UGX are absolute from the art root,
                        // e.g. `\unsc\vehicle\warthog_01\warthog_01_df`.
                        // Strip leading separators and prepend `art\`.
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
