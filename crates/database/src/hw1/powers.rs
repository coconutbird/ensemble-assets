//! Parser for `powers.xml.xmb` — leader power definitions.
//!
//! Each `<Power>` element describes a leader power (orbital bombardment, MAC blast, etc.).

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Deserializer, Serialize};

use crate::node_ext::expect_root;

/// A single leader power definition from `powers.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Power {
    /// Power name (unique key), e.g. `"UnscLeaderNuke"`.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Trigger script file name.
    #[serde(rename = "TriggerScript")]
    pub trigger_script: Option<String>,
    /// Script invoked by the command path for this power.
    #[serde(rename = "CommandTriggerScript")]
    pub command_trigger_script: Option<String>,
    /// Power attributes (contains data levels too).
    #[serde(rename = "Attributes")]
    pub attributes: Option<PowerAttributes>,
}

/// Attributes block for a power.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PowerAttributes {
    /// Power type: `"Transport"`, `"Cleansing"`, etc.
    #[serde(rename = "PowerType")]
    pub power_type: Option<String>,
    /// Power action kind.
    #[serde(rename = "Action")]
    pub action: Option<String>,
    /// Presence-based: element exists = true.
    #[serde(
        rename = "InfiniteUses",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub infinite_uses: Option<bool>,
    /// Presence-based: element exists = true.
    #[serde(
        rename = "LeaderPower",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub leader_power: Option<bool>,
    /// Auto-recharge interval in milliseconds.
    #[serde(rename = "AutoRecharge")]
    pub auto_recharge: Option<u32>,
    /// Display name string ID.
    #[serde(rename = "DisplayNameID")]
    pub display_name_id: Option<i32>,
    /// Rollover text string ID.
    #[serde(rename = "RolloverTextID")]
    pub rollover_text_id: Option<i32>,
    /// Prereq text string ID.
    #[serde(rename = "PrereqTextID")]
    pub prereq_text_id: Option<i32>,
    /// Text shown while choosing a target.
    #[serde(rename = "ChooseTextID")]
    pub choose_text_id: Option<i32>,
    /// Icon path.
    #[serde(rename = "Icon")]
    pub icon: Option<String>,
    /// Icon location indices. The runtime appends every repeated element.
    #[serde(rename = "IconLocation", default)]
    pub icon_locations: Vec<i32>,
    /// UI radius.
    #[serde(rename = "UIRadius")]
    pub ui_radius: Option<f32>,
    /// Cost element.
    #[serde(rename = "Cost")]
    pub cost: Option<PowerCost>,
    /// Object-dependent dynamic cost multipliers.
    #[serde(rename = "DynamicCost", default)]
    pub dynamic_costs: Vec<DynamicCost>,
    /// Object-dependent target effectiveness values.
    #[serde(rename = "TargetEffectiveness", default)]
    pub target_effectiveness: Vec<TargetEffectiveness>,
    /// Show transport arrows.
    #[serde(
        rename = "ShowTransportArrows",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub show_transport_arrows: Option<bool>,
    /// Presence-based: element exists = true.
    #[serde(
        rename = "ShowLimit",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub show_limit: Option<bool>,
    /// Maximum number of uses when the power is limited.
    #[serde(rename = "UseLimit")]
    pub use_limit: Option<i32>,
    /// Min distance to squad.
    #[serde(rename = "MinDistanceToSquad")]
    pub min_distance_to_squad: Option<f32>,
    /// Max distance to squad.
    #[serde(rename = "MaxDistanceToSquad")]
    pub max_distance_to_squad: Option<f32>,
    /// Base data level (inside Attributes).
    #[serde(rename = "BaseDataLevel")]
    pub base_data_level: Option<DataLevel>,
    /// Data levels (inside Attributes).
    #[serde(rename = "DataLevel", default)]
    pub data_levels: Vec<DataLevel>,
    /// Camera effect on power activation.
    #[serde(rename = "CameraEffectIn")]
    pub camera_effect_in: Option<String>,
    /// Camera effect on power deactivation.
    #[serde(rename = "CameraEffectOut")]
    pub camera_effect_out: Option<String>,
    /// Allow user camera scroll during power.
    #[serde(
        rename = "CameraEnableUserScroll",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub camera_enable_user_scroll: Option<bool>,
    /// Allow user camera yaw during power.
    #[serde(
        rename = "CameraEnableUserYaw",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub camera_enable_user_yaw: Option<bool>,
    /// Allow user camera zoom during power.
    #[serde(
        rename = "CameraEnableUserZoom",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub camera_enable_user_zoom: Option<bool>,
    /// Allow the camera to choose its zoom automatically.
    #[serde(
        rename = "CameraEnableAutoZoom",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub camera_enable_auto_zoom: Option<bool>,
    /// Apply automatic camera zoom immediately.
    #[serde(
        rename = "CameraEnableAutoZoomInstant",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub camera_enable_auto_zoom_instant: Option<bool>,
    /// Camera pitch maximum during power.
    #[serde(rename = "CameraPitchMax")]
    pub camera_pitch_max: Option<f32>,
    /// Camera pitch minimum during power.
    #[serde(rename = "CameraPitchMin")]
    pub camera_pitch_min: Option<f32>,
    /// Camera zoom maximum during power.
    #[serde(rename = "CameraZoomMax")]
    pub camera_zoom_max: Option<f32>,
    /// Camera zoom minimum during power.
    #[serde(rename = "CameraZoomMin")]
    pub camera_zoom_min: Option<f32>,
    /// Minigame name associated with this power.
    #[serde(rename = "Minigame")]
    pub minigame: Option<String>,
    /// Presence-based multi-recharge behavior flag.
    #[serde(
        rename = "MultiRechargePower",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub multi_recharge_power: Option<bool>,
    /// Power cannot be disrupted.
    #[serde(
        rename = "NotDisruptable",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub not_disruptable: Option<bool>,
    /// Sequential recharge flag.
    #[serde(
        rename = "SequentialRecharge",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub sequential_recharge: Option<bool>,
    /// Show in power menu.
    #[serde(
        rename = "ShowInPowerMenu",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub show_in_power_menu: Option<bool>,
    /// Show target highlight effect.
    #[serde(
        rename = "ShowTargetHighlight",
        default,
        deserialize_with = "deserialize_optional_target_highlight"
    )]
    pub show_target_highlight: Option<TargetHighlight>,
    /// Tech prerequisite names. The runtime appends every repeated element.
    #[serde(rename = "TechPrereq", default)]
    pub tech_prerequisites: Vec<String>,
    /// Whether this is a unit power.
    #[serde(
        rename = "UnitPower",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub unit_power: Option<bool>,
    /// Population type reference.
    #[serde(rename = "Pop", default)]
    pub population: Vec<PowerPopulation>,
    /// Proto-objects created or owned by the power.
    #[serde(rename = "ChildObjects")]
    pub child_objects: Option<PowerChildObjects>,
    /// Legacy Flash UI marker retained by shipped XML but ignored by the runtime power loader.
    #[serde(
        rename = "FlashUI",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub flash_ui: Option<bool>,
}

/// Cost element with one attribute per registered game resource.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PowerCost {
    #[serde(rename = "@Supplies")]
    pub supplies: Option<f32>,
    #[serde(rename = "@Power")]
    pub power: Option<f32>,
    #[serde(rename = "@CampaignFoo")]
    pub campaign_foo: Option<f32>,
    #[serde(rename = "@Collectable")]
    pub collectable: Option<f32>,
}

/// An object-specific cost multiplier.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DynamicCost {
    #[serde(rename = "@ObjectType", default)]
    pub object_type: String,
    #[serde(rename = "$text", default = "default_one")]
    pub multiplier: f32,
}

/// An object-specific target-effectiveness grade.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TargetEffectiveness {
    #[serde(rename = "@ObjectType", default)]
    pub object_type: String,
    #[serde(rename = "$text", default)]
    pub grade: i32,
}

/// Target-highlight settings, including the optional relation filter.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TargetHighlight {
    #[serde(rename = "@ObjectType")]
    pub object_type: Option<String>,
    #[serde(rename = "@Relation")]
    pub relation: Option<String>,
    #[serde(rename = "$text")]
    pub enabled: Option<bool>,
}

fn deserialize_optional_target_highlight<'de, D>(
    deserializer: D,
) -> Result<Option<TargetHighlight>, D::Error>
where
    D: Deserializer<'de>,
{
    TargetHighlight::deserialize(deserializer).map(Some)
}

/// A population amount used by a power.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PowerPopulation {
    #[serde(rename = "@type")]
    pub population_type: Option<String>,
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// Wrapper for the `<ChildObjects>` element.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PowerChildObjects {
    #[serde(rename = "Object", default)]
    pub objects: Vec<String>,
}

/// A data level entry (level-specific power parameters).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DataLevel {
    /// Level index (0-based). Absent for `BaseDataLevel`.
    #[serde(rename = "@level")]
    pub level: Option<u32>,
    /// Key-value data entries.
    #[serde(rename = "Data", default)]
    pub entries: Vec<DataEntry>,
}

/// A single data entry within a data level.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DataEntry {
    /// Data type: `"float"`, `"int"`, `"sound"`, `"protoobject"`, `"texture"`, etc.
    #[serde(rename = "@type", default)]
    pub data_type: String,
    /// Data name key.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Data value (text content).
    #[serde(rename = "$text", default)]
    pub value: String,
}

fn default_one() -> f32 {
    1.0
}

/// Parse all powers from a `powers.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or a power cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<Power>> {
    let root = expect_root(doc, "Powers")?;
    let powers: Vec<Power> = root
        .children
        .iter()
        .filter(|c| c.name == "Power")
        .map(bdt_serde::from_node)
        .collect::<Result<_, _>>()?;
    Ok(powers)
}
