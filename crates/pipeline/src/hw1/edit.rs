//! Dirty tracking for the edit/save workflow.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

/// Identifies a data table in a [`World`](super::World).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TableId {
    Objects,
    Squads,
    Techs,
    Abilities,
    Powers,
    Civs,
    Leaders,
    WeaponTypes,
    DamageTypes,
    GameData,
    Scenario,
    Visuals,
    Tactics,
    Physics,
}

impl TableId {
    pub const COUNT: usize = 14;

    pub const ALL: [TableId; Self::COUNT] = [
        Self::Objects,
        Self::Squads,
        Self::Techs,
        Self::Abilities,
        Self::Powers,
        Self::Civs,
        Self::Leaders,
        Self::WeaponTypes,
        Self::DamageTypes,
        Self::GameData,
        Self::Scenario,
        Self::Visuals,
        Self::Tactics,
        Self::Physics,
    ];

    fn index(self) -> usize {
        self as u8 as usize
    }

    /// Map a normalised game path (e.g. `"data\\objects.xml"`) to a `TableId`.
    ///
    /// Returns `None` for paths that don't correspond to a database table.
    pub fn from_game_path(path: &str) -> Option<Self> {
        // Normalise: lowercase, forward-slash → backslash, strip `.xmb` suffix.
        let p = path.to_ascii_lowercase().replace('/', "\\");
        let p = p.strip_suffix(".xmb").unwrap_or(&p);
        match p {
            "data\\objects.xml" => Some(Self::Objects),
            "data\\squads.xml" => Some(Self::Squads),
            "data\\techs.xml" => Some(Self::Techs),
            "data\\abilities.xml" => Some(Self::Abilities),
            "data\\powers.xml" => Some(Self::Powers),
            "data\\civs.xml" => Some(Self::Civs),
            "data\\leaders.xml" => Some(Self::Leaders),
            "data\\weapontypes.xml" => Some(Self::WeaponTypes),
            "data\\damagetypes.xml" => Some(Self::DamageTypes),
            "data\\gamedata.xml" => Some(Self::GameData),
            _ if p.starts_with("scenario\\") && p.ends_with(".scn") => Some(Self::Scenario),
            _ => None,
        }
    }

    /// The well-known game path for this table (reverse of [`from_game_path`]).
    pub fn game_path(self) -> Option<&'static str> {
        match self {
            Self::Objects => Some("data\\objects.xml"),
            Self::Squads => Some("data\\squads.xml"),
            Self::Techs => Some("data\\techs.xml"),
            Self::Abilities => Some("data\\abilities.xml"),
            Self::Powers => Some("data\\powers.xml"),
            Self::Civs => Some("data\\civs.xml"),
            Self::Leaders => Some("data\\leaders.xml"),
            Self::WeaponTypes => Some("data\\weapontypes.xml"),
            Self::DamageTypes => Some("data\\damagetypes.xml"),
            Self::GameData => Some("data\\gamedata.xml"),
            Self::Scenario | Self::Visuals | Self::Tactics | Self::Physics => None,
        }
    }
}

/// Classifies any game-path into a typed asset category.
///
/// Where [`TableId`] only covers the 10 database XML tables (plus
/// aggregate Visuals/Tactics/Physics), `AssetKind` handles **every**
/// file type the pipeline knows about — including per-object XML files
/// (`.vis`, `.tactics`, `.physics`) and binary assets (`.ugx`, `.uax`,
/// `.ddx`, `.xtd`, `.xtt`).
///
/// The contained `String` is the normalised **game path** (backslash-
/// separated, e.g. `"art\\unsc_inf_marine_01.vis"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetKind {
    /// A top-level database XML table (objects.xml, squads.xml, …).
    DatabaseTable(TableId),
    /// A per-object visual definition (`.vis` / `.vis.xmb`).
    Visual(String),
    /// A per-object tactics definition (`data\tactics\*.xml`).
    Tactics(String),
    /// A physics definition (`.physics` / `.physics.xmb`).
    Physics(String),
    /// A physics blueprint (`.blueprint` / `.blueprint.xmb`).
    Blueprint(String),
    /// A physics shape (`.shp` / `.shp.xmb`).
    Shape(String),
    /// A binary model (`.ugx`).
    Model(String),
    /// A binary animation (`.uax`).
    Animation(String),
    /// A binary texture (`.ddx`).
    Texture(String),
    /// Scenario scene data (`.scn`).
    Scenario(String),
    /// Terrain heightmap / lighting (`.xtd`).
    TerrainData(String),
    /// Terrain textures / foliage (`.xtt`).
    TerrainTextures(String),
}

impl AssetKind {
    /// Classify a normalised game path into an [`AssetKind`].
    ///
    /// Accepts paths with forward or back slashes, with or without an
    /// `.xmb` suffix.  Returns `None` for unrecognised patterns.
    pub fn from_game_path(path: &str) -> Option<Self> {
        let p = path.to_ascii_lowercase().replace('/', "\\");
        let p = p.strip_suffix(".xmb").unwrap_or(&p);

        // 1. Try database table first.
        if let Some(tid) = TableId::from_game_path(p) {
            // TableId::from_game_path already handles scenario .scn,
            // but we want the richer Scenario(path) variant here.
            if tid == TableId::Scenario {
                return Some(Self::Scenario(p.to_string()));
            }
            return Some(Self::DatabaseTable(tid));
        }

        // 2. Per-file XML assets (by extension or path convention).
        if p.ends_with(".vis") {
            return Some(Self::Visual(p.to_string()));
        }
        // Tactics files live under data\tactics\*.xml (not .tactics).
        if p.starts_with("data\\tactics\\") && p.ends_with(".xml") {
            return Some(Self::Tactics(p.to_string()));
        }
        if p.ends_with(".physics") {
            return Some(Self::Physics(p.to_string()));
        }
        if p.ends_with(".blueprint") {
            return Some(Self::Blueprint(p.to_string()));
        }
        if p.ends_with(".shp") {
            return Some(Self::Shape(p.to_string()));
        }

        // 3. Binary assets (by extension).
        if p.ends_with(".ugx") {
            return Some(Self::Model(p.to_string()));
        }
        if p.ends_with(".uax") {
            return Some(Self::Animation(p.to_string()));
        }
        if p.ends_with(".ddx") {
            return Some(Self::Texture(p.to_string()));
        }

        // 4. Terrain files.
        if p.ends_with(".xtd") {
            return Some(Self::TerrainData(p.to_string()));
        }
        if p.ends_with(".xtt") {
            return Some(Self::TerrainTextures(p.to_string()));
        }

        None
    }

    /// The game path carried by this asset kind (if any).
    pub fn game_path(&self) -> &str {
        match self {
            Self::DatabaseTable(t) => t.game_path().unwrap_or(""),
            Self::Visual(p)
            | Self::Tactics(p)
            | Self::Physics(p)
            | Self::Blueprint(p)
            | Self::Shape(p)
            | Self::Model(p)
            | Self::Animation(p)
            | Self::Texture(p)
            | Self::Scenario(p)
            | Self::TerrainData(p)
            | Self::TerrainTextures(p) => p,
        }
    }

    /// Whether this is a parseable XML/XMB asset (vs. a binary blob).
    pub fn is_xml(&self) -> bool {
        matches!(
            self,
            Self::DatabaseTable(_)
                | Self::Visual(_)
                | Self::Tactics(_)
                | Self::Physics(_)
                | Self::Blueprint(_)
                | Self::Shape(_)
                | Self::Scenario(_)
        )
    }

    /// Whether this is a binary asset that the engine loads lazily.
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            Self::Model(_)
                | Self::Animation(_)
                | Self::Texture(_)
                | Self::TerrainData(_)
                | Self::TerrainTextures(_)
        )
    }
}

/// Tracks which tables have been modified.
pub struct DirtySet {
    flags: [Cell<bool>; TableId::COUNT],
}

impl Default for DirtySet {
    fn default() -> Self {
        Self {
            flags: std::array::from_fn(|_| Cell::new(false)),
        }
    }
}

impl DirtySet {
    pub fn is_dirty(&self, table: TableId) -> bool {
        self.flags[table.index()].get()
    }

    pub fn is_any_dirty(&self) -> bool {
        self.flags.iter().any(Cell::get)
    }

    pub fn dirty_tables(&self) -> Vec<TableId> {
        TableId::ALL
            .iter()
            .copied()
            .filter(|&t| self.is_dirty(t))
            .collect()
    }

    pub fn clear(&self) {
        for flag in &self.flags {
            flag.set(false);
        }
    }

    pub(crate) fn flag(&self, table: TableId) -> &Cell<bool> {
        &self.flags[table.index()]
    }
}

/// RAII guard that marks a table dirty when dropped.
///
/// Returned by `World::*_mut()` accessors. Dereferences to `&mut T`
/// so callers can mutate the data naturally. The dirty flag is set on
/// drop, not on each write — this avoids overhead and keeps the guard
/// cheap.
pub struct DirtyGuard<'a, T> {
    data: &'a mut T,
    flag: &'a Cell<bool>,
}

impl<'a, T> DirtyGuard<'a, T> {
    pub(crate) fn new(data: &'a mut T, flag: &'a Cell<bool>) -> Self {
        Self { data, flag }
    }
}

impl<T> Deref for DirtyGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> DerefMut for DirtyGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T> Drop for DirtyGuard<'_, T> {
    fn drop(&mut self) {
        self.flag.set(true);
    }
}
