//! Parser for `objecttypes.xml.xmb`: the declared object types.
//!
//! The engine keeps objects and object types in one name map
//! (`BDatabase_setupDependentData` @ `0x1401ee690`):
//!
//! - Object `i` (after the `_update` overlay) has the ID `i`.
//! - Object type `j` has the ID `objects.len() + j`. Each child of the root counts, and the name
//!   is the text of the child.
//! - The map compares names without case and keeps the first name. A type with the name of an
//!   object, or of an earlier type, keeps its ID but a name lookup does not find it.
//!
//! `BProtoObject_load` @ `0x140341620` looks up each `<ObjectType>` of an object in this map. It
//! keeps the name only if the ID is a type ID (`>= objects.len()`). It ignores an undeclared name
//! or an object name. `BDatabase_setupDecomposedObjectTypes` @ `0x1401f2d60` then maps an object
//! ID to that object, and a type ID to each object that has the type.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::expect_root;

use super::ProtoObject;

/// One declared object type from `objecttypes.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectType {
    /// Type name, for example `"_Building"` or `"Infantry"`.
    #[serde(rename = "$text", default)]
    pub name: String,
}

/// Parse all object types from an `objecttypes.xml.xmb` document.
///
/// Each child of the root is one type, as in the engine.
///
/// # Errors
///
/// Returns an error if the root is not `ObjectTypes` or a child cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<ObjectType>> {
    let root = expect_root(doc, "ObjectTypes")?;
    let types = root
        .children
        .iter()
        .map(bdt_serde::from_node)
        .collect::<Result<_, _>>()?;
    Ok(types)
}

/// The engine name map of objects and object types.
///
/// Build it once with [`ObjectTypeIndex::new`] after the database is loaded.
#[derive(Debug, Clone, Default)]
pub struct ObjectTypeIndex {
    /// Engine ID for each upper-case name. The first name keeps its ID.
    ids: BTreeMap<String, usize>,
    /// Number of objects. Type IDs start at this value.
    object_count: usize,
}

impl ObjectTypeIndex {
    /// Build the name map from the object table and the object type table.
    #[must_use]
    pub fn new(objects: &[ProtoObject], object_types: &[ObjectType]) -> Self {
        let mut ids = BTreeMap::new();
        let names = objects
            .iter()
            .map(|object| &object.name)
            .chain(object_types.iter().map(|object_type| &object_type.name));
        for (id, name) in names.enumerate() {
            ids.entry(name.to_ascii_uppercase()).or_insert(id);
        }
        Self {
            ids,
            object_count: objects.len(),
        }
    }

    /// Return the engine ID of `name`: an object index, or `objects.len()` plus a type index.
    ///
    /// The lookup ignores case.
    #[must_use]
    pub fn id(&self, name: &str) -> Option<usize> {
        self.ids.get(&name.to_ascii_uppercase()).copied()
    }

    /// Return the index in `Database::object_types` of the declared type `name`.
    ///
    /// Returns `None` for an undeclared name and for an object name.
    #[must_use]
    pub fn declared_type(&self, name: &str) -> Option<usize> {
        self.id(name)?.checked_sub(self.object_count)
    }

    /// Return the declared types of `object` as indices in `Database::object_types`.
    ///
    /// The result is in ascending order and has no duplicates, like the type bit set of the
    /// engine.
    #[must_use]
    pub fn types_of(&self, object: &ProtoObject) -> Vec<usize> {
        let mut types: Vec<usize> = object
            .object_types
            .iter()
            .filter_map(|name| self.declared_type(name))
            .collect();
        types.sort_unstable();
        types.dedup();
        types
    }
}
