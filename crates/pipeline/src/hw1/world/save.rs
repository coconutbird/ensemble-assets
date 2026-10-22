//! Save of a `World` to the override directory: per-file saves and the dirty-table save.

use std::path::PathBuf;

use crate::source::AssetSource;

use super::super::edit::TableId;
use super::super::resolve::PhysicsChain;

impl super::World {
    // ---- Per-file save ----

    /// Save a single visual file to the override directory.
    ///
    /// Writes the `.vis` for the named object and clears its dirty key.
    /// Returns the path written, or an error if the object/path is unknown.
    ///
    /// # Errors
    ///
    /// Returns an error if the visual or its path is unknown, serialization fails, or writing fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the tactics data or path is unknown, serialization fails, or writing fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the chain or paths are unknown, serialization fails, or writing fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if terrain data or its path is unavailable, serialization fails, or writing fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if terrain textures or their path are unavailable, serialization fails, or writing fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if no string table is loaded, serialization fails, or writing fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the model is not cached, serialization fails, or writing fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the texture is not cached, serialization fails, or writing fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the animation is not cached or writing fails.
    pub fn save_animation(
        &mut self,
        game_path: &str,
        src: &AssetSource<impl assets::FileProvider>,
    ) -> Result<PathBuf, String> {
        let anim = self
            .animations
            .get(game_path)
            .ok_or_else(|| format!("animation not cached: {game_path}"))?;
        let bytes = anim
            .to_bytes()
            .map_err(|e| format!("serialize animation {game_path}: {e}"))?;
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

        for &table in &dirty {
            match table {
                TableId::Objects
                | TableId::Squads
                | TableId::Techs
                | TableId::Abilities
                | TableId::Powers
                | TableId::Civs
                | TableId::Leaders
                | TableId::WeaponTypes
                | TableId::DamageTypes
                | TableId::GameData
                | TableId::Scenario => self.save_database_table(src, table, &mut written)?,
                TableId::Visuals | TableId::Tactics | TableId::Physics => {
                    self.save_object_table(src, table, &mut written)?;
                }
                TableId::TerrainData
                | TableId::TerrainTextures
                | TableId::Strings
                | TableId::Models
                | TableId::Textures
                | TableId::Animations => self.save_binary_table(src, table, &mut written)?,
            }
        }

        self.dirty.clear();
        Ok(written)
    }

    /// Write one dirty database or scenario table.
    fn save_database_table(
        &self,
        src: &AssetSource<impl assets::FileProvider>,
        table: TableId,
        written: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
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
                    .to_document_single("WeaponTypes", "WeaponType", &self.database.weapon_types)
                    .map_err(|e| format!("serialize weapon_types: {e}"))?;
                written.push(src.write_xmb("data\\weapontypes.xml", &doc)?);
            }
            TableId::DamageTypes => {
                let doc = self
                    .database
                    .to_document_single("DamageTypes", "DamageType", &self.database.damage_types)
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
            _ => {}
        }
        Ok(())
    }

    /// Write the dirty per-object files (visuals, tactics or physics chains) of one table.
    fn save_object_table(
        &self,
        src: &AssetSource<impl assets::FileProvider>,
        table: TableId,
        written: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        match table {
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
                let iter: Box<dyn Iterator<Item = (&String, &PhysicsChain)>> = if keys.is_empty() {
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
            _ => {}
        }
        Ok(())
    }

    /// Write one dirty terrain, string or binary asset table.
    fn save_binary_table(
        &self,
        src: &AssetSource<impl assets::FileProvider>,
        table: TableId,
        written: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        match table {
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
                let iter: Box<dyn Iterator<Item = (&String, &ugx::UgxGeom)>> = if keys.is_empty() {
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
                let iter: Box<dyn Iterator<Item = (&String, &ddx::DdxTexture)>> = if keys.is_empty()
                {
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
                let iter: Box<dyn Iterator<Item = (&String, &uax::UaxFile)>> = if keys.is_empty() {
                    Box::new(self.animations.iter())
                } else {
                    Box::new(
                        self.animations
                            .iter()
                            .filter(|(k, _)| keys.contains(k.as_str())),
                    )
                };
                for (game_path, anim) in iter {
                    let bytes = anim
                        .to_bytes()
                        .map_err(|e| format!("serialize animation {game_path}: {e}"))?;
                    written.push(src.write_file(game_path, &bytes)?);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
