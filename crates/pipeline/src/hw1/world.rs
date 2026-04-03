//! HW1 world — database, resolved assets, scenario, and asset manifest.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::source::AssetSource;

use super::edit::{DirtyGuard, DirtySet, TableId};
use super::loader;
use super::manifest::{
    AssetManifest, BinaryValidation, collect_object_visual_assets, collect_scenario_assets_into,
    collect_visual_assets, discover_terrain_textures, discover_textures, parse_preload_list,
    resolve_scenario, resolve_textures_for,
};
use super::resolve::{LoadStats, ObjectAssets, PhysicsChain, resolve_physics_chain};
use super::scenario::{ScenarioData, ScenarioDescriptor, ScenarioList};

/// A fully loaded HW1 game world: database, resolved assets, scenario, and manifest.
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
    dirty: DirtySet,
}

impl World {
    /// Load a complete HW1 world from a game directory.
    ///
    /// `scenario` accepts an ERA filename (`"PHXscn01.era"`), a map name
    /// (`"PHXscn01"`), or an SCN path.
    pub fn load(game_dir: &str, scenario: Option<&str>) -> crate::Result<Self> {
        let mut src = loader::load_game_dir(game_dir);

        // Resolve and load the scenario ERA if requested.
        if let Some(scen) = scenario {
            if let Some(era_name) = loader::find_scenario_era(game_dir, scen) {
                loader::load_scenario_era(&mut src, game_dir, &era_name);
            } else {
                eprintln!("  WARN  could not find scenario ERA for '{scen}'");
            }
        }

        Self::load_from_source(&mut src)
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
            dirty: DirtySet::default(),
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

    /// Whether any table has been modified since load (or last save).
    pub fn is_dirty(&self) -> bool {
        self.dirty.is_any_dirty()
    }

    /// Which tables have been modified.
    pub fn dirty_tables(&self) -> Vec<TableId> {
        self.dirty.dirty_tables()
    }

    pub fn objects_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::ProtoObject>> {
        DirtyGuard::new(
            &mut self.database.objects,
            self.dirty.flag(TableId::Objects),
        )
    }

    pub fn squads_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Squad>> {
        DirtyGuard::new(&mut self.database.squads, self.dirty.flag(TableId::Squads))
    }

    pub fn techs_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Tech>> {
        DirtyGuard::new(&mut self.database.techs, self.dirty.flag(TableId::Techs))
    }

    pub fn abilities_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Ability>> {
        DirtyGuard::new(
            &mut self.database.abilities,
            self.dirty.flag(TableId::Abilities),
        )
    }

    pub fn powers_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Power>> {
        DirtyGuard::new(&mut self.database.powers, self.dirty.flag(TableId::Powers))
    }

    pub fn civs_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Civ>> {
        DirtyGuard::new(&mut self.database.civs, self.dirty.flag(TableId::Civs))
    }

    pub fn leaders_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::Leader>> {
        DirtyGuard::new(
            &mut self.database.leaders,
            self.dirty.flag(TableId::Leaders),
        )
    }

    pub fn weapon_types_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::WeaponType>> {
        DirtyGuard::new(
            &mut self.database.weapon_types,
            self.dirty.flag(TableId::WeaponTypes),
        )
    }

    pub fn damage_types_mut(&mut self) -> DirtyGuard<'_, Vec<database::hw1::DamageType>> {
        DirtyGuard::new(
            &mut self.database.damage_types,
            self.dirty.flag(TableId::DamageTypes),
        )
    }

    pub fn game_data_mut(&mut self) -> DirtyGuard<'_, Option<database::hw1::GameData>> {
        DirtyGuard::new(
            &mut self.database.game_data,
            self.dirty.flag(TableId::GameData),
        )
    }

    pub fn scenario_data_mut(&mut self) -> DirtyGuard<'_, Option<ScenarioData>> {
        DirtyGuard::new(&mut self.scenario_data, self.dirty.flag(TableId::Scenario))
    }

    pub fn visuals_mut(&mut self) -> DirtyGuard<'_, HashMap<String, database::hw1::Visual>> {
        DirtyGuard::new(&mut self.visuals, self.dirty.flag(TableId::Visuals))
    }

    pub fn tactics_mut(&mut self) -> DirtyGuard<'_, HashMap<String, database::hw1::TacticData>> {
        DirtyGuard::new(&mut self.tactics, self.dirty.flag(TableId::Tactics))
    }

    pub fn physics_mut(&mut self) -> DirtyGuard<'_, HashMap<String, PhysicsChain>> {
        DirtyGuard::new(&mut self.physics, self.dirty.flag(TableId::Physics))
    }

    /// Save all dirty tables to the override directory.
    ///
    /// Returns the list of files written. Clears dirty flags on success.
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
                TableId::Visuals | TableId::Tactics | TableId::Physics => {
                    // TODO: per-file serialization for visuals/tactics/physics
                }
            }
        }

        self.dirty.clear();
        Ok(written)
    }
}
