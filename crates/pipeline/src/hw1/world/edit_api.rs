//! Dirty-tracking edit accessors and lazy binary asset loads of a `World`.

use std::collections::HashMap;

use crate::source::AssetSource;

use super::super::edit::{DirtyGuard, KeyDirtyGuard, TableId};
use super::super::resolve::PhysicsChain;
use super::super::scenario::ScenarioData;

impl super::World {
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
    pub fn strings_mut(
        &mut self,
    ) -> DirtyGuard<'_, Option<super::super::stringtable::StringTable>> {
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
}
