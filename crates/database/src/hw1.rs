//! HW1 game database parser.
//!
//! Provides typed access to game data stored in XMB files:
//! - `objects.xml.xmb` + `objects_update.xml.xmb` — proto objects (units, buildings, projectiles)
//! - `objecttypes.xml.xmb` — declared object types
//! - `squads.xml.xmb` + `squads_update.xml.xmb` — squad definitions
//! - `techs.xml.xmb` + `techs_update.xml.xmb` — tech tree
//! - `abilities.xml.xmb` — ability definitions
//! - `powers.xml.xmb` — leader power definitions
//! - `civs.xml.xmb` — civilization definitions
//! - `leaders.xml.xmb` — leader definitions
//! - `weapontypes.xml.xmb` — weapon type damage tables
//! - `damagetypes.xml.xmb` — damage type definitions
//! - `gamedata.xml.xmb` — global game constants
//! - `gamemodes.xml.xmb` — skirmish game modes
//! - `aidifficultysettings.xml.xmb` — named AI settings by difficulty
//! - `hpbars.xml.xmb` — HP bar and other unit UI definitions
//! - `playercolors.xml.xmb` — player color tables
//! - `*.vis.xmb` — visual definitions (models, animations, attachments)
//! - `*.tactics.xmb` — combat tactics (weapons, actions)
//! - `*.physics.xmb` / `*.blueprint.xmb` / `*.shp.xmb` — physics data
//! - `*.pfx.xmb` — particle effect definitions

pub mod abilities;
pub mod aidifficultysettings;
pub mod civs;
pub mod damagetypes;
pub mod gamedata;
pub mod gamemodes;
pub mod hpbars;
pub mod leaders;
pub mod objects;
pub mod objecttypes;
pub mod overlay;
pub mod pfx;
pub mod physics;
pub mod playercolors;
pub mod powers;
pub mod squads;
pub mod tactics;
pub mod techs;
pub mod types;
pub mod visual;
pub mod weapontypes;

use alloc::string::String;
use alloc::vec::Vec;

use assets::AssetResolver;

/// Loader function: parses an XMB document into the database.
type DbLoader = fn(&mut Database, &xmb::Document) -> crate::Result<()>;

pub use abilities::Ability;
pub use aidifficultysettings::AIDifficultySetting;
pub use civs::Civ;
pub use damagetypes::DamageType;
pub use gamedata::GameData;
pub use gamemodes::GameMode;
pub use hpbars::HpBarDefinition;
pub use leaders::Leader;
pub use objects::ProtoObject;
pub use objecttypes::{ObjectType, ObjectTypeIndex};
pub use overlay::OverlayTable;
pub use physics::{Blueprint, Physics, Shape};
pub use playercolors::PlayerColors;
pub use powers::Power;
pub use squads::Squad;
pub use tactics::TacticData;
pub use techs::Tech;
pub use types::{RgbColor, Vector2, Vector3};
pub use visual::Visual;
pub use weapontypes::WeaponType;

/// A table entry with a stable identity key within its table.
///
/// HW1 spells the identity field differently across tables in XML — an `@name`
/// attribute (objects, squads, techs, powers), a capitalised `@Name`
/// (abilities, leaders), a `Name` child element (civs, weapon types), or the
/// element's text (damage types) — but every typed entry stores it uniformly as
/// `name`. This trait exposes that key so tooling (diffing, merging) can
/// identify entries without reconstructing the serialized form and guessing
/// which spelling a given table used.
pub trait Keyed {
    /// The entry's identity (its unique name) within its table.
    fn key(&self) -> &str;
}

macro_rules! impl_keyed {
    ($($ty:ty),+ $(,)?) => {
        $(impl Keyed for $ty {
            fn key(&self) -> &str {
                &self.name
            }
        })+
    };
}

impl_keyed!(
    ProtoObject,
    Squad,
    Tech,
    Ability,
    Power,
    Civ,
    Leader,
    WeaponType,
    DamageType,
    ObjectType,
    GameMode,
    AIDifficultySetting,
);

/// A complete HW1 game database, loaded from XMB documents.
///
/// [`Database::load`] gives the tables that the engine uses. A table index is the engine ID of
/// the record.
#[derive(Debug, Default)]
pub struct Database {
    /// Proto objects: `objects.xml` without its `update="true"` records, then
    /// `objects_update.xml`.
    pub objects: Vec<ProtoObject>,
    /// Declared object types from `objecttypes.xml`.
    pub object_types: Vec<ObjectType>,
    /// Squads: `squads.xml` without its `update="true"` records, then `squads_update.xml`.
    pub squads: Vec<Squad>,
    /// `<MergedSquads>` groups from `squads.xml.xmb`, in document order.
    pub merged_squads: Vec<squads::MergedSquads>,
    /// Techs: `techs.xml`, then `techs_update.xml`.
    pub techs: Vec<Tech>,
    /// Abilities from `abilities.xml.xmb`.
    pub abilities: Vec<Ability>,
    /// Leader powers from `powers.xml.xmb`.
    pub powers: Vec<Power>,
    /// Civilizations from `civs.xml.xmb`.
    pub civs: Vec<Civ>,
    /// Leaders from `leaders.xml.xmb`.
    pub leaders: Vec<Leader>,
    /// Weapon types from `weapontypes.xml.xmb`.
    pub weapon_types: Vec<WeaponType>,
    /// Damage types from `damagetypes.xml.xmb`.
    pub damage_types: Vec<DamageType>,
    /// Global constants from `gamedata.xml.xmb`.
    pub game_data: Option<GameData>,
    /// Game modes from `gamemodes.xml.xmb`.
    pub game_modes: Vec<GameMode>,
    /// AI difficulty settings from `aidifficultysettings.xml.xmb`.
    pub ai_difficulty_settings: Vec<AIDifficultySetting>,
    /// HP bar and other unit UI definitions from `hpbars.xml.xmb`.
    pub hp_bars: Option<HpBarDefinition>,
    /// Player color tables from `playercolors.xml.xmb`.
    pub player_colors: Option<PlayerColors>,
}

impl Database {
    /// Create an empty database.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the HW1 database as the engine loads it at boot (`BDatabase_setup`).
    ///
    /// Applies the `_update` overlays to the objects, squads and techs (see [`overlay`]). Each
    /// table index is the engine ID of the record. A missing file gives an empty table. A
    /// missing `_update` file adds no records. If the base file of an overlay table is missing,
    /// the table stays empty.
    ///
    /// # Errors
    ///
    /// Returns an error if a resolved asset is not valid XMB, if its contents cannot be
    /// deserialized, or if two objects have the same name.
    pub fn load(assets: &mut impl AssetResolver) -> crate::Result<Self> {
        let mut db = Self::new();
        for table in [
            OverlayTable::Objects,
            OverlayTable::Squads,
            OverlayTable::Techs,
        ] {
            let Some(base) = read_xmb(assets, table.base_path())? else {
                continue;
            };
            let update = read_xmb(assets, table.update_path())?;
            match table {
                OverlayTable::Objects => db.load_objects(&base, update.as_ref())?,
                OverlayTable::Squads => db.load_squads(&base, update.as_ref())?,
                OverlayTable::Techs => db.load_techs(&base, update.as_ref())?,
            };
        }
        db.load_single_file_tables(assets)?;
        Ok(db)
    }

    /// Load each table from its base file only, without the `_update` overlays.
    ///
    /// The objects and squads keep their `update="true"` records in file order. The result is
    /// not the database of the engine. Use it only to compare or write the base files, for
    /// example in a mod merge.
    ///
    /// # Errors
    ///
    /// Returns an error if a resolved asset is not valid XMB or its contents cannot be
    /// deserialized.
    pub fn load_base_files(assets: &mut impl AssetResolver) -> crate::Result<Self> {
        let mut db = Self::new();
        if let Some(doc) = read_xmb(assets, OverlayTable::Objects.base_path())? {
            db.objects = objects::parse(&doc)?;
        }
        if let Some(doc) = read_xmb(assets, OverlayTable::Squads.base_path())? {
            db.squads = squads::parse(&doc)?;
            db.merged_squads = squads::parse_merged(&doc)?;
        }
        if let Some(doc) = read_xmb(assets, OverlayTable::Techs.base_path())? {
            db.techs = techs::parse(&doc)?;
        }
        db.load_single_file_tables(assets)?;
        Ok(db)
    }

    /// Load each table that comes from one file.
    fn load_single_file_tables(&mut self, assets: &mut impl AssetResolver) -> crate::Result<()> {
        // Each entry: (game path, loader method)
        let loaders: &[(&str, DbLoader)] = &[
            ("data\\objecttypes.xml", |db, doc| {
                db.load_object_types(doc)?;
                Ok(())
            }),
            ("data\\abilities.xml", |db, doc| {
                db.load_abilities(doc)?;
                Ok(())
            }),
            ("data\\powers.xml", |db, doc| {
                db.load_powers(doc)?;
                Ok(())
            }),
            ("data\\civs.xml", |db, doc| {
                db.load_civs(doc)?;
                Ok(())
            }),
            ("data\\leaders.xml", |db, doc| {
                db.load_leaders(doc)?;
                Ok(())
            }),
            ("data\\weapontypes.xml", |db, doc| {
                db.load_weapon_types(doc)?;
                Ok(())
            }),
            ("data\\damagetypes.xml", |db, doc| {
                db.load_damage_types(doc)?;
                Ok(())
            }),
            ("data\\gamedata.xml", |db, doc| {
                db.load_game_data(doc)?;
                Ok(())
            }),
            ("data\\gamemodes.xml", |db, doc| {
                db.load_game_modes(doc)?;
                Ok(())
            }),
            ("data\\aidifficultysettings.xml", |db, doc| {
                db.load_ai_difficulty_settings(doc)?;
                Ok(())
            }),
            ("data\\hpbars.xml", |db, doc| {
                db.load_hp_bars(doc)?;
                Ok(())
            }),
            ("data\\playercolors.xml", |db, doc| {
                db.load_player_colors(doc)?;
                Ok(())
            }),
        ];

        for &(path, loader) in loaders {
            if let Some(doc) = read_xmb(assets, path)? {
                loader(self, &doc)?;
            }
        }
        Ok(())
    }

    /// Load proto objects from `objects.xml.xmb` and the optional `objects_update.xml.xmb`.
    ///
    /// Applies [`overlay::objects`]. Returns the number of objects.
    ///
    /// # Errors
    ///
    /// Returns an error if a document root or object is invalid, or if two objects have the
    /// same name.
    pub fn load_objects(
        &mut self,
        base: &xmb::Document,
        update: Option<&xmb::Document>,
    ) -> crate::Result<usize> {
        let update = update.map(objects::parse).transpose()?.unwrap_or_default();
        self.objects = overlay::objects(objects::parse(base)?, update)?;
        Ok(self.objects.len())
    }

    /// Load squads from `squads.xml.xmb` and the optional `squads_update.xml.xmb`.
    ///
    /// Applies [`overlay::squads`] and reads the `<MergedSquads>` groups of the base file
    /// (`BDatabase_setupProtoSquads` pass 2 reads `squads.xml` only). Returns the number of
    /// squads.
    ///
    /// # Errors
    ///
    /// Returns an error if a document root or squad is invalid.
    pub fn load_squads(
        &mut self,
        base: &xmb::Document,
        update: Option<&xmb::Document>,
    ) -> crate::Result<usize> {
        let update = update.map(squads::parse).transpose()?.unwrap_or_default();
        self.squads = overlay::squads(squads::parse(base)?, update);
        self.merged_squads = squads::parse_merged(base)?;
        Ok(self.squads.len())
    }

    /// Load techs from `techs.xml.xmb` and the optional `techs_update.xml.xmb`.
    ///
    /// Applies [`overlay::techs`]. Returns the number of techs.
    ///
    /// # Errors
    ///
    /// Returns an error if a document root or tech is invalid.
    pub fn load_techs(
        &mut self,
        base: &xmb::Document,
        update: Option<&xmb::Document>,
    ) -> crate::Result<usize> {
        let update = update.map(techs::parse).transpose()?.unwrap_or_default();
        self.techs = overlay::techs(techs::parse(base)?, update);
        Ok(self.techs.len())
    }

    /// Load object types from an `objecttypes.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or an object type is invalid.
    pub fn load_object_types(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        self.object_types = objecttypes::parse(doc)?;
        Ok(self.object_types.len())
    }

    /// Load game modes from a `gamemodes.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or a game mode is invalid.
    pub fn load_game_modes(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        self.game_modes = gamemodes::parse(doc)?;
        Ok(self.game_modes.len())
    }

    /// Load the AI difficulty settings from an `aidifficultysettings.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or a setting is invalid.
    pub fn load_ai_difficulty_settings(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        self.ai_difficulty_settings = aidifficultysettings::parse(doc)?;
        Ok(self.ai_difficulty_settings.len())
    }

    /// Load the HP bar definitions from an `hpbars.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or a definition is invalid.
    pub fn load_hp_bars(&mut self, doc: &xmb::Document) -> crate::Result<()> {
        self.hp_bars = Some(hpbars::parse(doc)?);
        Ok(())
    }

    /// Load the player color tables from a `playercolors.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document has no root or a color is invalid.
    pub fn load_player_colors(&mut self, doc: &xmb::Document) -> crate::Result<()> {
        self.player_colors = Some(playercolors::parse(doc)?);
        Ok(())
    }

    /// Build the engine name map of the objects and the object types.
    ///
    /// Use it to find the declared object types of an object.
    #[must_use]
    pub fn object_type_index(&self) -> ObjectTypeIndex {
        ObjectTypeIndex::new(&self.objects, &self.object_types)
    }

    /// Load abilities from an `abilities.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or ability contents are invalid.
    pub fn load_abilities(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let abs = abilities::parse(doc)?;
        let count = abs.len();
        self.abilities = abs;
        Ok(count)
    }

    /// Load powers from a `powers.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or power contents are invalid.
    pub fn load_powers(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let pows = powers::parse(doc)?;
        let count = pows.len();
        self.powers = pows;
        Ok(count)
    }

    /// Load civilizations from a `civs.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or civilization contents are invalid.
    pub fn load_civs(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let civs = civs::parse(doc)?;
        let count = civs.len();
        self.civs = civs;
        Ok(count)
    }

    /// Load leaders from a `leaders.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or leader contents are invalid.
    pub fn load_leaders(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let leaders = leaders::parse(doc)?;
        let count = leaders.len();
        self.leaders = leaders;
        Ok(count)
    }

    /// Load weapon types from a `weapontypes.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or weapon-type contents are invalid.
    pub fn load_weapon_types(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let wts = weapontypes::parse(doc)?;
        let count = wts.len();
        self.weapon_types = wts;
        Ok(count)
    }

    /// Load damage types from a `damagetypes.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or damage-type contents are invalid.
    pub fn load_damage_types(&mut self, doc: &xmb::Document) -> crate::Result<usize> {
        let dts = damagetypes::parse(doc)?;
        let count = dts.len();
        self.damage_types = dts;
        Ok(count)
    }

    /// Load game data from a `gamedata.xml.xmb` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document root or game-data contents are invalid.
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
    ///
    /// The overlay tables give two documents each (see [`Database::overlay_documents`]). The
    /// engine loads the same tables from the result.
    ///
    /// # Errors
    ///
    /// Returns an error if any populated database record cannot be serialized into an XMB node.
    pub fn to_documents(&self) -> crate::Result<Vec<(String, xmb::Document)>> {
        let mut docs = Vec::new();

        for (table, empty) in [
            (OverlayTable::Objects, self.objects.is_empty()),
            (OverlayTable::Squads, self.squads.is_empty()),
            (OverlayTable::Techs, self.techs.is_empty()),
        ] {
            if !empty {
                docs.extend(self.overlay_documents(table)?);
            }
        }
        if !self.object_types.is_empty() {
            docs.push((
                String::from("data\\objecttypes.xml"),
                self.to_document_single("ObjectTypes", "ObjectType", &self.object_types)?,
            ));
        }
        if !self.abilities.is_empty() {
            docs.push((
                String::from("data\\abilities.xml"),
                self.to_document_single("Abilities", "Ability", &self.abilities)?,
            ));
        }
        if !self.powers.is_empty() {
            docs.push((
                String::from("data\\powers.xml"),
                self.to_document_single("Powers", "Power", &self.powers)?,
            ));
        }
        if !self.civs.is_empty() {
            docs.push((
                String::from("data\\civs.xml"),
                self.to_document_single("Civs", "Civ", &self.civs)?,
            ));
        }
        if !self.leaders.is_empty() {
            docs.push((
                String::from("data\\leaders.xml"),
                self.to_document_single("Leaders", "Leader", &self.leaders)?,
            ));
        }
        if !self.weapon_types.is_empty() {
            docs.push((
                String::from("data\\weapontypes.xml"),
                self.to_document_single("WeaponTypes", "WeaponType", &self.weapon_types)?,
            ));
        }
        if !self.damage_types.is_empty() {
            docs.push((
                String::from("data\\damagetypes.xml"),
                self.to_document_single("DamageTypes", "DamageType", &self.damage_types)?,
            ));
        }
        if let Some(gd) = &self.game_data {
            let node = bdt_serde::to_node("GameData", gd)?;
            docs.push((
                String::from("data\\gamedata.xml"),
                xmb::Document::with_root(node),
            ));
        }
        if !self.game_modes.is_empty() {
            docs.push((
                String::from("data\\gamemodes.xml"),
                self.to_document_single("GameModes", "GameMode", &self.game_modes)?,
            ));
        }
        if !self.ai_difficulty_settings.is_empty() {
            docs.push((
                String::from("data\\aidifficultysettings.xml"),
                self.to_document_single(
                    "AIDifficultySettings",
                    "AIDifficultySetting",
                    &self.ai_difficulty_settings,
                )?,
            ));
        }
        if let Some(hp_bars) = &self.hp_bars {
            docs.push((
                String::from("data\\hpbars.xml"),
                xmb::Document::with_root(bdt_serde::to_node("HPBarDefinition", hp_bars)?),
            ));
        }
        if let Some(colors) = &self.player_colors {
            docs.push((
                String::from("data\\playercolors.xml"),
                xmb::Document::with_root(bdt_serde::to_node("playerColors", colors)?),
            ));
        }

        Ok(docs)
    }

    /// Build the base file and an empty `_update` file of an overlay table.
    ///
    /// The base file holds all records of the table in order, without `update` flags. The
    /// empty `_update` file replaces the shipped `_update` file. The engine then loads the
    /// table with the same records at the same indices. Use this to save a table of a database
    /// from [`Database::load`].
    ///
    /// # Errors
    ///
    /// Returns an error if a record cannot be serialized into an XMB node.
    pub fn overlay_documents(
        &self,
        table: OverlayTable,
    ) -> crate::Result<[(String, xmb::Document); 2]> {
        let (root, record) = (table.root_name(), table.record_name());
        let base = match table {
            OverlayTable::Objects => self.to_document_single(
                root,
                record,
                &overlay::clear_update_flags(&self.objects, |object| &mut object.update),
            )?,
            OverlayTable::Squads => self.to_document_single(
                root,
                record,
                &overlay::clear_update_flags(&self.squads, |squad| &mut squad.update),
            )?,
            OverlayTable::Techs => self.to_document_single(root, record, &self.techs)?,
        };
        let update = xmb::Document::with_root(bdt::Node::new(root));
        Ok([
            (String::from(table.base_path()), base),
            (String::from(table.update_path()), update),
        ])
    }

    /// Serialize a single collection into an XMB document.
    ///
    /// Builds `<root_name><child_name>...</child_name>...</root_name>`.
    ///
    /// # Errors
    ///
    /// Returns an error if an item cannot be serialized into an XMB node.
    pub fn to_document_single<T: serde::Serialize>(
        &self,
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

/// Resolve the packed `.xmb` form of the game path `path` and read the document.
///
/// Returns `None` if the asset does not exist.
fn read_xmb(assets: &mut impl AssetResolver, path: &str) -> crate::Result<Option<xmb::Document>> {
    let mut packed = String::from(path);
    packed.push_str(".xmb");
    assets
        .resolve(&packed)
        .map(|raw| xmb::Reader::read(&raw))
        .transpose()
        .map_err(Into::into)
}
