//! HW1 game database parser.
//!
//! Provides typed access to game data stored in XMB files:
//! - `objects.xml.xmb` — proto objects (units, buildings, projectiles)
//! - `squads.xml.xmb` — squad definitions
//! - `techs.xml.xmb` — tech tree
//! - `abilities.xml.xmb` — ability definitions
//! - `powers.xml.xmb` — leader power definitions
//! - `civs.xml.xmb` — civilization definitions
//! - `leaders.xml.xmb` — leader definitions
//! - `weapontypes.xml.xmb` — weapon type damage tables
//! - `damagetypes.xml.xmb` — damage type definitions
//! - `gamedata.xml.xmb` — global game constants
//! - `*.vis.xmb` — visual definitions (models, animations, attachments)
//! - `*.tactics.xmb` — combat tactics (weapons, actions)
//! - `*.physics.xmb` / `*.blueprint.xmb` / `*.shp.xmb` — physics data

pub mod abilities;
pub mod civs;
pub mod damagetypes;
pub mod gamedata;
pub mod leaders;
pub mod objects;
pub mod physics;
pub mod powers;
pub mod squads;
pub mod tactics;
pub mod techs;
pub mod visual;
pub mod weapontypes;

use alloc::string::String;
use alloc::vec::Vec;

use assets::AssetResolver;

/// Loader function: parses an XMB document into the database.
type DbLoader = fn(&mut Database, &xmb::Document) -> crate::Result<()>;

pub use abilities::Ability;
pub use civs::Civ;
pub use damagetypes::DamageType;
pub use gamedata::GameData;
pub use leaders::Leader;
pub use objects::ProtoObject;
pub use physics::{Blueprint, Physics, Shape};
pub use powers::Power;
pub use squads::Squad;
pub use tactics::TacticData;
pub use techs::Tech;
pub use visual::Visual;
pub use weapontypes::WeaponType;

/// A complete HW1 game database, loaded from XMB documents.
#[derive(Debug, Default)]
pub struct Database {
    pub objects: Vec<ProtoObject>,
    pub squads: Vec<Squad>,
    pub techs: Vec<Tech>,
    pub abilities: Vec<Ability>,
    pub powers: Vec<Power>,
    pub civs: Vec<Civ>,
    pub leaders: Vec<Leader>,
    pub weapon_types: Vec<WeaponType>,
    pub damage_types: Vec<DamageType>,
    pub game_data: Option<GameData>,
}

impl Database {
    /// Create an empty database.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the full HW1 database from an [`AssetResolver`].
    ///
    /// Resolves each database XMB by its well-known virtual path, parses it,
    /// and populates the corresponding field.  Missing files are silently
    /// skipped (the field stays at its `Default` value).
    pub fn load(assets: &mut impl AssetResolver) -> crate::Result<Self> {
        let mut db = Self::new();

        // Each entry: (virtual path, loader method)
        let loaders: &[(&str, DbLoader)] = &[
            ("data\\objects.xml.xmb", |db, doc| {
                db.load_objects(doc)?;
                Ok(())
            }),
            ("data\\squads.xml.xmb", |db, doc| {
                db.load_squads(doc)?;
                Ok(())
            }),
            ("data\\techs.xml.xmb", |db, doc| {
                db.load_techs(doc)?;
                Ok(())
            }),
            ("data\\abilities.xml.xmb", |db, doc| {
                db.load_abilities(doc)?;
                Ok(())
            }),
            ("data\\powers.xml.xmb", |db, doc| {
                db.load_powers(doc)?;
                Ok(())
            }),
            ("data\\civs.xml.xmb", |db, doc| {
                db.load_civs(doc)?;
                Ok(())
            }),
            ("data\\leaders.xml.xmb", |db, doc| {
                db.load_leaders(doc)?;
                Ok(())
            }),
            ("data\\weapontypes.xml.xmb", |db, doc| {
                db.load_weapon_types(doc)?;
                Ok(())
            }),
            ("data\\damagetypes.xml.xmb", |db, doc| {
                db.load_damage_types(doc)?;
                Ok(())
            }),
            ("data\\gamedata.xml.xmb", |db, doc| {
                db.load_game_data(doc)?;
                Ok(())
            }),
        ];

        for &(path, loader) in loaders {
            if let Some(raw) = assets.resolve(path) {
                let doc = xmb::Reader::read(&raw)?;
                loader(&mut db, &doc)?;
            }
        }

        Ok(db)
    }

    /// Load proto objects from an `objects.xml.xmb` document.
    pub fn load_objects(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let objs = objects::parse(doc)?;
        let count = objs.len();
        self.objects = objs;
        Ok(count)
    }

    /// Load squads from a `squads.xml.xmb` document.
    pub fn load_squads(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let squads = squads::parse(doc)?;
        let count = squads.len();
        self.squads = squads;
        Ok(count)
    }

    /// Load techs from a `techs.xml.xmb` document.
    pub fn load_techs(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let techs = techs::parse(doc)?;
        let count = techs.len();
        self.techs = techs;
        Ok(count)
    }

    /// Load abilities from an `abilities.xml.xmb` document.
    pub fn load_abilities(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let abs = abilities::parse(doc)?;
        let count = abs.len();
        self.abilities = abs;
        Ok(count)
    }

    /// Load powers from a `powers.xml.xmb` document.
    pub fn load_powers(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let pows = powers::parse(doc)?;
        let count = pows.len();
        self.powers = pows;
        Ok(count)
    }

    /// Load civilizations from a `civs.xml.xmb` document.
    pub fn load_civs(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let civs = civs::parse(doc)?;
        let count = civs.len();
        self.civs = civs;
        Ok(count)
    }

    /// Load leaders from a `leaders.xml.xmb` document.
    pub fn load_leaders(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let leaders = leaders::parse(doc)?;
        let count = leaders.len();
        self.leaders = leaders;
        Ok(count)
    }

    /// Load weapon types from a `weapontypes.xml.xmb` document.
    pub fn load_weapon_types(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let wts = weapontypes::parse(doc)?;
        let count = wts.len();
        self.weapon_types = wts;
        Ok(count)
    }

    /// Load damage types from a `damagetypes.xml.xmb` document.
    pub fn load_damage_types(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let dts = damagetypes::parse(doc)?;
        let count = dts.len();
        self.damage_types = dts;
        Ok(count)
    }

    /// Load game data from a `gamedata.xml.xmb` document.
    pub fn load_game_data(&mut self, doc: &xmb::Document) -> crate::Result<()> {
        self.game_data = Some(gamedata::parse(doc)?);
        Ok(())
    }

    // ── Serialisation helpers ──────────────────────────────────────────

    /// Rebuild all database XMB documents from the typed structs.
    ///
    /// Returns a list of `(game_path, Document)` pairs, one for each
    /// database file that has data.  The game paths use the same
    /// backslash-separated convention as the ERA entries
    /// (e.g. `"data\\objects.xml"`).
    pub fn to_documents(&self) -> crate::Result<Vec<(String, xmb::Document)>> {
        let mut docs = Vec::new();

        if !self.objects.is_empty() {
            docs.push((
                String::from("data\\objects.xml"),
                Self::collection_to_doc("Objects", "Object", &self.objects)?,
            ));
        }
        if !self.squads.is_empty() {
            docs.push((
                String::from("data\\squads.xml"),
                Self::collection_to_doc("Squads", "Squad", &self.squads)?,
            ));
        }
        if !self.techs.is_empty() {
            docs.push((
                String::from("data\\techs.xml"),
                Self::collection_to_doc("TechTree", "Tech", &self.techs)?,
            ));
        }
        if !self.abilities.is_empty() {
            docs.push((
                String::from("data\\abilities.xml"),
                Self::collection_to_doc("Abilities", "Ability", &self.abilities)?,
            ));
        }
        if !self.powers.is_empty() {
            docs.push((
                String::from("data\\powers.xml"),
                Self::collection_to_doc("Powers", "Power", &self.powers)?,
            ));
        }
        if !self.civs.is_empty() {
            docs.push((
                String::from("data\\civs.xml"),
                Self::collection_to_doc("Civs", "Civ", &self.civs)?,
            ));
        }
        if !self.leaders.is_empty() {
            docs.push((
                String::from("data\\leaders.xml"),
                Self::collection_to_doc("Leaders", "Leader", &self.leaders)?,
            ));
        }
        if !self.weapon_types.is_empty() {
            docs.push((
                String::from("data\\weapontypes.xml"),
                Self::collection_to_doc("WeaponTypes", "WeaponType", &self.weapon_types)?,
            ));
        }
        if !self.damage_types.is_empty() {
            docs.push((
                String::from("data\\damagetypes.xml"),
                Self::collection_to_doc("DamageTypes", "DamageType", &self.damage_types)?,
            ));
        }
        if let Some(ref gd) = self.game_data {
            let node = bdt_serde::to_node("GameData", gd)?;
            docs.push((
                String::from("data\\gamedata.xml"),
                xmb::Document::with_root(node),
            ));
        }

        Ok(docs)
    }

    /// Serialize a single collection into an XMB document.
    ///
    /// Builds `<root_name><child_name>...</child_name>...</root_name>`.
    fn collection_to_doc<T: serde::Serialize>(
        root_name: &str,
        child_name: &str,
        items: &[T],
    ) -> crate::Result<xmb::Document> {
        let mut root = bdt::Node::new(root_name);
        for item in items {
            root.add_child(bdt_serde::to_node(child_name, item)?);
        }
        Ok(xmb::Document::with_root(root))
    }
}
