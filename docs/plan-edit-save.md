# Edit & Save Pipeline

Load → Edit → Save workflow for HW1 game data, building toward editing tools and a language server.

## What We Have

- **Load**: Full world from ERAs (database, visuals, tactics, physics, scenarios)
- **Round-trip**: `Database::to_documents()` serializes all tables back to XMB, verified by test
- **Write-back**: `AssetSource::write_xmb` / `write_xml` to override directory
- **bdt-serde**: Symmetric `from_node` / `to_node` for all database types + scenario data
- **Validation**: Database parse validation with `bdt_serde::Warning` diagnostics

## Design — DirtyGuard

All mutable access goes through `DirtyGuard<T>`, a `DerefMut` wrapper that
auto-marks the owning table dirty on drop. Same `World` type for engine and
tools — engine reads fields directly, tools call `_mut()` to get tracked access.

```rust
// Read — direct field access, zero cost
let hp = world.database.objects[0].hitpoints;

// Write — DirtyGuard marks table dirty automatically
world.objects_mut()[0].hitpoints = 500.0;
world.scenario_data_mut().objects_mut().push(placed_obj);
world.visuals_mut().get_mut("warthog").unwrap().model = "new.ugx";

// Save — only writes tables that were touched
world.save(&src)?;
```

```rust
pub struct DirtyGuard<'a, T> {
    data: &'a mut T,
    flag: &'a Cell<bool>,
}

impl<T> Deref for DirtyGuard<'_, T> { type Target = T; ... }
impl<T> DerefMut for DirtyGuard<'_, T> { ... }
impl<T> Drop for DirtyGuard<'_, T> {
    fn drop(&mut self) { self.flag.set(true); }
}
```

No per-field tracking. Each table serializes in milliseconds so we save the
whole table when any part is dirty.

## Phase 1 — Edit API + Selective Save

### 1.1 DirtyGuard + DirtySet

```rust
pub struct DirtySet {
    flags: [Cell<bool>; TableId::COUNT],
}

pub enum TableId {
    Objects, Squads, Techs, Abilities, Powers,
    Civs, Leaders, WeaponTypes, DamageTypes, GameData,
    Scenario, Visuals, Tactics, Physics,
}
```

### 1.2 Mutable Accessors on World

One `_mut()` method per editable collection:

```rust
impl World {
    pub fn objects_mut(&mut self) -> DirtyGuard<'_, Vec<ProtoObject>> { ... }
    pub fn squads_mut(&mut self) -> DirtyGuard<'_, Vec<Squad>> { ... }
    pub fn visuals_mut(&mut self) -> DirtyGuard<'_, HashMap<String, Visual>> { ... }
    pub fn scenario_data_mut(&mut self) -> DirtyGuard<'_, ScenarioData> { ... }
    // etc for all editable tables
}
```

### 1.3 Selective Save

```rust
impl World {
    pub fn save(&mut self, src: &AssetSource<impl FileProvider>) -> Result<Vec<PathBuf>>;
    pub fn is_dirty(&self) -> bool;
    pub fn dirty_tables(&self) -> Vec<TableId>;
}
```

- Iterates dirty tables only
- Database tables: `Database::to_document(table_id)` → `src.write_xmb()`
- Scenario: `ScenarioData::to_document()` → `src.write_xmb()`
- Visuals/tactics/physics: individual file serialization
- Returns list of written paths
- Clears dirty set on success

### 1.4 Deliverables

- [x] `DirtyGuard<T>` type (pipeline crate)
- [x] `DirtySet` + `TableId` types
- [x] `_mut()` accessors on `World` for all editable data
- [x] `Database::to_document(TableId)` — single-table serialization
- [x] `ScenarioData::to_document()` — SCN serialization
- [x] `World::save()` — selective write to override dir
- [x] Test: mutate objects + scenario, save, reload from override, verify round-trip

## Phase 2 — Structured Diagnostics

Goal: Diagnostics suitable for a language server (location + severity + message).

### 2.1 Diagnostic Type

```rust
pub struct Diagnostic {
    pub path: String,          // game path (e.g. "data\\objects.xml")
    pub location: Location,    // where in the file
    pub severity: Severity,    // Error, Warning, Info
    pub code: DiagnosticCode,  // machine-readable category
    pub message: String,       // human-readable
}

pub enum Location {
    Table(TableId),
    Entry { table: TableId, name: String },
    Field { table: TableId, name: String, field: String },
}

pub enum DiagnosticCode {
    ParseError,
    UnknownField,
    TypeMismatch,
    MissingReference,    // object refs visual that doesn't exist
    DuplicateName,
    MissingAsset,        // .ugx/.uax/.ddx not found in ERAs
}
```

### 2.2 Cross-Reference Validation

- Object → visual exists
- Object → tactics file exists
- Visual → model `.ugx` exists in ERAs
- Visual → animation `.uax` exists
- Squad → object references valid proto names
- Tech → prereq tech names exist

### 2.3 Deliverables

- [x] `Diagnostic`, `Location`, `Severity`, `DiagnosticCode` types
- [x] `validate_world()` cross-reference validation pass
- [x] Object → visual/tactics/physics checks, squad → object checks, duplicate name detection
- [x] Test: inject a bad reference, verify diagnostic produced

## Phase 3 — Incremental Watch

Goal: File-watching loop that re-validates on change (language server core + engine hot-reload).

### 3.1 File Watcher

Watch the override directory for changes. On file write:

1. Classify the changed file via `AssetKind::from_game_path`
2. Re-parse / invalidate just that asset (database table, visual, tactics, physics, model, texture, etc.)
3. Re-run cross-reference validation for affected entries
4. Emit updated diagnostics

### 3.2 Deliverables

- [x] File → table mapping (`TableId::from_game_path` / `table_for_override_path`)
- [x] Universal asset classification (`AssetKind` enum + `from_game_path`)
- [x] Reverse lookup (`World::owners_of_asset`) — maps file paths back to object names
- [x] Incremental re-parse on change (`World::reload_table` for DB, `World::reload_asset` for all)
- [x] Per-file XML reload (visuals, tactics, physics) with HashMap entry replacement
- [x] Binary asset invalidation (models, animations, textures, terrain)
- [x] Dependency-aware re-validation (`validate_world` after reload)
- [x] `notify` crate integration for filesystem watching (`WorldWatcher` in `crates/watch`)
- [x] `WorldEvent::AssetReloaded(AssetKind)` — events for all asset types
- [x] Integration tests (manual reload + filesystem watcher)

## Repo Split Point

After Phase 1, the edit/save API is stable enough to split:

- **`ensemble-formats`** — format crates (era, xmb, bdt, ugx, uax, ddx, xtd, xtt)
- **`ensemble-assets`** — pipeline + database (depends on formats)
- **`ensemble-tools`** — CLI, editor UI, language server (depends on assets)

Phases 2–3 likely belong in `ensemble-tools` since they're consumer-facing.
