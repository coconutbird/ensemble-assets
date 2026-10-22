//! The `_update` overlays of `objects.xml`, `squads.xml` and `techs.xml`.
//!
//! The engine does not merge fields between a base file and its `_update` file. It replaces or
//! appends whole records. `BDatabase_setupDependentData` @ `0x1401ee690` gives the IDs, and
//! `BDatabase_setupProtoObjects` @ `0x1401f27f0`, `BDatabase_setupProtoSquads` @ `0x1401f2fc0`
//! and `BDatabase_setupProtoTechs` @ `0x1401f4e80` load the records in the same order:
//!
//! - Objects and squads: the engine skips each base record that has `update="true"`. Then it
//!   appends each record of the `_update` file. A record gets the next index.
//! - Techs: the engine appends each record of `techs_update.xml` after the records of
//!   `techs.xml`. It does not read the `update` attribute.
//!
//! The name maps of the engine compare names without case and keep the first name. A second
//! object with the same name stops the database setup
//! (`"Could not add object %s (duplicate?)"`). A second squad or tech with the same name keeps
//! its index, but a name lookup finds the first record.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use super::{ProtoObject, Squad, Tech};

/// A table that the engine loads from a base file and an `_update` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayTable {
    /// `objects.xml` and `objects_update.xml`.
    Objects,
    /// `squads.xml` and `squads_update.xml`.
    Squads,
    /// `techs.xml` and `techs_update.xml`.
    Techs,
}

impl OverlayTable {
    /// Return the game path of the base file, for example `"data\\objects.xml"`.
    #[must_use]
    pub const fn base_path(self) -> &'static str {
        match self {
            Self::Objects => "data\\objects.xml",
            Self::Squads => "data\\squads.xml",
            Self::Techs => "data\\techs.xml",
        }
    }

    /// Return the game path of the `_update` file, for example `"data\\objects_update.xml"`.
    #[must_use]
    pub const fn update_path(self) -> &'static str {
        match self {
            Self::Objects => "data\\objects_update.xml",
            Self::Squads => "data\\squads_update.xml",
            Self::Techs => "data\\techs_update.xml",
        }
    }

    /// Return the name of the root element of both files, for example `"Objects"`.
    #[must_use]
    pub const fn root_name(self) -> &'static str {
        match self {
            Self::Objects => "Objects",
            Self::Squads => "Squads",
            Self::Techs => "TechTree",
        }
    }

    /// Return the name of a record element, for example `"Object"`.
    #[must_use]
    pub const fn record_name(self) -> &'static str {
        match self {
            Self::Objects => "Object",
            Self::Squads => "Squad",
            Self::Techs => "Tech",
        }
    }
}

/// Apply `objects_update.xml` to the records of `objects.xml`.
///
/// # Errors
///
/// Returns [`crate::Error::DuplicateName`] if two records of the result have the same name
/// (case-insensitive). The engine stops the database setup in this case.
pub fn objects(
    base: Vec<ProtoObject>,
    update: Vec<ProtoObject>,
) -> crate::Result<Vec<ProtoObject>> {
    let records = replace_updated(base, update, |object| object.update);
    let mut names = BTreeSet::new();
    for object in &records {
        if !names.insert(object.name.to_ascii_uppercase()) {
            return Err(crate::Error::DuplicateName {
                table: "object",
                name: object.name.clone(),
            });
        }
    }
    Ok(records)
}

/// Apply `squads_update.xml` to the records of `squads.xml`.
///
/// The result keeps squads with the same name.
#[must_use]
pub fn squads(base: Vec<Squad>, update: Vec<Squad>) -> Vec<Squad> {
    replace_updated(base, update, |squad| squad.update)
}

/// Append the records of `techs_update.xml` to the records of `techs.xml`.
///
/// The result keeps techs with the same name.
#[must_use]
pub fn techs(mut base: Vec<Tech>, update: Vec<Tech>) -> Vec<Tech> {
    base.extend(update);
    base
}

/// Skip each base record that has `update="true"`, then append each update record.
fn replace_updated<T>(base: Vec<T>, update: Vec<T>, flag: fn(&T) -> Option<bool>) -> Vec<T> {
    let mut records: Vec<T> = base
        .into_iter()
        .filter(|record| flag(record) != Some(true))
        .collect();
    records.extend(update);
    records
}

/// Return a copy of `records` without the `update` flags.
///
/// The engine keeps each record of a base file that has no `update="true"`. Write this copy to
/// the base file, and an empty `_update` file, to make the engine load the same table.
pub(super) fn clear_update_flags<T: Clone>(
    records: &[T],
    flag: fn(&mut T) -> &mut Option<bool>,
) -> Vec<T> {
    records
        .iter()
        .map(|record| {
            let mut record = record.clone();
            *flag(&mut record) = None;
            record
        })
        .collect()
}
