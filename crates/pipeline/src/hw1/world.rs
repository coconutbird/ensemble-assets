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
//! 3. **Edit** — Use `*_mut()` accessors to get [`DirtyGuard`](super::edit::DirtyGuard) or
//!    [`KeyDirtyGuard`](super::edit::KeyDirtyGuard) handles. Mutations
//!    through these guards automatically track what changed.
//!
//! 4. **Save** — [`World::save`] serializes only the dirty tables/files
//!    to the override directory. Per-file saves (e.g. [`save_visual`](World::save_visual))
//!    are also available for fine-grained control.

mod edit_api;
mod reload;
mod save;
mod summary;

use std::collections::{HashMap, HashSet};

use crate::source::AssetSource;

use super::edit::{DirtySet, TableId};
use super::loader;
use super::manifest::{
    AssetManifest, BinaryValidation, collect_object_visual_assets, collect_scenario_assets_into,
    collect_visual_assets, discover_terrain_textures, discover_textures, parse_preload_list,
    resolve_scenario, resolve_textures_for,
};
use super::resolve::{LoadStats, ObjectAssets, PhysicsChain, resolve_physics_chain};
use super::scenario::{ScenarioData, ScenarioDescriptor, ScenarioList};

/// Controls which optional asset inventories are built while loading a world.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldLoadOptions {
    resolve_object_assets: bool,
    discover_texture_manifest: bool,
}

impl WorldLoadOptions {
    /// Load every editable object asset and eagerly inventory referenced textures.
    #[must_use]
    pub const fn full() -> Self {
        Self {
            resolve_object_assets: true,
            discover_texture_manifest: true,
        }
    }

    /// Load runtime-critical database and scenario data without editor inventories.
    ///
    /// Object visuals can subsequently be populated on demand with
    /// [`World::load_visuals_for`]. Tactics and physics asset chains remain
    /// unloaded because the runtime simulation uses the typed game database.
    #[must_use]
    pub const fn runtime() -> Self {
        Self {
            resolve_object_assets: false,
            discover_texture_manifest: false,
        }
    }
}

impl Default for WorldLoadOptions {
    fn default() -> Self {
        Self::full()
    }
}

#[derive(Default)]
struct ObjectAssetInventory {
    assets: HashMap<String, ObjectAssets>,
    visuals: HashMap<String, database::hw1::Visual>,
    tactics: HashMap<String, database::hw1::TacticData>,
    physics: HashMap<String, PhysicsChain>,
    manifest: AssetManifest,
    stats: LoadStats,
}

impl ObjectAssetInventory {
    fn empty(objects_total: usize) -> Self {
        Self {
            stats: LoadStats {
                objects_total,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn load(
        src: &mut AssetSource<impl assets::FileProvider>,
        database: &database::hw1::Database,
    ) -> Self {
        let mut inventory = Self::empty(database.objects.len());
        for object in &database.objects {
            let mut object_assets = ObjectAssets {
                name: object.name.clone(),
                object_class: object.object_class.clone(),
                object_types: object.object_types.clone(),
                ..Default::default()
            };

            resolve_visual_asset(src, object, &mut object_assets, &mut inventory);
            resolve_tactics_asset(src, object, &mut object_assets, &mut inventory);
            resolve_physics_asset(src, object, &mut object_assets, &mut inventory);
            inventory.assets.insert(object.name.clone(), object_assets);
        }
        inventory
    }
}

fn resolve_visual_asset(
    src: &mut AssetSource<impl assets::FileProvider>,
    object: &database::hw1::ProtoObject,
    object_assets: &mut ObjectAssets,
    inventory: &mut ObjectAssetInventory,
) {
    let Some(visual_ref) = &object.visual else {
        return;
    };

    load_object_visual(
        src,
        object,
        visual_ref,
        object_assets,
        &mut inventory.visuals,
        &mut inventory.manifest,
        &mut inventory.stats,
    );
}

/// Load the visual file `visual_ref` of one object.
///
/// The function sets the visual path of `object_assets` and counts the attempt in `stats`. If
/// the visual loads, the function adds its asset paths to `object_assets` and `manifest` and
/// inserts it in `visuals`. If the visual does not load, the function records the failure in
/// `stats`.
///
/// Returns `true` if the visual loads.
fn load_object_visual(
    src: &mut AssetSource<impl assets::FileProvider>,
    object: &database::hw1::ProtoObject,
    visual_ref: &str,
    object_assets: &mut ObjectAssets,
    visuals: &mut HashMap<String, database::hw1::Visual>,
    manifest: &mut AssetManifest,
    stats: &mut LoadStats,
) -> bool {
    stats.objects_with_visual += 1;
    let visual_normalized = visual_ref.replace('/', "\\");
    let visual_normalized = visual_normalized.trim_start_matches('\\');
    let visual_base = format!("art\\{visual_normalized}");
    object_assets.visual = Some(visual_base.clone());

    let Some(visual_document) = src.read_xmb(&visual_base) else {
        stats.visuals_failed.push(visual_base);
        return false;
    };
    match database::hw1::visual::parse(&visual_document) {
        Ok(visual) => {
            stats.visuals_resolved += 1;
            collect_visual_assets(&visual, manifest);
            collect_object_visual_assets(&visual, object_assets);
            visuals.insert(object.name.clone(), visual);
            true
        }
        Err(error) => {
            stats.visuals_failed.push(format!("{visual_base}: {error}"));
            false
        }
    }
}

fn resolve_tactics_asset(
    src: &mut AssetSource<impl assets::FileProvider>,
    object: &database::hw1::ProtoObject,
    object_assets: &mut ObjectAssets,
    inventory: &mut ObjectAssetInventory,
) {
    let Some(tactics_ref) = &object.tactics else {
        return;
    };

    inventory.stats.objects_with_tactics += 1;
    let tactics_base = format!("data\\tactics\\{tactics_ref}");
    object_assets.tactics = Some(tactics_base.clone());

    let Some(tactics_document) = src.read_xmb(&tactics_base) else {
        inventory.stats.tactics_failed.push(tactics_base);
        return;
    };
    match database::hw1::tactics::parse(&tactics_document) {
        Ok(tactics) => {
            inventory.stats.tactics_resolved += 1;
            inventory.tactics.insert(object.name.clone(), tactics);
        }
        Err(error) => inventory
            .stats
            .tactics_failed
            .push(format!("{tactics_base}: {error}")),
    }
}

fn resolve_physics_asset(
    src: &mut AssetSource<impl assets::FileProvider>,
    object: &database::hw1::ProtoObject,
    object_assets: &mut ObjectAssets,
    inventory: &mut ObjectAssetInventory,
) {
    let Some(physics_ref) = &object.physics_info else {
        return;
    };

    inventory.stats.objects_with_physics += 1;
    let physics_base = format!("physics\\{physics_ref}.physics");
    object_assets.physics = Some(physics_base.clone());

    let Some(physics_document) = src.read_xmb(&physics_base) else {
        inventory.stats.physics_failed.push(physics_base);
        return;
    };
    match database::hw1::physics::parse_physics(&physics_document) {
        Ok(physics) => {
            inventory.stats.physics_resolved += 1;
            let mut chain = PhysicsChain {
                physics,
                ..Default::default()
            };
            resolve_physics_chain(src, &mut chain, &mut inventory.stats);
            if let Some(blueprint) = &chain.blueprint {
                if let Some(blueprint_ref) = chain.physics.primary_blueprint() {
                    object_assets.blueprint = Some(format!("physics\\{blueprint_ref}.blueprint"));
                }
                if let Some(shape_ref) = &blueprint.shape {
                    object_assets.shape = Some(format!("physics\\{shape_ref}.shp"));
                }
            }
            inventory.physics.insert(object.name.clone(), chain);
        }
        Err(error) => inventory
            .stats
            .physics_failed
            .push(format!("{physics_base}: {error}")),
    }
}

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
    /// The parsed game database (objects, squads, techs and other tables).
    pub database: database::hw1::Database,
    /// Per-object resolved file paths, keyed by object name.
    pub assets: HashMap<String, ObjectAssets>,
    /// Parsed visuals, keyed by object name.
    pub visuals: HashMap<String, database::hw1::Visual>,
    /// Parsed tactics, keyed by object name.
    pub tactics: HashMap<String, database::hw1::TacticData>,
    /// Physics chains (physics, blueprint and shape), keyed by object name.
    pub physics: HashMap<String, PhysicsChain>,
    /// Descriptor of the active scenario, or `None` for the base world.
    pub scenario: Option<ScenarioDescriptor>,
    /// Parsed `.scn` data (placed objects, players, terrain, etc.).
    pub scenario_data: Option<ScenarioData>,
    /// All scenarios from the scenario descriptions file.
    pub scenario_list: ScenarioList,
    /// Inventory of the binary assets that the world refers to.
    pub manifest: AssetManifest,
    /// Statistics from the world load.
    pub stats: LoadStats,
    /// Localized string table (default: English).
    pub strings: Option<super::stringtable::StringTable>,
    /// Parsed lightset data (GLS + FLS) for the active scenario.
    pub lightset: Option<super::lightset::LightSetData>,
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
    ///
    /// # Errors
    ///
    /// Returns an error if any required database asset cannot be decoded or deserialized.
    pub fn load(
        game_dir: &str,
    ) -> crate::Result<(Self, AssetSource<crate::source::StdFileProvider>)> {
        let mut src = loader::load_game_dir(game_dir);
        let mut world = Self::load_from_source(&mut src)?;
        world.base_era_count = src.era_count();
        Ok((world, src))
    }

    /// Load from a pre-configured [`AssetSource`].
    ///
    /// # Errors
    ///
    /// Returns an error if any required database asset cannot be decoded or deserialized.
    pub fn load_from_source(
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> crate::Result<Self> {
        Self::load_from_source_with_options(src, WorldLoadOptions::default())
    }

    /// Load from a pre-configured asset source using the requested load profile.
    ///
    /// [`WorldLoadOptions::runtime`] omits editor-oriented, whole-database asset
    /// inventories. This substantially reduces startup work for applications
    /// that resolve presentation assets only for active entities.
    ///
    /// # Errors
    ///
    /// Returns an error if any required database or scenario asset cannot be
    /// decoded or deserialized.
    pub fn load_from_source_with_options(
        src: &mut AssetSource<impl assets::FileProvider>,
        options: WorldLoadOptions,
    ) -> crate::Result<Self> {
        // 1. Load the full database
        let database = database::hw1::Database::load(src)?;

        // 2. Load scenario descriptors
        let scenario_list = ScenarioList::load(src);

        // 3. Resolve all per-object asset chains when the load profile needs them.
        let ObjectAssetInventory {
            assets: assets_map,
            visuals,
            tactics,
            physics,
            mut manifest,
            stats,
        } = if options.resolve_object_assets {
            ObjectAssetInventory::load(src, &database)
        } else {
            ObjectAssetInventory::empty(database.objects.len())
        };

        // 4. Parse preload lists from scenario ERAs
        parse_preload_list(src, "visFileList.txt", &mut manifest.preload_vis_refs);
        parse_preload_list(src, "pfxFileList.txt", &mut manifest.preload_pfx_refs);
        parse_preload_list(src, "tfxFileList.txt", &mut manifest.preload_tfx_refs);

        // 5. Eagerly discover texture references from UGX material chunks
        if options.discover_texture_manifest {
            discover_textures(src, &manifest.model_refs, &mut manifest.texture_refs);
            discover_textures(src, &manifest.damage_model_refs, &mut manifest.texture_refs);
        }

        // 6. Auto-detect and load scenario from the SCN file
        let (scenario, scenario_data) = resolve_scenario(src, &scenario_list, &mut manifest);

        // 7. Discover terrain texture references from XTT files
        if options.discover_texture_manifest {
            discover_terrain_textures(src, &manifest.terrain_refs, &mut manifest.texture_refs);
        }

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

        // 10. Load lightset data (GLS + FLS) if a scenario is present
        let lightset = load_lightset(scenario.as_ref(), scenario_data.as_ref(), src);

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
            lightset,
            terrain_data,
            terrain_textures,
            models: HashMap::new(),
            textures: HashMap::new(),
            animations: HashMap::new(),
            dirty: DirtySet::default(),
            base_era_count: 0,
        })
    }

    /// Resolve visual definitions for the requested proto-object names.
    ///
    /// Already-attempted objects are skipped. Missing objects, missing visual
    /// files, and invalid visual documents are recorded in [`Self::stats`],
    /// matching the behavior of a full world load.
    ///
    /// Returns the number of visual definitions newly loaded.
    pub fn load_visuals_for<I, S>(
        &mut self,
        src: &mut AssetSource<impl assets::FileProvider>,
        object_names: I,
    ) -> usize
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let requested: HashSet<String> = object_names
            .into_iter()
            .map(|name| name.as_ref().to_ascii_lowercase())
            .collect();
        let requests = self
            .database
            .objects
            .iter()
            .filter(|object| requested.contains(&object.name.to_ascii_lowercase()))
            .filter(|object| {
                self.assets
                    .get(&object.name)
                    .is_none_or(|assets| assets.visual.is_none())
            })
            .filter_map(|object| Some((object, object.visual.as_deref()?)))
            .collect::<Vec<_>>();

        let mut loaded = 0;
        for (object, visual_ref) in requests {
            let object_assets =
                self.assets
                    .entry(object.name.clone())
                    .or_insert_with(|| ObjectAssets {
                        name: object.name.clone(),
                        object_class: object.object_class.clone(),
                        object_types: object.object_types.clone(),
                        ..ObjectAssets::default()
                    });
            if load_object_visual(
                src,
                object,
                visual_ref,
                object_assets,
                &mut self.visuals,
                &mut self.manifest,
                &mut self.stats,
            ) {
                loaded += 1;
            }
        }
        loaded
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
        self.lightset = None;
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

        // 7. Load lightset data (GLS + FLS) if the new scenario has one.
        self.lightset = load_lightset(scenario_desc.as_ref(), scenario_data.as_ref(), src);

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
        let Some(obj) = self.assets.get(name) else {
            return Vec::new();
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
        let Some(obj) = self.assets.get(name) else {
            return Vec::new();
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
        let Some(obj) = self.assets.get(name) else {
            return Vec::new();
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
}

/// Load lightset data from a scenario descriptor + scene data.
///
/// Extracts the lightset reference from the SCN data, then delegates to
/// [`super::lightset::load`].
fn load_lightset(
    descriptor: Option<&ScenarioDescriptor>,
    scn: Option<&ScenarioData>,
    src: &mut AssetSource<impl assets::FileProvider>,
) -> Option<super::lightset::LightSetData> {
    let desc = descriptor?;
    let ls_ref = scn?.lightset();
    if ls_ref.is_empty() {
        return None;
    }
    super::lightset::load(desc, ls_ref, src)
}
