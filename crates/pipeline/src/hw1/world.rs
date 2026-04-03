//! HW1 world — the central data structure for the entire pipeline.
//!
//! [`World`] holds the loaded database, resolved per-object asset chains,
//! scenario data, the asset manifest, cached binary assets, and the dirty
//! tracking state for the edit/save workflow.
//!
//! # Lifecycle
//!
//! ```text
//! ┌─────────┐      ┌──────────┐      ┌──────────┐
//! │  Load   │ ───▶ │  Inspect │ ───▶ │  Edit    │
//! │ (ERAs)  │      │  /Query  │      │ (guards) │
//! └─────────┘      └──────────┘      └────┬─────┘
//!                                         │
//!                                    ┌────▼─────┐
//!                                    │  Save    │
//!                                    │ (dirty)  │
//!                                    └──────────┘
//! ```
//!
//! 1. **Load** — [`World::load`] or [`World::load_from_source`] reads all
//!    ERAs, parses the database, resolves asset chains, and optionally
//!    loads a scenario.
//!
//! 2. **Inspect** — Read fields directly (`world.database.objects`,
//!    `world.visuals`, `world.manifest`, etc.) or use query helpers
//!    like [`search_assets`](World::search_assets).
//!
//! 3. **Edit** — Use `*_mut()` accessors to get [`DirtyGuard`] or
//!    [`KeyDirtyGuard`](super::edit::KeyDirtyGuard) handles. Mutations
//!    through these guards automatically track what changed.
//!
//! 4. **Save** — [`World::save`] serializes only the dirty tables/files
//!    to the override directory. Per-file saves (e.g. [`save_visual`](World::save_visual))
//!    are also available for fine-grained control.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::source::AssetSource;

use super::edit::{AssetKind, DirtyGuard, DirtySet, KeyDirtyGuard, TableId};
use super::loader;
use super::manifest::{
    AssetManifest, BinaryValidation, collect_object_visual_assets, collect_scenario_assets_into,
    collect_visual_assets, discover_terrain_textures, discover_textures, parse_preload_list,
    resolve_scenario, resolve_textures_for,
};
use super::resolve::{LoadStats, ObjectAssets, PhysicsChain, resolve_physics_chain};
use super::scenario::{ScenarioData, ScenarioDescriptor, ScenarioList};

/// A fully loaded HW1 game world: database, resolved assets, scenario, and manifest.
///
/// See the [module-level documentation](self) for the load → edit → save lifecycle.
///
/// # Public fields
///
/// Most fields are `pub` for direct read access. For **mutation**, use
/// the `*_mut()` accessors (e.g. [`objects_mut`](Self::objects_mut),
/// [`visual_mut`](Self::visual_mut)) which return dirty-tracking guards.
pub struct World {
    pub database: database::hw1::Database,
    /// Per-object resolved file paths, keyed by object name.
    pub assets: HashMap<String, ObjectAssets>,
    pub visuals: HashMap<String, database::hw1::Visual>,
    pub tactics: HashMap<String, database::hw1::TacticData>,
    pub physics: HashMap<String, PhysicsChain>,
    pub scenario: Option<ScenarioDescriptor>,
    /// Parsed `.scn` data (placed objects, players, terrain, etc.).
    pub scenario_data: Option<ScenarioData>,
    pub scenario_list: ScenarioList,
    pub manifest: AssetManifest,
    pub stats: LoadStats,
    /// Localized string table (default: English).
    pub strings: Option<super::stringtable::StringTable>,
    /// Cached terrain heightmap/lighting (lazy-loaded on first access).
    pub terrain_data: Option<xtd::XtdFile>,
    /// Cached terrain textures/foliage/roads (lazy-loaded on first access).
    pub terrain_textures: Option<xtt::XttFile>,
    /// Cached models (UGX), keyed by game path.
    pub models: HashMap<String, ugx::UgxGeom>,
    /// Cached textures (DDX), keyed by game path.
    pub textures: HashMap<String, ddx::DdxTexture>,
    /// Cached animations (UAX), keyed by game path.
    pub animations: HashMap<String, uax::UaxFile>,
    dirty: DirtySet,
    /// Number of ERA archives in the base-game stack (before any scenario ERA).
    /// Used by [`swap_scenario`] / [`clear_scenario`] to pop scenario ERAs.
    base_era_count: usize,
}

impl World {
    /// Load the base HW1 world from a game directory.
    ///
    /// Returns the loaded world **and** the [`AssetSource`] so callers can
    /// continue resolving assets (textures, models, etc.) after loading.
    ///
    /// To load a scenario, call [`swap_scenario`](Self::swap_scenario) on
    /// the returned world:
    ///
    /// ```rust,no_run
    /// let (mut world, mut src) = pipeline::hw1::World::load("/path/to/hw1").unwrap();
    /// world.swap_scenario(&mut src, "blood_gulch");
    /// ```
    pub fn load(
        game_dir: &str,
    ) -> crate::Result<(Self, AssetSource<crate::source::StdFileProvider>)> {
        let mut src = loader::load_game_dir(game_dir);
        let mut world = Self::load_from_source(&mut src)?;
        world.base_era_count = src.era_count();
        Ok((world, src))
    }

    /// Load from a pre-configured [`AssetSource`].
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
                let vis_normalized = vis_ref.replace('/', "\\");
                let vis_normalized = vis_normalized.trim_start_matches('\\');
                let vis_base = format!("art\\{vis_normalized}");
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

        // 4. Parse preload lists from scenario ERAs
        parse_preload_list(src, "visFileList.txt", &mut manifest.preload_vis_refs);
        parse_preload_list(src, "pfxFileList.txt", &mut manifest.preload_pfx_refs);
        parse_preload_list(src, "tfxFileList.txt", &mut manifest.preload_tfx_refs);

        // 5. Eagerly discover texture references from UGX material chunks
        discover_textures(src, &manifest.model_refs, &mut manifest.texture_refs);
        discover_textures(src, &manifest.damage_model_refs, &mut manifest.texture_refs);

        // 6. Auto-detect and load scenario from the SCN file
        let (scenario, scenario_data) = resolve_scenario(src, &scenario_list, &mut manifest);

        // 7. Discover terrain texture references from XTT files
        discover_terrain_textures(src, &manifest.terrain_refs, &mut manifest.texture_refs);

        // 8. Load the default (English) string table from locale.era
        let strings = super::stringtable::load_default(src);

        // 9. Eagerly load terrain data if a scenario is present
        let terrain_data = scenario.as_ref().and_then(|s| {
            let path = s.xtd_path()?;
            let data = src.resolve_exact(&path)?;
            xtd::Reader::read(&data).ok()
        });
        let terrain_textures = scenario.as_ref().and_then(|s| {
            let path = s.xtt_path()?;
            let data = src.resolve_exact(&path)?;
            xtt::Reader::read(&data).ok()
        });

        Ok(World {
            database,
            assets: assets_map,
            visuals,
            tactics,
            physics,
            scenario,
            scenario_data,
            scenario_list,
            manifest,
            stats,
            strings,
            terrain_data,
            terrain_textures,
            models: HashMap::new(),
            textures: HashMap::new(),
            animations: HashMap::new(),
            dirty: DirtySet::default(),
            base_era_count: 0,
        })
    }

    /// Unload the current scenario without loading a new one.
    ///
    /// This clears all scenario-specific state (descriptor, SCN data,
    /// terrain, scenario manifest entries) while preserving the base-game
    /// database, resolved asset chains, and string table.
    ///
    /// Any scenario ERAs pushed on top of the base-game stack are
    /// automatically popped so that file resolution reverts to the
    /// base-game ERAs.
    ///
    /// # Example
    /// ```rust,no_run
    /// # let dir = "path/to/hw1";
    /// let (mut world, mut src) = pipeline::hw1::World::load(&dir).unwrap();
    /// world.swap_scenario(&mut src, "PHXscn01");
    /// // Unload the scenario, popping its ERA from the stack.
    /// world.clear_scenario(&mut src);
    /// assert!(world.scenario.is_none());
    /// ```
    pub fn clear_scenario(&mut self, src: &mut AssetSource<impl assets::FileProvider>) {
        // Pop any ERAs that were pushed on top of the base-game stack.
        while src.era_count() > self.base_era_count {
            if let Some(label) = src.pop_era() {
                println!("  Popped ERA: {label}");
            }
        }

        self.scenario = None;
        self.scenario_data = None;
        self.terrain_data = None;
        self.terrain_textures = None;

        // Clear scenario-specific manifest entries.
        self.manifest.clear_scenario_refs();

        // Reset dirty flags for scenario-related tables.
        self.dirty.clear_table(TableId::Scenario);
        self.dirty.clear_table(TableId::TerrainData);
        self.dirty.clear_table(TableId::TerrainTextures);
    }

    /// Swap the active scenario: unload the current one and load a new one.
    ///
    /// This is much cheaper than rebuilding the entire `World` because it
    /// preserves the database, all resolved asset chains (visuals, tactics,
    /// physics), and the string table. Only scenario-specific data is
    /// re-resolved from the new ERA.
    ///
    /// `scenario` accepts an ERA filename (`"blood_gulch.era"`), a map name
    /// (`"blood_gulch"`), or an SCN path — the same formats as [`World::load`].
    ///
    /// Any previously loaded scenario ERA is automatically popped from the
    /// asset source stack before pushing the new one.
    ///
    /// # Example
    /// ```rust,no_run
    /// let (mut world, mut src) = pipeline::hw1::World::load("path/to/hw1").unwrap();
    /// world.swap_scenario(&mut src, "PHXscn01");
    /// // Switch to a different map — no need to track state.
    /// world.swap_scenario(&mut src, "blood_gulch");
    /// ```
    pub fn swap_scenario(
        &mut self,
        src: &mut AssetSource<crate::source::StdFileProvider>,
        scenario: &str,
    ) {
        // 1. Unload current scenario (pop any scenario ERAs).
        self.clear_scenario(src);

        // 2. Find and load the new scenario ERA.
        if !src.load_scenario(scenario) {
            return;
        }

        // 3. Re-parse preload lists from the new scenario ERA.
        parse_preload_list(src, "visFileList.txt", &mut self.manifest.preload_vis_refs);
        parse_preload_list(src, "pfxFileList.txt", &mut self.manifest.preload_pfx_refs);
        parse_preload_list(src, "tfxFileList.txt", &mut self.manifest.preload_tfx_refs);

        // 4. Resolve the scenario from the new ERA.
        let (scenario_desc, scenario_data) =
            resolve_scenario(src, &self.scenario_list, &mut self.manifest);

        // 5. Discover terrain textures from the new scenario.
        discover_terrain_textures(
            src,
            &self.manifest.terrain_refs,
            &mut self.manifest.texture_refs,
        );

        // 6. Eagerly load terrain data if the new scenario has it.
        self.terrain_data = scenario_desc.as_ref().and_then(|s| {
            let path = s.xtd_path()?;
            let data = src.resolve_exact(&path)?;
            xtd::Reader::read(&data).ok()
        });
        self.terrain_textures = scenario_desc.as_ref().and_then(|s| {
            let path = s.xtt_path()?;
            let data = src.resolve_exact(&path)?;
            xtt::Reader::read(&data).ok()
        });

        self.scenario = scenario_desc;
        self.scenario_data = scenario_data;
    }

    /// Resolve a localized string by its `_locID`.
    ///
    /// Prefers the `_mouseKeyboard` override when present (the PC path),
    /// falling back to the default (gamepad) text.
    ///
    /// Returns `None` if no string table is loaded or the ID is not found.
    pub fn resolve_string(&self, loc_id: i32) -> Option<&str> {
        self.strings.as_ref()?.get_pc(loc_id)
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
            "  Model refs:   {} unique .ugx",
            self.manifest.model_refs.len()
        );
        println!(
            "  Anim refs:    {} unique .uax",
            self.manifest.anim_refs.len()
        );
        println!("  Damage refs:  {}", self.manifest.damage_model_refs.len());
        println!(
            "  Texture refs: {} unique .ddx",
            self.manifest.texture_refs.len()
        );
        println!();
        println!("Preload Lists:");
        println!(
            "  VIS preload:  {} entries",
            self.manifest.preload_vis_refs.len()
        );
        println!(
            "  TFX preload:  {} entries",
            self.manifest.preload_tfx_refs.len()
        );
        println!(
            "  PFX preload:  {} entries",
            self.manifest.preload_pfx_refs.len()
        );
        println!();
        println!("Scenario Refs:");
        println!("  Lightsets:    {} refs", self.manifest.lightset_refs.len());
        println!(
            "  Cinematics:   {} refs",
            self.manifest.cinematic_refs.len()
        );
        println!(
            "  Talking Heads:{} refs",
            self.manifest.talking_head_refs.len()
        );
        println!("  Terrain:      {} refs", self.manifest.terrain_refs.len());
        println!(
            "  Sound Banks:  {} refs",
            self.manifest.sound_bank_refs.len()
        );
        if let Some(sky) = &self.manifest.sky_ref {
            println!("  Sky:          {sky}");
        }
        if let Some(env) = &self.manifest.terrain_env_ref {
            println!("  TerrainEnv:   {env}");
        }
        if let Some(mm) = &self.manifest.minimap_ref {
            println!("  Minimap:      {mm}");
        }
        println!();
        println!(
            "Scenarios:     {} descriptors",
            self.scenario_list.scenarios.len()
        );
        if let Some(desc) = &self.scenario {
            println!("Active:        {}", desc.name());
        }
        if let Some(scn) = &self.scenario_data {
            println!(
                "  SCN objects:  {} placed, {} players, {} positions",
                scn.objects().len(),
                scn.players().len(),
                scn.positions().len()
            );
        }
        if self.terrain_data.is_some() || self.terrain_textures.is_some() {
            println!(
                "  XTD loaded:   {}   XTT loaded: {}",
                self.terrain_data.is_some(),
                self.terrain_textures.is_some()
            );
        }
        if let Some(st) = &self.strings {
            println!(
                "\nString Table:  {} strings ({})",
                st.len(),
                st.language_name,
            );
        }
    }

    /// Get the asset bundle for a specific object by exact name.
    pub fn object_assets(&self, name: &str) -> Option<&ObjectAssets> {
        self.assets.get(name)
    }

    /// Search for objects whose name contains `pattern` (case-insensitive).
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

    /// Read and parse the XTD terrain heightmap/lighting for a scenario (lazy).
    pub fn read_terrain_data(
        &self,
        scenario: &ScenarioDescriptor,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<xtd::XtdFile> {
        let path = scenario.xtd_path()?;
        let data = src.resolve_exact(&path)?;
        xtd::Reader::read(&data).ok()
    }

    /// Read and parse the XTT terrain textures/foliage for a scenario (lazy).
    pub fn read_terrain_textures(
        &self,
        scenario: &ScenarioDescriptor,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<xtt::XttFile> {
        let path = scenario.xtt_path()?;
        let data = src.resolve_exact(&path)?;
        xtt::Reader::read(&data).ok()
    }

    /// Resolve texture paths for an object by reading UGX material chunks (lazy).
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

    /// Like [`resolve_textures`](Self::resolve_textures) but takes an [`ObjectAssets`] directly.
    pub fn resolve_textures_for_obj(
        &self,
        obj: &ObjectAssets,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Vec<String> {
        resolve_textures_for(obj, src)
    }

    /// Deep-validate all manifest binary assets (parse each `.ugx`, `.uax`, `.ddx`).
    pub fn validate_binary_assets(
        &self,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> BinaryValidation {
        let mut result = BinaryValidation::default();

        // Validate models (including damage models)
        let all_models = self
            .manifest
            .model_refs
            .iter()
            .chain(self.manifest.damage_model_refs.iter());
        for path in all_models {
            match src.resolve_exact(path) {
                Some(data) => match ugx::UgxGeom::from_bytes(&data) {
                    Ok(_) => result.models_ok += 1,
                    Err(e) => result.models_failed.push(format!("{path}: {e}")),
                },
                None => result.models_missing.push(path.clone()),
            }
        }

        for path in &self.manifest.anim_refs {
            match src.resolve_exact(path) {
                Some(data) => match uax::UaxFile::from_bytes(&data) {
                    Ok(_) => result.anims_ok += 1,
                    Err(e) => result.anims_failed.push(format!("{path}: {e}")),
                },
                None => result.anims_missing.push(path.clone()),
            }
        }

        for path in &self.manifest.texture_refs {
            match src.resolve_exact(path) {
                Some(data) => match ddx::DdxTexture::from_bytes(&data) {
                    Ok(_) => result.textures_ok += 1,
                    Err(e) => result.textures_failed.push(format!("{path}: {e}")),
                },
                None => result.textures_missing.push(path.clone()),
            }
        }

        result
    }

    /// Read and parse the `.scn` file for a scenario descriptor (lazy).
    pub fn read_scenario(
        &self,
        scenario: &ScenarioDescriptor,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<ScenarioData> {
        scenario.read_scenario(src)
    }

    /// Collect all asset references from a parsed scenario into the manifest.
    pub fn collect_scenario_assets(&mut self, scenario: &ScenarioDescriptor, scn: &ScenarioData) {
        collect_scenario_assets_into(scenario, scn, &mut self.manifest);
    }

    /// Read and parse a UGX model file (lazy).
    pub fn read_model(
        &self,
        path: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<ugx::UgxGeom> {
        let data = src.resolve_exact(path)?;
        ugx::UgxGeom::from_bytes(&data).ok()
    }

    /// Read and parse all models for a named object (lazy).
    pub fn read_object_models(
        &self,
        name: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Vec<(String, ugx::UgxGeom)> {
        let obj = match self.assets.get(name) {
            Some(o) => o,
            None => return Vec::new(),
        };
        obj.models
            .iter()
            .filter_map(|path| {
                let data = src.resolve_exact(path)?;
                let geom = ugx::UgxGeom::from_bytes(&data).ok()?;
                Some((path.clone(), geom))
            })
            .collect()
    }

    /// Read and parse a UAX animation file (lazy).
    pub fn read_animation(
        &self,
        path: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<uax::UaxFile> {
        let data = src.resolve_exact(path)?;
        uax::UaxFile::from_bytes(&data).ok()
    }

    /// Read and parse all animations for a named object (lazy).
    pub fn read_object_animations(
        &self,
        name: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Vec<(String, uax::UaxFile)> {
        let obj = match self.assets.get(name) {
            Some(o) => o,
            None => return Vec::new(),
        };
        obj.anims
            .iter()
            .filter_map(|path| {
                let data = src.resolve_exact(path)?;
                let anim = uax::UaxFile::from_bytes(&data).ok()?;
                Some((path.clone(), anim))
            })
            .collect()
    }

    /// Read and parse a DDX texture file (lazy).
    pub fn read_texture(
        &self,
        path: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<ddx::DdxTexture> {
        let data = src.resolve_exact(path)?;
        ddx::DdxTexture::from_bytes(&data).ok()
    }

    // ---- Edit API ----
    //
    // Each `*_mut()` method returns a [`DirtyGuard`] (table-level) or
    // [`KeyDirtyGuard`] (per-file) that dereferences to `&mut T`.
    // Modifying the data through the guard automatically marks the
    // corresponding table dirty so that [`save`](Self::save) knows
    // what to write.

    /// Whether any table has been modified since load (or last save).
    pub fn is_dirty(&self) -> bool {
        self.dirty.is_any_dirty()
    }

    /// Which tables have been modified.
    pub fn dirty_tables(&self) -> Vec<TableId> {
        self.dirty.dirty_tables()
    }

    /// Get a mutable reference to the objects table (marks dirty on drop).
    pub fn objects_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::ProtoObject>> {
        DirtyGuard::new(
            &mut self.database.objects,
            self.dirty.flag(TableId::Objects),
        )
    }

    /// Get a mutable reference to the squads table.
    pub fn squads_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Squad>> {
        DirtyGuard::new(&mut self.database.squads, self.dirty.flag(TableId::Squads))
    }

    /// Get a mutable reference to the techs table.
    pub fn techs_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Tech>> {
        DirtyGuard::new(&mut self.database.techs, self.dirty.flag(TableId::Techs))
    }

    /// Get a mutable reference to the abilities table.
    pub fn abilities_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Ability>> {
        DirtyGuard::new(
            &mut self.database.abilities,
            self.dirty.flag(TableId::Abilities),
        )
    }

    /// Get a mutable reference to the powers table.
    pub fn powers_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Power>> {
        DirtyGuard::new(&mut self.database.powers, self.dirty.flag(TableId::Powers))
    }

    /// Get a mutable reference to the civs table.
    pub fn civs_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Civ>> {
        DirtyGuard::new(&mut self.database.civs, self.dirty.flag(TableId::Civs))
    }

    /// Get a mutable reference to the leaders table.
    pub fn leaders_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Leader>> {
        DirtyGuard::new(
            &mut self.database.leaders,
            self.dirty.flag(TableId::Leaders),
        )
    }

    /// Get a mutable reference to the weapon types table.
    pub fn weapon_types_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::WeaponType>> {
        DirtyGuard::new(
            &mut self.database.weapon_types,
            self.dirty.flag(TableId::WeaponTypes),
        )
    }

    /// Get a mutable reference to the damage types table.
    pub fn damage_types_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::DamageType>> {
        DirtyGuard::new(
            &mut self.database.damage_types,
            self.dirty.flag(TableId::DamageTypes),
        )
    }

    /// Get a mutable reference to the game data singleton.
    pub fn game_data_mut(&mut self) -> DirtyGuard<'_, Option<database::hw1::GameData>> {
        DirtyGuard::new(
            &mut self.database.game_data,
            self.dirty.flag(TableId::GameData),
        )
    }

    /// Get a mutable reference to the scenario data (map-specific settings).
    pub fn scenario_data_mut(&mut self) -> DirtyGuard<'_, Option<ScenarioData>> {
        DirtyGuard::new(&mut self.scenario_data, self.dirty.flag(TableId::Scenario))
    }

    /// Get a mutable reference to all resolved visuals (marks entire table dirty).
    pub fn visuals_mut(&mut self) -> DirtyGuard<'_, HashMap<String, database::hw1::Visual>> {
        DirtyGuard::new(&mut self.visuals, self.dirty.flag(TableId::Visuals))
    }

    /// Get a mutable reference to all resolved tactics (marks entire table dirty).
    pub fn tactics_mut(&mut self) -> DirtyGuard<'_, HashMap<String, database::hw1::TacticData>> {
        DirtyGuard::new(&mut self.tactics, self.dirty.flag(TableId::Tactics))
    }

    /// Get a mutable reference to all resolved physics chains.
    pub fn physics_mut(&mut self) -> DirtyGuard<'_, HashMap<String, PhysicsChain>> {
        DirtyGuard::new(&mut self.physics, self.dirty.flag(TableId::Physics))
    }

    /// Get a mutable reference to terrain visual data (XTD).
    pub fn terrain_data_mut(&mut self) -> DirtyGuard<'_, Option<xtd::XtdFile>> {
        DirtyGuard::new(
            &mut self.terrain_data,
            self.dirty.flag(TableId::TerrainData),
        )
    }

    /// Get a mutable reference to terrain textures (XTT).
    pub fn terrain_textures_mut(&mut self) -> DirtyGuard<'_, Option<xtt::XttFile>> {
        DirtyGuard::new(
            &mut self.terrain_textures,
            self.dirty.flag(TableId::TerrainTextures),
        )
    }

    /// Get a mutable reference to the string table (localized text).
    pub fn strings_mut(&mut self) -> DirtyGuard<'_, Option<super::stringtable::StringTable>> {
        DirtyGuard::new(&mut self.strings, self.dirty.flag(TableId::Strings))
    }

    // ---- Per-key mutable accessors ----

    /// Get a mutable reference to a single visual, marking only that key dirty.
    ///
    /// Returns `None` if the object name isn't in the visuals map.
    pub fn visual_mut(&mut self, name: &str) -> Option<KeyDirtyGuard<'_, database::hw1::Visual>> {
        let vis = self.visuals.get_mut(name)?;
        Some(KeyDirtyGuard::new(
            vis,
            &self.dirty,
            TableId::Visuals,
            name.to_string(),
        ))
    }

    /// Get a mutable reference to a single tactics entry, marking only that key dirty.
    ///
    /// Returns `None` if the object name isn't in the tactics map.
    pub fn tactic_mut(
        &mut self,
        name: &str,
    ) -> Option<KeyDirtyGuard<'_, database::hw1::TacticData>> {
        let tac = self.tactics.get_mut(name)?;
        Some(KeyDirtyGuard::new(
            tac,
            &self.dirty,
            TableId::Tactics,
            name.to_string(),
        ))
    }

    /// Get a mutable reference to a single physics chain, marking only that key dirty.
    ///
    /// Returns `None` if the object name isn't in the physics map.
    pub fn physics_entry_mut(&mut self, name: &str) -> Option<KeyDirtyGuard<'_, PhysicsChain>> {
        let chain = self.physics.get_mut(name)?;
        Some(KeyDirtyGuard::new(
            chain,
            &self.dirty,
            TableId::Physics,
            name.to_string(),
        ))
    }

    /// Get a mutable reference to a cached model by game path.
    ///
    /// The model must have been loaded via `load_model()` first.
    pub fn model_mut(&mut self, path: &str) -> Option<KeyDirtyGuard<'_, ugx::UgxGeom>> {
        let geom = self.models.get_mut(path)?;
        Some(KeyDirtyGuard::new(
            geom,
            &self.dirty,
            TableId::Models,
            path.to_string(),
        ))
    }

    /// Get a mutable reference to a cached texture by game path.
    ///
    /// The texture must have been loaded via `load_texture()` first.
    pub fn texture_mut(&mut self, path: &str) -> Option<KeyDirtyGuard<'_, ddx::DdxTexture>> {
        let tex = self.textures.get_mut(path)?;
        Some(KeyDirtyGuard::new(
            tex,
            &self.dirty,
            TableId::Textures,
            path.to_string(),
        ))
    }

    /// Get a mutable reference to a cached animation by game path.
    ///
    /// The animation must have been loaded via `load_animation()` first.
    pub fn animation_mut(&mut self, path: &str) -> Option<KeyDirtyGuard<'_, uax::UaxFile>> {
        let anim = self.animations.get_mut(path)?;
        Some(KeyDirtyGuard::new(
            anim,
            &self.dirty,
            TableId::Animations,
            path.to_string(),
        ))
    }

    // ---- Lazy load into cache ----

    /// Load a model into the cache and return a reference.
    ///
    /// If already cached, returns the cached version.
    pub fn load_model(
        &mut self,
        path: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<&ugx::UgxGeom> {
        if !self.models.contains_key(path) {
            let data = src.resolve_exact(path)?;
            let geom = ugx::UgxGeom::from_bytes(&data).ok()?;
            self.models.insert(path.to_string(), geom);
        }
        self.models.get(path)
    }

    /// Load a texture into the cache and return a reference.
    ///
    /// If already cached, returns the cached version.
    pub fn load_texture(
        &mut self,
        path: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<&ddx::DdxTexture> {
        if !self.textures.contains_key(path) {
            let data = src.resolve_exact(path)?;
            let tex = ddx::DdxTexture::from_bytes(&data).ok()?;
            self.textures.insert(path.to_string(), tex);
        }
        self.textures.get(path)
    }

    /// Load an animation into the cache and return a reference.
    ///
    /// If already cached, returns the cached version.
    pub fn load_animation(
        &mut self,
        path: &str,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<&uax::UaxFile> {
        if !self.animations.contains_key(path) {
            let data = src.resolve_exact(path)?;
            let anim = uax::UaxFile::from_bytes(&data).ok()?;
            self.animations.insert(path.to_string(), anim);
        }
        self.animations.get(path)
    }

    // ---- Per-file save ----

    /// Save a single visual file to the override directory.
    ///
    /// Writes the `.vis` for the named object and clears its dirty key.
    /// Returns the path written, or an error if the object/path is unknown.
    pub fn save_visual(
        &mut self,
        name: &str,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let vis = self
            .visuals
            .get(name)
            .ok_or_else(|| format!("unknown visual: {name}"))?;
        let game_path = self
            .assets
            .get(name)
            .and_then(|oa| oa.visual.as_ref())
            .ok_or_else(|| format!("no visual path for object: {name}"))?;
        let doc = database::hw1::visual::to_document(vis)
            .map_err(|e| format!("serialize visual {name}: {e}"))?;
        let path = src.write_xmb(game_path, &doc)?;
        self.dirty.clear_key(TableId::Visuals, name);
        Ok(path)
    }

    /// Save a single tactics file to the override directory.
    ///
    /// Writes the tactics XML for the named object and clears its dirty key.
    pub fn save_tactic(
        &mut self,
        name: &str,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let tac = self
            .tactics
            .get(name)
            .ok_or_else(|| format!("unknown tactics: {name}"))?;
        let game_path = self
            .assets
            .get(name)
            .and_then(|oa| oa.tactics.as_ref())
            .ok_or_else(|| format!("no tactics path for object: {name}"))?;
        let doc = database::hw1::tactics::to_document(tac)
            .map_err(|e| format!("serialize tactics {name}: {e}"))?;
        let path = src.write_xmb(game_path, &doc)?;
        self.dirty.clear_key(TableId::Tactics, name);
        Ok(path)
    }

    /// Save a single physics chain (`.physics`, `.blueprint`, `.shp`) to
    /// the override directory.
    ///
    /// Writes all files in the chain and clears its dirty key.
    pub fn save_physics(
        &mut self,
        name: &str,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<Vec<PathBuf>, String> {
        let chain = self
            .physics
            .get(name)
            .ok_or_else(|| format!("unknown physics: {name}"))?;
        let oa = self
            .assets
            .get(name)
            .ok_or_else(|| format!("no asset paths for object: {name}"))?;
        let mut written = Vec::new();
        if let Some(ref game_path) = oa.physics {
            let doc = database::hw1::physics::physics_to_document(&chain.physics)
                .map_err(|e| format!("serialize physics {name}: {e}"))?;
            written.push(src.write_xmb(game_path, &doc)?);
        }
        if let (Some(bp), Some(game_path)) = (&chain.blueprint, &oa.blueprint) {
            let doc = database::hw1::physics::blueprint_to_document(bp)
                .map_err(|e| format!("serialize blueprint {name}: {e}"))?;
            written.push(src.write_xmb(game_path, &doc)?);
        }
        if let (Some(shp), Some(game_path)) = (&chain.shape, &oa.shape) {
            let doc = database::hw1::physics::shape_to_document(shp)
                .map_err(|e| format!("serialize shape {name}: {e}"))?;
            written.push(src.write_xmb(game_path, &doc)?);
        }
        self.dirty.clear_key(TableId::Physics, name);
        Ok(written)
    }

    /// Save the terrain heightmap/lighting (XTD) to the override directory.
    ///
    /// Writes the binary XTD file and clears the dirty flag.
    pub fn save_terrain_data(
        &mut self,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let xtd = self.terrain_data.as_ref().ok_or("no terrain data loaded")?;
        let scenario = self.scenario.as_ref().ok_or("no scenario loaded")?;
        let game_path = scenario.xtd_path().ok_or("scenario has no XTD path")?;
        let bytes = xtd::Writer::write(xtd).map_err(|e| format!("serialize terrain data: {e}"))?;
        let path = src.write_file(&game_path, &bytes)?;
        self.dirty.flag(TableId::TerrainData).set(false);
        Ok(path)
    }

    /// Save the terrain textures/foliage/roads (XTT) to the override directory.
    ///
    /// Writes the binary XTT file and clears the dirty flag.
    pub fn save_terrain_textures(
        &mut self,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let xtt = self
            .terrain_textures
            .as_ref()
            .ok_or("no terrain textures loaded")?;
        let scenario = self.scenario.as_ref().ok_or("no scenario loaded")?;
        let game_path = scenario.xtt_path().ok_or("scenario has no XTT path")?;
        let bytes =
            xtt::Writer::write(xtt).map_err(|e| format!("serialize terrain textures: {e}"))?;
        let path = src.write_file(&game_path, &bytes)?;
        self.dirty.flag(TableId::TerrainTextures).set(false);
        Ok(path)
    }

    /// Save the string table to the override directory.
    ///
    /// Writes the XMB string table and clears the dirty flag.
    pub fn save_strings(
        &mut self,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let st = self.strings.as_ref().ok_or("no string table loaded")?;
        let doc = st.to_document();
        let game_path = st.game_path();
        let path = src.write_xmb(&game_path, &doc)?;
        self.dirty.flag(TableId::Strings).set(false);
        Ok(path)
    }

    /// Save a single cached model (UGX) to the override directory.
    ///
    /// Writes the binary UGX file and clears the dirty key.
    pub fn save_model(
        &mut self,
        game_path: &str,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let geom = self
            .models
            .get(game_path)
            .ok_or_else(|| format!("model not cached: {game_path}"))?;
        let bytes = ugx::Writer::write(geom, ugx::UgxVersion::Hw1)
            .map_err(|e| format!("serialize model {game_path}: {e}"))?;
        let path = src.write_file(game_path, &bytes)?;
        self.dirty.clear_key(TableId::Models, game_path);
        Ok(path)
    }

    /// Save a single cached texture (DDX) to the override directory.
    ///
    /// Writes the DDS file and clears the dirty key.
    pub fn save_texture(
        &mut self,
        game_path: &str,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let tex = self
            .textures
            .get(game_path)
            .ok_or_else(|| format!("texture not cached: {game_path}"))?;
        let bytes =
            ddx::Writer::write(tex).map_err(|e| format!("serialize texture {game_path}: {e}"))?;
        let path = src.write_file(game_path, &bytes)?;
        self.dirty.clear_key(TableId::Textures, game_path);
        Ok(path)
    }

    /// Save a single cached animation (UAX) to the override directory.
    ///
    /// Writes the binary UAX file and clears the dirty key.
    pub fn save_animation(
        &mut self,
        game_path: &str,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let anim = self
            .animations
            .get(game_path)
            .ok_or_else(|| format!("animation not cached: {game_path}"))?;
        let bytes = anim.to_bytes();
        let path = src.write_file(game_path, &bytes)?;
        self.dirty.clear_key(TableId::Animations, game_path);
        Ok(path)
    }

    /// Save all dirty tables to the override directory.
    ///
    /// Iterates over every table flagged as dirty and serializes it to
    /// `{override_dir}/{era_label}/{game_path}`. For per-file tables
    /// (visuals, tactics, physics, models, textures, animations), only
    /// the specific keys that were modified are written.
    ///
    /// Returns the list of files written. Clears dirty flags on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the `AssetSource` has no override directory
    /// set, or if serialization/IO fails for any table.
    pub fn save(
        &mut self,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<Vec<PathBuf>, String> {
        let dirty = self.dirty.dirty_tables();
        if dirty.is_empty() {
            return Ok(Vec::new());
        }

        let mut written = Vec::new();

        for table in &dirty {
            match table {
                TableId::Objects => {
                    let doc = self
                        .database
                        .to_document_single("Objects", "Object", &self.database.objects)
                        .map_err(|e| format!("serialize objects: {e}"))?;
                    written.push(src.write_xmb("data\\objects.xml", &doc)?);
                }
                TableId::Squads => {
                    let doc = self
                        .database
                        .to_document_single("Squads", "Squad", &self.database.squads)
                        .map_err(|e| format!("serialize squads: {e}"))?;
                    written.push(src.write_xmb("data\\squads.xml", &doc)?);
                }
                TableId::Techs => {
                    let doc = self
                        .database
                        .to_document_single("TechTree", "Tech", &self.database.techs)
                        .map_err(|e| format!("serialize techs: {e}"))?;
                    written.push(src.write_xmb("data\\techs.xml", &doc)?);
                }
                TableId::Abilities => {
                    let doc = self
                        .database
                        .to_document_single("Abilities", "Ability", &self.database.abilities)
                        .map_err(|e| format!("serialize abilities: {e}"))?;
                    written.push(src.write_xmb("data\\abilities.xml", &doc)?);
                }
                TableId::Powers => {
                    let doc = self
                        .database
                        .to_document_single("Powers", "Power", &self.database.powers)
                        .map_err(|e| format!("serialize powers: {e}"))?;
                    written.push(src.write_xmb("data\\powers.xml", &doc)?);
                }
                TableId::Civs => {
                    let doc = self
                        .database
                        .to_document_single("Civs", "Civ", &self.database.civs)
                        .map_err(|e| format!("serialize civs: {e}"))?;
                    written.push(src.write_xmb("data\\civs.xml", &doc)?);
                }
                TableId::Leaders => {
                    let doc = self
                        .database
                        .to_document_single("Leaders", "Leader", &self.database.leaders)
                        .map_err(|e| format!("serialize leaders: {e}"))?;
                    written.push(src.write_xmb("data\\leaders.xml", &doc)?);
                }
                TableId::WeaponTypes => {
                    let doc = self
                        .database
                        .to_document_single(
                            "WeaponTypes",
                            "WeaponType",
                            &self.database.weapon_types,
                        )
                        .map_err(|e| format!("serialize weapon_types: {e}"))?;
                    written.push(src.write_xmb("data\\weapontypes.xml", &doc)?);
                }
                TableId::DamageTypes => {
                    let doc = self
                        .database
                        .to_document_single(
                            "DamageTypes",
                            "DamageType",
                            &self.database.damage_types,
                        )
                        .map_err(|e| format!("serialize damage_types: {e}"))?;
                    written.push(src.write_xmb("data\\damagetypes.xml", &doc)?);
                }
                TableId::GameData => {
                    if let Some(ref gd) = self.database.game_data {
                        let node = bdt_serde::to_node("GameData", gd)
                            .map_err(|e| format!("serialize gamedata: {e}"))?;
                        let doc = xmb::Document::with_root(node);
                        written.push(src.write_xmb("data\\gamedata.xml", &doc)?);
                    }
                }
                TableId::Scenario => {
                    if let (Some(desc), Some(scn)) = (&self.scenario, &self.scenario_data) {
                        let doc = scn
                            .to_document()
                            .map_err(|e| format!("serialize scenario: {e}"))?;
                        written.push(src.write_xmb(&desc.scn_path(), &doc)?);
                    }
                }
                TableId::Visuals => {
                    let keys = self.dirty.dirty_keys(TableId::Visuals);
                    let iter: Box<dyn Iterator<Item = (&String, &database::hw1::Visual)>> =
                        if keys.is_empty() {
                            // Whole-table dirty (via visuals_mut()) — save all.
                            Box::new(self.visuals.iter())
                        } else {
                            Box::new(
                                self.visuals
                                    .iter()
                                    .filter(|(k, _)| keys.contains(k.as_str())),
                            )
                        };
                    for (obj_name, vis) in iter {
                        if let Some(game_path) =
                            self.assets.get(obj_name).and_then(|oa| oa.visual.as_ref())
                        {
                            let doc = database::hw1::visual::to_document(vis)
                                .map_err(|e| format!("serialize visual {obj_name}: {e}"))?;
                            written.push(src.write_xmb(game_path, &doc)?);
                        }
                    }
                }
                TableId::Tactics => {
                    let keys = self.dirty.dirty_keys(TableId::Tactics);
                    let iter: Box<dyn Iterator<Item = (&String, &database::hw1::TacticData)>> =
                        if keys.is_empty() {
                            Box::new(self.tactics.iter())
                        } else {
                            Box::new(
                                self.tactics
                                    .iter()
                                    .filter(|(k, _)| keys.contains(k.as_str())),
                            )
                        };
                    for (obj_name, tac) in iter {
                        if let Some(game_path) =
                            self.assets.get(obj_name).and_then(|oa| oa.tactics.as_ref())
                        {
                            let doc = database::hw1::tactics::to_document(tac)
                                .map_err(|e| format!("serialize tactics {obj_name}: {e}"))?;
                            written.push(src.write_xmb(game_path, &doc)?);
                        }
                    }
                }
                TableId::Physics => {
                    let keys = self.dirty.dirty_keys(TableId::Physics);
                    let iter: Box<dyn Iterator<Item = (&String, &PhysicsChain)>> =
                        if keys.is_empty() {
                            Box::new(self.physics.iter())
                        } else {
                            Box::new(
                                self.physics
                                    .iter()
                                    .filter(|(k, _)| keys.contains(k.as_str())),
                            )
                        };
                    for (obj_name, chain) in iter {
                        let Some(oa) = self.assets.get(obj_name) else {
                            continue;
                        };
                        // Write .physics
                        if let Some(ref game_path) = oa.physics {
                            let doc = database::hw1::physics::physics_to_document(&chain.physics)
                                .map_err(|e| format!("serialize physics {obj_name}: {e}"))?;
                            written.push(src.write_xmb(game_path, &doc)?);
                        }
                        // Write .blueprint
                        if let (Some(bp), Some(game_path)) = (&chain.blueprint, &oa.blueprint) {
                            let doc = database::hw1::physics::blueprint_to_document(bp)
                                .map_err(|e| format!("serialize blueprint {obj_name}: {e}"))?;
                            written.push(src.write_xmb(game_path, &doc)?);
                        }
                        // Write .shp
                        if let (Some(shp), Some(game_path)) = (&chain.shape, &oa.shape) {
                            let doc = database::hw1::physics::shape_to_document(shp)
                                .map_err(|e| format!("serialize shape {obj_name}: {e}"))?;
                            written.push(src.write_xmb(game_path, &doc)?);
                        }
                    }
                }
                TableId::TerrainData => {
                    if let Some(ref xtd) = self.terrain_data
                        && let Some(ref scenario) = self.scenario
                        && let Some(game_path) = scenario.xtd_path()
                    {
                        let bytes = xtd::Writer::write(xtd)
                            .map_err(|e| format!("serialize terrain data: {e}"))?;
                        written.push(src.write_file(&game_path, &bytes)?);
                    }
                }
                TableId::TerrainTextures => {
                    if let Some(ref xtt) = self.terrain_textures
                        && let Some(ref scenario) = self.scenario
                        && let Some(game_path) = scenario.xtt_path()
                    {
                        let bytes = xtt::Writer::write(xtt)
                            .map_err(|e| format!("serialize terrain textures: {e}"))?;
                        written.push(src.write_file(&game_path, &bytes)?);
                    }
                }
                TableId::Strings => {
                    if let Some(ref st) = self.strings {
                        let doc = st.to_document();
                        let game_path = st.game_path();
                        written.push(src.write_xmb(&game_path, &doc)?);
                    }
                }
                TableId::Models => {
                    let keys = self.dirty.dirty_keys(TableId::Models);
                    let iter: Box<dyn Iterator<Item = (&String, &ugx::UgxGeom)>> =
                        if keys.is_empty() {
                            Box::new(self.models.iter())
                        } else {
                            Box::new(
                                self.models
                                    .iter()
                                    .filter(|(k, _)| keys.contains(k.as_str())),
                            )
                        };
                    for (game_path, geom) in iter {
                        let bytes = ugx::Writer::write(geom, ugx::UgxVersion::Hw1)
                            .map_err(|e| format!("serialize model {game_path}: {e}"))?;
                        written.push(src.write_file(game_path, &bytes)?);
                    }
                }
                TableId::Textures => {
                    let keys = self.dirty.dirty_keys(TableId::Textures);
                    let iter: Box<dyn Iterator<Item = (&String, &ddx::DdxTexture)>> =
                        if keys.is_empty() {
                            Box::new(self.textures.iter())
                        } else {
                            Box::new(
                                self.textures
                                    .iter()
                                    .filter(|(k, _)| keys.contains(k.as_str())),
                            )
                        };
                    for (game_path, tex) in iter {
                        let bytes = ddx::Writer::write(tex)
                            .map_err(|e| format!("serialize texture {game_path}: {e}"))?;
                        written.push(src.write_file(game_path, &bytes)?);
                    }
                }
                TableId::Animations => {
                    let keys = self.dirty.dirty_keys(TableId::Animations);
                    let iter: Box<dyn Iterator<Item = (&String, &uax::UaxFile)>> =
                        if keys.is_empty() {
                            Box::new(self.animations.iter())
                        } else {
                            Box::new(
                                self.animations
                                    .iter()
                                    .filter(|(k, _)| keys.contains(k.as_str())),
                            )
                        };
                    for (game_path, anim) in iter {
                        let bytes = anim.to_bytes();
                        written.push(src.write_file(game_path, &bytes)?);
                    }
                }
            }
        }

        self.dirty.clear();
        Ok(written)
    }

    /// Re-parse a single database table from the asset source.
    ///
    /// This is the incremental-reload primitive: after an override file
    /// changes on disk, call this with the corresponding [`TableId`] to
    /// update just that table without a full world reload.
    ///
    /// Returns `Ok(true)` if the table was successfully reloaded,
    /// `Ok(false)` if the source file was not found (table left unchanged),
    /// or `Err` on parse failure.
    pub fn reload_table(
        &mut self,
        table: TableId,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> crate::Result<bool> {
        match table {
            TableId::Objects => {
                if let Some(doc) = src.read_xmb("data\\objects.xml") {
                    self.database.load_objects(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Squads => {
                if let Some(doc) = src.read_xmb("data\\squads.xml") {
                    self.database.load_squads(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Techs => {
                if let Some(doc) = src.read_xmb("data\\techs.xml") {
                    self.database.load_techs(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Abilities => {
                if let Some(doc) = src.read_xmb("data\\abilities.xml") {
                    self.database.load_abilities(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Powers => {
                if let Some(doc) = src.read_xmb("data\\powers.xml") {
                    self.database.load_powers(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Civs => {
                if let Some(doc) = src.read_xmb("data\\civs.xml") {
                    self.database.load_civs(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Leaders => {
                if let Some(doc) = src.read_xmb("data\\leaders.xml") {
                    self.database.load_leaders(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::WeaponTypes => {
                if let Some(doc) = src.read_xmb("data\\weapontypes.xml") {
                    self.database.load_weapon_types(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::DamageTypes => {
                if let Some(doc) = src.read_xmb("data\\damagetypes.xml") {
                    self.database.load_damage_types(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::GameData => {
                if let Some(doc) = src.read_xmb("data\\gamedata.xml") {
                    self.database.load_game_data(&doc)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Scenario => {
                // Scenario reload requires re-reading the .scn file
                if let Some(desc) = &self.scenario
                    && let Some(scn) = desc.read_scenario(src)
                {
                    self.scenario_data = Some(scn);
                    return Ok(true);
                }
                Ok(false)
            }
            TableId::Visuals
            | TableId::Tactics
            | TableId::Physics
            | TableId::Models
            | TableId::Animations
            | TableId::Textures => {
                // These are per-file — use reload_asset() instead.
                Ok(false)
            }
            TableId::Strings => {
                if let Some(ref st) = self.strings {
                    let lang = st.language_code.clone();
                    self.strings = super::stringtable::StringTable::load(&lang, src);
                    return Ok(self.strings.is_some());
                }
                Ok(false)
            }
            TableId::TerrainData => {
                if let Some(ref scenario) = self.scenario
                    && let Some(path) = scenario.xtd_path()
                    && let Some(data) = src.resolve_exact(&path)
                {
                    self.terrain_data = xtd::Reader::read(&data).ok();
                    return Ok(true);
                }
                Ok(false)
            }
            TableId::TerrainTextures => {
                if let Some(ref scenario) = self.scenario
                    && let Some(path) = scenario.xtt_path()
                    && let Some(data) = src.resolve_exact(&path)
                {
                    self.terrain_textures = xtt::Reader::read(&data).ok();
                    return Ok(true);
                }
                Ok(false)
            }
        }
    }

    // ---- Per-asset incremental reload ----

    /// Find which object name(s) reference a given game path.
    ///
    /// Searches the [`ObjectAssets`] map for visual, tactics, physics,
    /// blueprint, shape, model, animation, and damage-model references
    /// that match `game_path` (case-insensitive).
    pub fn owners_of_asset(&self, game_path: &str) -> Vec<String> {
        let needle = game_path.to_ascii_lowercase();
        self.assets
            .iter()
            .filter(|(_, oa)| {
                let paths = oa
                    .visual
                    .iter()
                    .chain(oa.tactics.iter())
                    .chain(oa.physics.iter())
                    .chain(oa.blueprint.iter())
                    .chain(oa.shape.iter())
                    .chain(oa.models.iter())
                    .chain(oa.anims.iter())
                    .chain(oa.damage_models.iter());
                paths.into_iter().any(|p| p.to_ascii_lowercase() == needle)
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Reload any asset by its [`AssetKind`].
    ///
    /// This is the universal incremental-reload entry point. It
    /// dispatches to `reload_table` for database XMLs, re-parses
    /// per-object XML files (visuals, tactics, physics), and
    /// invalidates binary asset caches.
    ///
    /// Returns `Ok(true)` if something was updated.
    pub fn reload_asset(
        &mut self,
        kind: &AssetKind,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> crate::Result<bool> {
        match kind {
            AssetKind::DatabaseTable(tid) => self.reload_table(*tid, src),

            AssetKind::Visual(path) => {
                let owners = self.owners_of_asset(path);
                if owners.is_empty() {
                    return Ok(false);
                }
                let doc = match src.read_xmb(path) {
                    Some(d) => d,
                    None => return Ok(false),
                };
                let vis = database::hw1::visual::parse(&doc)?;
                for owner in &owners {
                    // Update manifest + per-object asset refs.
                    if let Some(oa) = self.assets.get_mut(owner) {
                        oa.models.clear();
                        oa.anims.clear();
                        oa.damage_models.clear();
                        collect_object_visual_assets(&vis, oa);
                    }
                    collect_visual_assets(&vis, &mut self.manifest);
                    self.visuals.insert(owner.clone(), vis.clone());
                }
                Ok(true)
            }

            AssetKind::Tactics(path) => {
                let owners = self.owners_of_asset(path);
                if owners.is_empty() {
                    return Ok(false);
                }
                let doc = match src.read_xmb(path) {
                    Some(d) => d,
                    None => return Ok(false),
                };
                let tac = database::hw1::tactics::parse(&doc)?;
                for owner in owners {
                    self.tactics.insert(owner, tac.clone());
                }
                Ok(true)
            }

            AssetKind::Physics(path) => {
                let owners = self.owners_of_asset(path);
                if owners.is_empty() {
                    return Ok(false);
                }
                let doc = match src.read_xmb(path) {
                    Some(d) => d,
                    None => return Ok(false),
                };
                let phys = database::hw1::physics::parse_physics(&doc)?;
                let mut chain = PhysicsChain {
                    physics: phys,
                    ..Default::default()
                };
                resolve_physics_chain(src, &mut chain, &mut self.stats);
                for owner in owners {
                    self.physics.insert(owner, chain.clone());
                }
                Ok(true)
            }

            AssetKind::Blueprint(_) | AssetKind::Shape(_) => {
                // Blueprint/shape changes affect the physics chain.
                // Find owners via the path and re-resolve the full chain.
                let owners = self.owners_of_asset(kind.game_path());
                let mut any = false;
                for owner in owners {
                    if let Some(oa) = self.assets.get(&owner)
                        && let Some(phys_path) = &oa.physics
                    {
                        let phys_path = phys_path.clone();
                        let reloaded = self.reload_asset(&AssetKind::Physics(phys_path), src)?;
                        any = any || reloaded;
                    }
                }
                Ok(any)
            }

            AssetKind::Scenario(path) => {
                // Re-read the .scn file.
                if let Some(doc) = src.read_xmb(path) {
                    let root = doc.root();
                    if let Some(root) = root {
                        self.scenario_data = Some(super::scenario::parse_scenario_data(root));
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            AssetKind::TerrainData(path) => {
                // Re-read into cache if we had it loaded.
                if self.terrain_data.is_some()
                    && let Some(data) = src.resolve_exact(path)
                {
                    self.terrain_data = xtd::Reader::read(&data).ok();
                }
                Ok(true)
            }

            AssetKind::TerrainTextures(path) => {
                if self.terrain_textures.is_some()
                    && let Some(data) = src.resolve_exact(path)
                {
                    self.terrain_textures = xtt::Reader::read(&data).ok();
                }
                Ok(true)
            }

            // Re-read cached binary assets if they're in the cache.
            AssetKind::Model(path) => {
                if self.models.contains_key(path)
                    && let Some(data) = src.resolve_exact(path)
                    && let Ok(geom) = ugx::UgxGeom::from_bytes(&data)
                {
                    self.models.insert(path.clone(), geom);
                }
                Ok(true)
            }
            AssetKind::Animation(path) => {
                if self.animations.contains_key(path)
                    && let Some(data) = src.resolve_exact(path)
                    && let Ok(anim) = uax::UaxFile::from_bytes(&data)
                {
                    self.animations.insert(path.clone(), anim);
                }
                Ok(true)
            }
            AssetKind::Texture(path) => {
                if self.textures.contains_key(path)
                    && let Some(data) = src.resolve_exact(path)
                    && let Ok(tex) = ddx::DdxTexture::from_bytes(&data)
                {
                    self.textures.insert(path.clone(), tex);
                }
                Ok(true)
            }
        }
    }
}
