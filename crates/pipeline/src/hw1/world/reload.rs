//! Incremental reload of `World` tables and single assets from the asset source.

use database::hw1::OverlayTable;

use crate::source::AssetSource;

use super::super::edit::{AssetKind, TableId};
use super::super::manifest::{collect_object_visual_assets, collect_visual_assets};
use super::super::resolve::resolve_physics_chain;

impl super::World {
    /// Re-parse a single database table from the asset source.
    ///
    /// This is the incremental-reload primitive: after an override file
    /// changes on disk, call this with the corresponding [`TableId`] to
    /// update just that table without a full world reload.
    ///
    /// Returns `Ok(true)` if the table was successfully reloaded,
    /// `Ok(false)` if the source file was not found (table left unchanged),
    /// or `Err` on parse failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the selected table cannot be parsed.
    pub fn reload_table(
        &mut self,
        table: TableId,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> crate::Result<bool> {
        match table {
            TableId::Objects => {
                if let Some(doc) = src.read_xmb(OverlayTable::Objects.base_path()) {
                    let update = src.read_xmb(OverlayTable::Objects.update_path());
                    self.database.load_objects(&doc, update.as_ref())?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Squads => {
                if let Some(doc) = src.read_xmb(OverlayTable::Squads.base_path()) {
                    let update = src.read_xmb(OverlayTable::Squads.update_path());
                    self.database.load_squads(&doc, update.as_ref())?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            TableId::Techs => {
                if let Some(doc) = src.read_xmb(OverlayTable::Techs.base_path()) {
                    let update = src.read_xmb(OverlayTable::Techs.update_path());
                    self.database.load_techs(&doc, update.as_ref())?;
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
                // Scenario reload re-reads the .scn, .sc2 and .sc3 files.
                if let Some(desc) = &self.scenario
                    && let Some(scn) = desc.read_scenario(src)
                {
                    let sc2 = desc.read_sc2(src)?;
                    let sc3 = desc.read_sc3(src)?;
                    self.scenario_data = Some(scn);
                    self.scenario_sc2 = sc2;
                    self.scenario_sc3 = sc3;
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
                    self.strings = super::super::stringtable::StringTable::load(&lang, src);
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
    /// Searches the [`ObjectAssets`](crate::hw1::ObjectAssets) map for visual, tactics, physics,
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
    ///
    /// # Errors
    ///
    /// Returns an error if a changed XML asset cannot be parsed.
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
                let Some(doc) = src.read_xmb(path) else {
                    return Ok(false);
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
                let Some(doc) = src.read_xmb(path) else {
                    return Ok(false);
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
                let Some(doc) = src.read_xmb(path) else {
                    return Ok(false);
                };
                let phys = database::hw1::physics::parse_physics(&doc)?;
                let mut chain = super::super::resolve::PhysicsChain {
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
                        self.scenario_data =
                            Some(super::super::scenario::parse_scenario_data(root));
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
