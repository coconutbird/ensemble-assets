//! Dirty tracking for the edit/save workflow.
//!
//! The edit system uses RAII guards ([`DirtyGuard`], [`KeyDirtyGuard`])
//! that automatically mark data as modified when dropped. This lets
//! [`World::save`](super::World::save) write only the tables that
//! actually changed, rather than re-serializing everything.
//!
//! # How it works
//!
//! 1. Call a `*_mut()` accessor on [`World`](super::World) to get a guard.
//! 2. Mutate the data through the guard's `DerefMut` impl.
//! 3. When the guard drops, the corresponding dirty flag is set.
//! 4. [`World::save`](super::World::save) inspects the dirty flags and
//!    only serializes the changed tables/files.
//!
//! # Table-level vs key-level tracking
//!
//! - **Table-level** ([`DirtyGuard`]): Used for database tables like
//!   `objects`, `squads`, `techs`. The entire table is re-serialized.
//!   Obtained via `world.objects_mut()`, `world.squads_mut()`, etc.
//!
//! - **Key-level** ([`KeyDirtyGuard`]): Used for per-file assets like
//!   visuals, tactics, physics, models, textures. Only the specific
//!   file for that key is re-serialized.
//!   Obtained via `world.visual_mut("name")`, `world.model_mut("path")`, etc.
//!
//! # Example
//!
//! ```no_run
//! # use pipeline::hw1::World;
//! # let mut world: World = todo!();
//! // Table-level: modify the objects table.
//! {
//!     let mut objects = world.objects_mut();
//!     for obj in objects.iter_mut() {
//!         if obj.hitpoints.unwrap_or(0.0) < 100.0 {
//!             obj.hitpoints = Some(100.0);
//!         }
//!     }
//! } // ← DirtyGuard drops, Objects table marked dirty.
//!
//! // Key-level: modify a single visual file.
//! if let Some(mut vis) = world.visual_mut("unsc_inf_marine_01") {
//!     vis.default_model = Some("Default".to_string());
//! } // ← KeyDirtyGuard drops, only "unsc_inf_marine_01" visual marked dirty.
//!
//! assert!(world.is_dirty());
//! ```

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

/// Identifies a data table in a [`World`](super::World).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TableId {
    /// Proto-objects in `data\objects.xml`.
    Objects,
    /// Proto-squads in `data\squads.xml`.
    Squads,
    /// Techs in `data\techs.xml`.
    Techs,
    /// Abilities in `data\abilities.xml`.
    Abilities,
    /// Powers in `data\powers.xml`.
    Powers,
    /// Civs in `data\civs.xml`.
    Civs,
    /// Leaders in `data\leaders.xml`.
    Leaders,
    /// Weapon types in `data\weapontypes.xml`.
    WeaponTypes,
    /// Damage types in `data\damagetypes.xml`.
    DamageTypes,
    /// Global game settings in `data\gamedata.xml`.
    GameData,
    /// The active scenario file (`scenario\...\*.scn`).
    Scenario,
    /// Per-object visual files (`.vis`).
    Visuals,
    /// Per-object tactics files.
    Tactics,
    /// Per-object physics chains (physics, blueprint and shape files).
    Physics,
    /// Terrain data (`.xtd`) of the active scenario.
    TerrainData,
    /// Terrain textures (`.xtt`) of the active scenario.
    TerrainTextures,
    /// The localized string table.
    Strings,
    /// Model files (`.ugx`).
    Models,
    /// Animation files (`.uax`).
    Animations,
    /// Texture files (`.ddx`).
    Textures,
}

impl TableId {
    /// Number of `TableId` variants.
    pub const COUNT: usize = 20;

    /// All `TableId` variants, in declaration order.
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
        Self::TerrainData,
        Self::TerrainTextures,
        Self::Strings,
        Self::Models,
        Self::Animations,
        Self::Textures,
    ];

    fn index(self) -> usize {
        self as u8 as usize
    }

    /// Map a normalised game path (e.g. `"data\\objects.xml"`) to a `TableId`.
    ///
    /// An `_update` file (e.g. `"data\\objects_update.xml"`) maps to the table that it changes.
    /// Returns `None` for paths that don't correspond to a database table.
    #[must_use]
    pub fn from_game_path(path: &str) -> Option<Self> {
        // Normalise: lowercase, forward-slash → backslash, strip `.xmb` suffix.
        let p = path.to_ascii_lowercase().replace('/', "\\");
        let p = p.strip_suffix(".xmb").unwrap_or(&p);
        match p {
            "data\\objects.xml" | "data\\objects_update.xml" => Some(Self::Objects),
            "data\\squads.xml" | "data\\squads_update.xml" => Some(Self::Squads),
            "data\\techs.xml" | "data\\techs_update.xml" => Some(Self::Techs),
            "data\\abilities.xml" => Some(Self::Abilities),
            "data\\powers.xml" => Some(Self::Powers),
            "data\\civs.xml" => Some(Self::Civs),
            "data\\leaders.xml" => Some(Self::Leaders),
            "data\\weapontypes.xml" => Some(Self::WeaponTypes),
            "data\\damagetypes.xml" => Some(Self::DamageTypes),
            "data\\gamedata.xml" => Some(Self::GameData),
            _ if p.starts_with("scenario\\") && crate::source::has_extension(p, "scn") => {
                Some(Self::Scenario)
            }
            _ => None,
        }
    }

    /// The well-known game path for this table (reverse of [`Self::from_game_path`]).
    #[must_use]
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
            Self::Scenario
            | Self::Visuals
            | Self::Tactics
            | Self::Physics
            | Self::TerrainData
            | Self::TerrainTextures
            | Self::Strings
            | Self::Models
            | Self::Animations
            | Self::Textures => None,
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
    #[must_use]
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
        if crate::source::has_extension(p, "vis") {
            return Some(Self::Visual(p.to_string()));
        }
        // Tactics files live under data\tactics\*.xml (not .tactics).
        if p.starts_with("data\\tactics\\") && crate::source::has_extension(p, "xml") {
            return Some(Self::Tactics(p.to_string()));
        }
        if crate::source::has_extension(p, "physics") {
            return Some(Self::Physics(p.to_string()));
        }
        if crate::source::has_extension(p, "blueprint") {
            return Some(Self::Blueprint(p.to_string()));
        }
        if crate::source::has_extension(p, "shp") {
            return Some(Self::Shape(p.to_string()));
        }

        // 3. Binary assets (by extension).
        if crate::source::has_extension(p, "ugx") {
            return Some(Self::Model(p.to_string()));
        }
        if crate::source::has_extension(p, "uax") {
            return Some(Self::Animation(p.to_string()));
        }
        if crate::source::has_extension(p, "ddx") {
            return Some(Self::Texture(p.to_string()));
        }

        // 4. Terrain files.
        if crate::source::has_extension(p, "xtd") {
            return Some(Self::TerrainData(p.to_string()));
        }
        if crate::source::has_extension(p, "xtt") {
            return Some(Self::TerrainTextures(p.to_string()));
        }

        None
    }

    /// The game path carried by this asset kind (if any).
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
///
/// For per-file tables (Visuals, Tactics, Physics), also tracks which
/// specific keys (object names) are dirty so that [`super::World::save`] can
/// write only the changed files instead of the entire table.
pub struct DirtySet {
    flags: [Cell<bool>; TableId::COUNT],
    /// Dirty keys for per-file tables.
    dirty_visuals: RefCell<HashSet<String>>,
    dirty_tactics: RefCell<HashSet<String>>,
    dirty_physics: RefCell<HashSet<String>>,
    dirty_models: RefCell<HashSet<String>>,
    dirty_animations: RefCell<HashSet<String>>,
    dirty_textures: RefCell<HashSet<String>>,
}

impl Default for DirtySet {
    fn default() -> Self {
        Self {
            flags: std::array::from_fn(|_| Cell::new(false)),
            dirty_visuals: RefCell::new(HashSet::new()),
            dirty_tactics: RefCell::new(HashSet::new()),
            dirty_physics: RefCell::new(HashSet::new()),
            dirty_models: RefCell::new(HashSet::new()),
            dirty_animations: RefCell::new(HashSet::new()),
            dirty_textures: RefCell::new(HashSet::new()),
        }
    }
}

impl DirtySet {
    /// Return `true` if `table` has unsaved changes.
    pub fn is_dirty(&self, table: TableId) -> bool {
        self.flags[table.index()].get()
    }

    /// Return `true` if any table has unsaved changes.
    pub fn is_any_dirty(&self) -> bool {
        self.flags.iter().any(Cell::get)
    }

    /// Return the tables that have unsaved changes, in `TableId::ALL` order.
    pub fn dirty_tables(&self) -> Vec<TableId> {
        TableId::ALL
            .iter()
            .copied()
            .filter(|&t| self.is_dirty(t))
            .collect()
    }

    /// Return the set of dirty keys for a per-file table.
    ///
    /// Returns an empty set for non-per-file tables. When the table-level
    /// flag is set but the key set is empty, it means the whole table was
    /// dirtied (via `visuals_mut()` etc.) and all entries should be saved.
    pub fn dirty_keys(&self, table: TableId) -> HashSet<String> {
        match table {
            TableId::Visuals => self.dirty_visuals.borrow().clone(),
            TableId::Tactics => self.dirty_tactics.borrow().clone(),
            TableId::Physics => self.dirty_physics.borrow().clone(),
            TableId::Models => self.dirty_models.borrow().clone(),
            TableId::Animations => self.dirty_animations.borrow().clone(),
            TableId::Textures => self.dirty_textures.borrow().clone(),
            _ => HashSet::new(),
        }
    }

    /// Mark a specific key dirty within a per-file table.
    ///
    /// Also sets the table-level dirty flag.
    pub fn mark_key(&self, table: TableId, key: String) {
        self.flags[table.index()].set(true);
        match table {
            TableId::Visuals => {
                self.dirty_visuals.borrow_mut().insert(key);
            }
            TableId::Tactics => {
                self.dirty_tactics.borrow_mut().insert(key);
            }
            TableId::Physics => {
                self.dirty_physics.borrow_mut().insert(key);
            }
            TableId::Models => {
                self.dirty_models.borrow_mut().insert(key);
            }
            TableId::Animations => {
                self.dirty_animations.borrow_mut().insert(key);
            }
            TableId::Textures => {
                self.dirty_textures.borrow_mut().insert(key);
            }
            _ => {}
        }
    }

    /// Clear dirty state for a single table (flag + any per-file keys).
    pub fn clear_table(&self, table: TableId) {
        self.flags[table.index()].set(false);
        match table {
            TableId::Visuals => self.dirty_visuals.borrow_mut().clear(),
            TableId::Tactics => self.dirty_tactics.borrow_mut().clear(),
            TableId::Physics => self.dirty_physics.borrow_mut().clear(),
            TableId::Models => self.dirty_models.borrow_mut().clear(),
            TableId::Animations => self.dirty_animations.borrow_mut().clear(),
            TableId::Textures => self.dirty_textures.borrow_mut().clear(),
            _ => {}
        }
    }

    /// Clear the dirty state of all tables, with all per-file keys.
    pub fn clear(&self) {
        for flag in &self.flags {
            flag.set(false);
        }
        self.dirty_visuals.borrow_mut().clear();
        self.dirty_tactics.borrow_mut().clear();
        self.dirty_physics.borrow_mut().clear();
        self.dirty_models.borrow_mut().clear();
        self.dirty_animations.borrow_mut().clear();
        self.dirty_textures.borrow_mut().clear();
    }

    /// Clear dirty state for a single per-file key.
    pub fn clear_key(&self, table: TableId, key: &str) {
        let set = match table {
            TableId::Visuals => &self.dirty_visuals,
            TableId::Tactics => &self.dirty_tactics,
            TableId::Physics => &self.dirty_physics,
            TableId::Models => &self.dirty_models,
            TableId::Animations => &self.dirty_animations,
            TableId::Textures => &self.dirty_textures,
            _ => return,
        };
        let mut s = set.borrow_mut();
        s.remove(key);
        // If no more dirty keys, clear the table-level flag too.
        if s.is_empty() {
            self.flags[table.index()].set(false);
        }
    }

    pub(crate) fn flag(&self, table: TableId) -> &Cell<bool> {
        &self.flags[table.index()]
    }
}

/// RAII guard that marks a whole table dirty when dropped.
///
/// Returned by `World::objects_mut()`, `World::squads_mut()`, etc.
/// Dereferences to `&mut T` so you can mutate the inner data naturally
/// using standard `Vec`/`HashMap` methods. The dirty flag is set
/// unconditionally on drop — even if you didn't change anything.
///
/// ```no_run
/// # use pipeline::hw1::World;
/// # let mut world: World = todo!();
/// let mut techs = world.techs_mut();
/// techs.retain(|t| !t.name.is_empty()); // standard Vec method
/// // dirty flag set when `techs` goes out of scope
/// ```
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

/// RAII guard that marks a specific key dirty within a per-file table.
///
/// Returned by `World::visual_mut("name")`, `World::tactic_mut("name")`,
/// `World::model_mut("path")`, etc. Dereferences to `&mut T` so you
/// can mutate the data naturally. On drop, marks both the table-level
/// flag **and** records the specific key so that
/// [`World::save`](super::World::save) only re-serializes that one file.
///
/// ```no_run
/// # use pipeline::hw1::World;
/// # let mut world: World = todo!();
/// // Only the "unsc_inf_marine_01" tactics file will be saved.
/// if let Some(mut tac) = world.tactic_mut("unsc_inf_marine_01") {
///     tac.weapons.clear();
/// }
/// ```
pub struct KeyDirtyGuard<'a, T> {
    data: &'a mut T,
    dirty: &'a DirtySet,
    table: TableId,
    key: String,
}

impl<'a, T> KeyDirtyGuard<'a, T> {
    pub(crate) fn new(data: &'a mut T, dirty: &'a DirtySet, table: TableId, key: String) -> Self {
        Self {
            data,
            dirty,
            table,
            key,
        }
    }
}

impl<T> Deref for KeyDirtyGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> DerefMut for KeyDirtyGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T> Drop for KeyDirtyGuard<'_, T> {
    fn drop(&mut self) {
        self.dirty.mark_key(self.table, self.key.clone());
    }
}
