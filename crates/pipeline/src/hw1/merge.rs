//! Semantic merge of several HW1 mods into one, expressed as a flatten of a
//! layered [`AssetSource`](crate::source::AssetSource).
//!
//! A mod's `ModData` folder is just an unpacked ERA, so merging is "stack the
//! base game plus each mod as layers, resolve, and write the result back out":
//!
//! 1. Load the base game database (the vanilla baseline).
//! 2. Layer each mod's `ModData` as a folder above the base ERAs.
//! 3. For the typed database tables, field-merge each layer's version against
//!    the base so non-overlapping edits coexist (only true overlaps conflict,
//!    later mod wins) — the one schema-aware step.
//! 4. Flatten everything else (loose/binary files) straight out, last layer
//!    winning, optionally writing only what differs from vanilla.
//!
//! The merged tables are written as packed `.xml.xmb`; the result directory is
//! a ready-to-load `ModData`. Packaging it (e.g. a `.hwmod` manifest) is the
//! caller's concern.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use database::hw1::{Database, Keyed};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::loader::load_game_dir;
use crate::source::{AssetSource, LoadRule, StdFileProvider, normalise_path};

/// A mod to layer into the merge, in load order (later wins on conflict).
pub struct ModOverlay {
    /// Display label (mod title or path) used in conflict reports.
    pub label: String,
    /// The mod's `ModData` directory.
    pub mod_data: PathBuf,
}

/// A field-level conflict: two mods changed the same entry field differently,
/// or two mods shipped the same loose file.
pub struct Conflict {
    pub table: String,
    pub entry: String,
    pub field: String,
    pub previous_mod: String,
    pub winning_mod: String,
}

/// Options controlling how the merged `ModData` is written.
pub struct MergeOptions {
    /// Only write loose files that differ from the base game ("vanilla"), so the
    /// output contains just the delta. On by default.
    pub only_changed: bool,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self { only_changed: true }
    }
}

/// Summary of a merge run.
pub struct MergeReport {
    pub files_written: Vec<PathBuf>,
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug)]
pub enum MergeError {
    /// No mods were provided to merge.
    NoMods,
    /// Loading or (de)serializing game data failed (database / xmb).
    Data(String),
    /// JSON value conversion during the field merge failed.
    Json(serde_json::Error),
    /// A filesystem error (the message includes the path).
    Io(String),
}

impl std::fmt::Display for MergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoMods => write!(f, "no mods to merge"),
            Self::Data(e) => write!(f, "game data error: {e}"),
            Self::Json(e) => write!(f, "merge conversion error: {e}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for MergeError {}

impl From<serde_json::Error> for MergeError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Merge `overlays` (in load order) against the base game at `game_dir`, writing
/// the merged `ModData` into `out_mod_data`.
pub fn merge_mods(
    game_dir: &str,
    overlays: &[ModOverlay],
    out_mod_data: &Path,
    options: &MergeOptions,
) -> Result<MergeReport, MergeError> {
    if overlays.is_empty() {
        return Err(MergeError::NoMods);
    }

    // 1. Load the base game database — the vanilla baseline. Done before any
    //    mod folder is layered on, so it reflects only the base ERAs.
    let mut src = load_game_dir(game_dir);
    let base = Database::load(&mut src).map_err(|e| MergeError::Data(e.to_string()))?;

    // 2. Layer each mod's ModData as a folder above the base ERAs.
    for m in overlays {
        src.add_folder(&m.mod_data.to_string_lossy(), LoadRule::Replace)
            .map_err(MergeError::Io)?;
    }

    // 3. Field-merge each typed table that at least one mod overrides.
    let mut merged = Database::new();
    let mut conflicts = Vec::new();
    let mut handled: Vec<&str> = Vec::new();

    macro_rules! merge_vec {
        ($field:ident, $vpath:literal, $parse:path) => {{
            handled.push($vpath);
            let layers = parse_table_layers(&src, $vpath, $parse)?;
            if !layers.is_empty() {
                let (out, c) = merge_table(stringify!($field), &base.$field, &layers)?;
                merged.$field = out;
                conflicts.extend(c);
            }
        }};
    }

    merge_vec!(objects, "data\\objects.xml", database::hw1::objects::parse);
    merge_vec!(squads, "data\\squads.xml", database::hw1::squads::parse);
    merge_vec!(techs, "data\\techs.xml", database::hw1::techs::parse);
    merge_vec!(abilities, "data\\abilities.xml", database::hw1::abilities::parse);
    merge_vec!(powers, "data\\powers.xml", database::hw1::powers::parse);
    merge_vec!(civs, "data\\civs.xml", database::hw1::civs::parse);
    merge_vec!(leaders, "data\\leaders.xml", database::hw1::leaders::parse);
    merge_vec!(weapon_types, "data\\weapontypes.xml", database::hw1::weapontypes::parse);
    merge_vec!(damage_types, "data\\damagetypes.xml", database::hw1::damagetypes::parse);

    // gamedata is a singleton, not a keyed table.
    {
        handled.push("data\\gamedata.xml");
        let layers = parse_singleton_layers(&src, "data\\gamedata.xml", database::hw1::gamedata::parse)?;
        if !layers.is_empty() {
            let (out, c) = merge_singleton("gamedata", base.game_data.as_ref(), layers)?;
            merged.game_data = out;
            conflicts.extend(c);
        }
    }

    // 4. Write the merged tables into the output ModData as packed XMB.
    let docs = merged
        .to_documents()
        .map_err(|e| MergeError::Data(e.to_string()))?;
    let mut files_written = Vec::new();
    for (vpath, doc) in &docs {
        let bytes = xmb::Writer::write_native(doc).map_err(|e| MergeError::Data(e.to_string()))?;
        // to_documents yields e.g. "data\objects.xml"; the engine reads packed.
        let rel = format!("{}.xmb", vpath.replace('\\', "/"));
        let fs_path = out_mod_data.join(&rel);
        if let Some(parent) = fs_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MergeError::Io(format!("{}: {e}", parent.display())))?;
        }
        std::fs::write(&fs_path, &bytes).map_err(|e| MergeError::Io(format!("{}: {e}", fs_path.display())))?;
        files_written.push(fs_path);
    }

    // 5. Flatten every non-table file from the mod folders into the output,
    //    last layer winning, skipping the merged tables (both unpacked and
    //    packed forms) so a table file is never also copied verbatim.
    let mut skip = HashSet::new();
    for vpath in &handled {
        let norm = normalise_path(vpath);
        skip.insert(format!("{norm}.xmb"));
        skip.insert(norm);
    }
    let flat = src
        .flatten_folders_to_dir(out_mod_data, &skip, options.only_changed)
        .map_err(MergeError::Io)?;
    files_written.extend(flat.files_written);
    for o in flat.overwrites {
        conflicts.push(Conflict {
            table: "files".to_string(),
            entry: o.game_path,
            field: "<file>".to_string(),
            previous_mod: o.previous,
            winning_mod: o.winner,
        });
    }

    Ok(MergeReport {
        files_written,
        conflicts,
    })
}

/// Parse one keyed table from each mod layer that overrides it, in load order.
fn parse_table_layers<T>(
    src: &AssetSource<StdFileProvider>,
    vpath: &str,
    parse: fn(&xmb::Document) -> database::Result<Vec<T>>,
) -> Result<Vec<(String, Vec<T>)>, MergeError> {
    let mut out = Vec::new();
    for (label, bytes) in src.read_each_folder_data(vpath) {
        let doc = xmb::Reader::read(&bytes).map_err(|e| MergeError::Data(e.to_string()))?;
        let table = parse(&doc).map_err(|e| MergeError::Data(e.to_string()))?;
        out.push((label, table));
    }
    Ok(out)
}

/// Parse one singleton document from each mod layer that overrides it.
fn parse_singleton_layers<T>(
    src: &AssetSource<StdFileProvider>,
    vpath: &str,
    parse: fn(&xmb::Document) -> database::Result<T>,
) -> Result<Vec<(String, T)>, MergeError> {
    let mut out = Vec::new();
    for (label, bytes) in src.read_each_folder_data(vpath) {
        let doc = xmb::Reader::read(&bytes).map_err(|e| MergeError::Data(e.to_string()))?;
        let value = parse(&doc).map_err(|e| MergeError::Data(e.to_string()))?;
        out.push((label, value));
    }
    Ok(out)
}

/// Three-way merge a keyed table: start from `base`, apply each mod in order.
/// A field a mod changed from base is applied; if a prior mod already changed
/// that field to a different value, it's a conflict (last mod wins).
fn merge_table<T: Serialize + DeserializeOwned + Keyed>(
    table: &str,
    base: &[T],
    mod_tables: &[(String, Vec<T>)],
) -> Result<(Vec<T>, Vec<Conflict>), MergeError> {
    // Index base by key, preserving order.
    let mut order: Vec<String> = Vec::new();
    let mut base_map: BTreeMap<String, Value> = BTreeMap::new();
    for item in base {
        let v = serde_json::to_value(item)?;
        let key = item.key().to_string();
        order.push(key.clone());
        base_map.insert(key, v);
    }

    let mut merged = base_map.clone();
    let mut added: Vec<String> = Vec::new();
    // (entry key, field) -> mod that last changed it.
    let mut field_owner: HashMap<(String, String), String> = HashMap::new();
    let mut conflicts = Vec::new();

    for (label, items) in mod_tables {
        for item in items {
            let mod_v = serde_json::to_value(item)?;
            let key = item.key().to_string();

            match (base_map.get(&key), merged.get_mut(&key)) {
                // Existing entry: field-level merge against base.
                (Some(Value::Object(base_obj)), Some(Value::Object(merged_obj))) => {
                    if let Value::Object(mod_obj) = &mod_v {
                        for (field, mod_field) in mod_obj {
                            if base_obj.get(field) == Some(mod_field) {
                                continue; // mod didn't change this field
                            }
                            let owner = (key.clone(), field.clone());
                            if let Some(prev) = field_owner.get(&owner)
                                && merged_obj.get(field) != Some(mod_field)
                            {
                                conflicts.push(Conflict {
                                    table: table.to_string(),
                                    entry: key.clone(),
                                    field: field.clone(),
                                    previous_mod: prev.clone(),
                                    winning_mod: label.clone(),
                                });
                            }
                            merged_obj.insert(field.clone(), mod_field.clone());
                            field_owner.insert(owner, label.clone());
                        }
                    }
                }
                // New entry (not in base): take it, flagging cross-mod divergence.
                (None, existing) => {
                    if let Some(existing) = existing
                        && *existing != mod_v
                    {
                        conflicts.push(Conflict {
                            table: table.to_string(),
                            entry: key.clone(),
                            field: "<entry>".to_string(),
                            previous_mod: field_owner
                                .get(&(key.clone(), "<entry>".to_string()))
                                .cloned()
                                .unwrap_or_default(),
                            winning_mod: label.clone(),
                        });
                    } else if !merged.contains_key(&key) {
                        added.push(key.clone());
                    }
                    merged.insert(key.clone(), mod_v.clone());
                    field_owner.insert((key.clone(), "<entry>".to_string()), label.clone());
                }
                // Non-object entry: whole-value last-wins.
                _ => {
                    merged.insert(key.clone(), mod_v.clone());
                }
            }
        }
    }

    let mut out = Vec::new();
    for key in order.iter().chain(added.iter()) {
        if let Some(v) = merged.get(key) {
            out.push(serde_json::from_value::<T>(v.clone())?);
        }
    }
    Ok((out, conflicts))
}

/// Field-level three-way merge for a singleton document (e.g. gamedata).
fn merge_singleton<T: Serialize + DeserializeOwned>(
    table: &str,
    base: Option<&T>,
    mods: Vec<(String, T)>,
) -> Result<(Option<T>, Vec<Conflict>), MergeError> {
    let base_v = base.map(serde_json::to_value).transpose()?;
    let mut merged_v = base_v.clone();
    let mut field_owner: HashMap<String, String> = HashMap::new();
    let mut conflicts = Vec::new();

    for (label, item) in &mods {
        let mod_v = serde_json::to_value(item)?;
        match (&base_v, &mut merged_v) {
            (Some(Value::Object(base_obj)), Some(Value::Object(merged_obj))) => {
                if let Value::Object(mod_obj) = &mod_v {
                    for (field, mod_field) in mod_obj {
                        if base_obj.get(field) == Some(mod_field) {
                            continue;
                        }
                        if let Some(prev) = field_owner.get(field)
                            && merged_obj.get(field) != Some(mod_field)
                        {
                            conflicts.push(Conflict {
                                table: table.to_string(),
                                entry: table.to_string(),
                                field: field.clone(),
                                previous_mod: prev.clone(),
                                winning_mod: label.clone(),
                            });
                        }
                        merged_obj.insert(field.clone(), mod_field.clone());
                        field_owner.insert(field.clone(), label.clone());
                    }
                }
            }
            // No base (or non-object): last mod wins outright.
            _ => merged_v = Some(mod_v),
        }
    }

    let out = merged_v.map(serde_json::from_value).transpose()?;
    Ok((out, conflicts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Entry {
        #[serde(rename = "@name")]
        name: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        hp: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        dmg: Option<i64>,
    }

    impl Keyed for Entry {
        fn key(&self) -> &str {
            &self.name
        }
    }

    fn entry(name: &str, hp: Option<i64>, dmg: Option<i64>) -> Entry {
        Entry {
            name: name.to_string(),
            hp,
            dmg,
        }
    }

    fn find<'a>(v: &'a [Entry], name: &str) -> &'a Entry {
        v.iter().find(|e| e.name == name).expect("entry present")
    }

    /// Mirrors the abilities/leaders schema, whose identity serializes as a
    /// capitalised `@Name`. Keying off `Keyed::key` must be independent of that.
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct CapEntry {
        #[serde(rename = "@Name")]
        name: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        val: Option<i64>,
    }

    impl Keyed for CapEntry {
        fn key(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn non_overlapping_field_edits_combine() {
        let base = vec![entry("marine", Some(100), Some(10))];
        let mod_a = vec![entry("marine", Some(200), Some(10))]; // changed hp
        let mod_b = vec![entry("marine", Some(100), Some(50))]; // changed dmg

        let (out, conflicts) =
            merge_table("objects", &base, &[("A".into(), mod_a), ("B".into(), mod_b)]).unwrap();

        let m = find(&out, "marine");
        assert_eq!(m.hp, Some(200));
        assert_eq!(m.dmg, Some(50));
        assert!(conflicts.is_empty());
    }

    #[test]
    fn same_field_divergence_is_a_conflict_last_wins() {
        let base = vec![entry("marine", Some(100), None)];
        let mod_a = vec![entry("marine", Some(200), None)];
        let mod_b = vec![entry("marine", Some(300), None)];

        let (out, conflicts) =
            merge_table("objects", &base, &[("A".into(), mod_a), ("B".into(), mod_b)]).unwrap();

        assert_eq!(find(&out, "marine").hp, Some(300)); // later mod wins
        assert_eq!(conflicts.len(), 1);
        let c = &conflicts[0];
        assert_eq!(c.entry, "marine");
        assert_eq!(c.field, "hp");
        assert_eq!(c.previous_mod, "A");
        assert_eq!(c.winning_mod, "B");
    }

    #[test]
    fn added_entry_survives() {
        let base = vec![entry("marine", Some(100), None)];
        let mod_a = vec![
            entry("marine", Some(100), None),
            entry("spartan", Some(500), None),
        ];

        let (out, conflicts) = merge_table("objects", &base, &[("A".into(), mod_a)]).unwrap();

        assert_eq!(out.len(), 2);
        assert_eq!(find(&out, "spartan").hp, Some(500));
        assert!(conflicts.is_empty());
    }

    #[test]
    fn base_order_is_preserved() {
        let base = vec![
            entry("a", None, None),
            entry("b", None, None),
            entry("c", None, None),
        ];
        let mod_a = vec![entry("b", Some(1), None)];

        let (out, _) = merge_table("objects", &base, &[("A".into(), mod_a)]).unwrap();
        let names: Vec<&str> = out.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["a", "b", "c"]);
    }

    /// Regression: tables whose identity serializes as something other than
    /// `@name` (here `@Name`) must still merge by identity rather than treating
    /// a modded entry as new and duplicating the base copy.
    #[test]
    fn non_at_name_identity_merges_without_duplicating() {
        let base = vec![CapEntry {
            name: "UnscLockdown".into(),
            val: Some(1),
        }];
        let mod_a = vec![CapEntry {
            name: "UnscLockdown".into(),
            val: Some(2),
        }];

        let (out, conflicts) = merge_table("abilities", &base, &[("A".into(), mod_a)]).unwrap();

        assert_eq!(out.len(), 1, "must edit the base entry, not duplicate it");
        assert_eq!(out[0].val, Some(2));
        assert!(conflicts.is_empty());
    }
}
