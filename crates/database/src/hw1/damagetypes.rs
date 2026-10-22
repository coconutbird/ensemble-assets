//! Parser for `damagetypes.xml.xmb` — damage type definitions.
//!
//! Each `<DamageType>` defines an armor/damage category (Light, Heavy, Building, etc.).

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::parse_table;

/// A single damage type definition from `damagetypes.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DamageType {
    /// Damage type name, e.g. `"Light"`, `"Heavy"`, `"Building"`.
    #[serde(rename = "$text", default)]
    pub name: String,
    /// Whether this type has an attack rating.
    #[serde(rename = "@AttackRating")]
    pub attack_rating: Option<bool>,
    /// Whether this is a base type.
    #[serde(rename = "@BaseType")]
    pub base_type: Option<bool>,
    /// Whether this is a shielded type.
    #[serde(rename = "@Shielded")]
    pub shielded: Option<bool>,
}

/// Parse all damage types from a `damagetypes.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or a damage type cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<DamageType>> {
    parse_table(doc, "DamageTypes", "DamageType")
}
