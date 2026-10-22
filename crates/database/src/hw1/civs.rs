//! Parser for `civs.xml.xmb` — civilization definitions.
//!
//! Each `<Civ>` element describes a faction (UNSC, Covenant, Gaia).

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::parse_table;

/// A single civilization definition from `civs.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Civ {
    /// Civilization name, e.g. `"UNSC"`, `"Covenant"`, `"Gaia"`.
    #[serde(rename = "Name", default)]
    pub name: String,
    /// Alpha flag (from attribute).
    #[serde(rename = "@Alpha")]
    pub alpha: Option<i32>,
    /// Display name string ID.
    #[serde(rename = "DisplayNameID")]
    pub display_name_id: Option<i32>,
    /// Civ tech to apply at game start.
    #[serde(rename = "CivTech")]
    pub civ_tech: Option<String>,
    /// Command acknowledgement object.
    #[serde(rename = "CommandAckObject")]
    pub command_ack_object: Option<String>,
    /// Rally point object.
    #[serde(rename = "RallyPointObject")]
    pub rally_point_object: Option<String>,
    /// Local rally point object.
    #[serde(rename = "LocalRallyPointObject")]
    pub local_rally_point_object: Option<String>,
    /// Hull expansion value.
    #[serde(rename = "ExpandHull")]
    pub expand_hull: Option<f32>,
    /// Terrain push-off value.
    #[serde(rename = "TerrainPushOff")]
    pub terrain_push_off: Option<f32>,
    /// Building magnet range.
    #[serde(rename = "BuildingMagnetRange")]
    pub building_magnet_range: Option<f32>,
    /// Transport unit name.
    #[serde(rename = "Transport")]
    pub transport: Option<String>,
    /// Transport trigger unit name.
    #[serde(rename = "TransportTrigger")]
    pub transport_trigger: Option<String>,
    /// Sound bank file.
    #[serde(rename = "SoundBank")]
    pub sound_bank: Option<String>,
    /// Leader menu name string ID.
    #[serde(rename = "LeaderMenuNameID")]
    pub leader_menu_name_id: Option<i32>,
    /// Whether powers come from the hero unit.
    #[serde(
        rename = "PowerFromHero",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub power_from_hero: Option<bool>,
    /// UI control background image path.
    #[serde(rename = "UIControlBackground")]
    pub ui_control_background: Option<String>,
}

/// Parse all civilizations from a `civs.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or a civilization cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<Civ>> {
    parse_table(doc, "Civs", "Civ")
}
