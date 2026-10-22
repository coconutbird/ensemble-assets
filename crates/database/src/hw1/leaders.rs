//! Parser for `leaders.xml.xmb` — leader definitions.
//!
//! Each `<Leader>` element describes a playable leader (Cutter, Arbiter, etc.).

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use super::Vector3;
use crate::node_ext::parse_table;

/// A single leader definition from `leaders.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Leader {
    /// Leader name (unique key), e.g. `"Cutter"`, `"Arbiter"`.
    #[serde(rename = "@Name", default)]
    pub name: String,
    /// Icon path.
    #[serde(rename = "@Icon")]
    pub icon: Option<String>,
    /// Leader picker order index.
    #[serde(rename = "@LeaderPickerOrder")]
    pub leader_picker_order: Option<i32>,
    /// Stats ID.
    #[serde(rename = "@StatsID")]
    pub stats_id: Option<i32>,
    /// Default player slot flags (hex string).
    #[serde(rename = "@DefaultPlayerSlotFlags")]
    pub default_player_slot_flags: Option<String>,
    /// Alpha flag.
    #[serde(rename = "@Alpha")]
    pub alpha: Option<i32>,
    /// Whether this is a random leader placeholder.
    #[serde(rename = "@Random")]
    pub random: Option<bool>,
    /// Civilization name.
    #[serde(rename = "Civ")]
    pub civ: Option<String>,
    /// Tech to apply for this leader.
    #[serde(rename = "Tech")]
    pub tech: Option<String>,
    /// Primary leader power.
    #[serde(rename = "Power")]
    pub power: Option<String>,
    /// Name string ID.
    #[serde(rename = "NameID")]
    pub name_id: Option<i32>,
    /// Description string ID.
    #[serde(rename = "DescriptionID")]
    pub description_id: Option<i32>,
    /// Flash civ ID.
    #[serde(rename = "FlashCivID")]
    pub flash_civ_id: Option<i32>,
    /// Flash image name.
    #[serde(rename = "FlashImg")]
    pub flash_img: Option<String>,
    /// Flash portrait image path.
    #[serde(rename = "FlashPortrait")]
    pub flash_portrait: Option<String>,
    /// UI control background image path.
    #[serde(rename = "UIControlBackground")]
    pub ui_control_background: Option<String>,
    /// Starting resources.
    #[serde(rename = "Resource", default)]
    pub resources: Vec<ResourceEntry>,
    /// Starting unit definitions.
    #[serde(rename = "StartingUnit", default)]
    pub starting_units: Vec<StartingUnit>,
    /// Starting squad definitions.
    #[serde(rename = "StartingSquad", default)]
    pub starting_squads: Vec<StartingSquad>,
    /// Rally point offset.
    #[serde(rename = "RallyPointOffset")]
    pub rally_point_offset: Option<Vector3>,
    /// Groups of support powers exposed by this leader.
    #[serde(rename = "SupportPower", default)]
    pub support_powers: Vec<SupportPower>,
    /// Repair rate.
    #[serde(rename = "RepairRate")]
    pub repair_rate: Option<f32>,
    /// Repair delay in seconds.
    #[serde(rename = "RepairDelay")]
    pub repair_delay: Option<f32>,
    /// Repair cost.
    #[serde(rename = "RepairCost", default)]
    pub repair_cost: Vec<ResourceEntry>,
    /// Repair time in seconds.
    #[serde(rename = "RepairTime")]
    pub repair_time: Option<f32>,
    /// Population caps.
    #[serde(rename = "Pop", default)]
    pub pops: Vec<PopEntry>,
    /// Reverse hot-drop resource cost.
    #[serde(rename = "ReverseHotDropCost")]
    pub reverse_hot_drop_cost: Option<f32>,
    /// Test flag (debug/development).
    #[serde(rename = "@Test")]
    pub test: Option<bool>,
}

/// A resource entry (type + amount).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ResourceEntry {
    /// Resource name from `@Type`, for example `Supplies` or `Power`.
    #[serde(rename = "@Type", default)]
    pub resource_type: String,
    /// Starting amount of the resource.
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// A starting unit definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StartingUnit {
    /// Proto-object name of the unit.
    #[serde(rename = "$text", default)]
    pub proto_object: String,
    /// Spawn offset from the start position, from `@Offset`.
    #[serde(rename = "@Offset")]
    pub offset: Option<Vector3>,
    /// Proto-object that the unit builds on start, from `@BuildOther`.
    #[serde(rename = "@BuildOther")]
    pub build_other: Option<String>,
    /// When `true`, the unit is doppled on start (`@DoppleOnStart`).
    #[serde(rename = "@DoppleOnStart")]
    pub dopple_on_start: Option<bool>,
}

/// A starting squad definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StartingSquad {
    /// Proto-squad name of the squad.
    #[serde(rename = "$text", default)]
    pub proto_squad: String,
    /// When `true`, the squad flies in on start (`@FlyIn`).
    #[serde(rename = "@FlyIn")]
    pub fly_in: Option<bool>,
    /// Spawn offset from the start position, from `@Offset`.
    #[serde(rename = "@Offset")]
    pub offset: Option<Vector3>,
}

/// A support-power group and its menu placement/prerequisite metadata.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SupportPower {
    /// Slot index of the power icon in the support-power menu.
    #[serde(rename = "@IconLocation")]
    pub icon_location: Option<i32>,
    /// Tech that the player must have before the group unlocks.
    #[serde(rename = "@TechPrereq")]
    pub tech_prereq: Option<String>,
    /// Power names in this group, from the `<Power>` children.
    #[serde(rename = "Power", default)]
    pub powers: Vec<String>,
}

/// A population entry (type + count + max).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PopEntry {
    /// Population type name from `@Type`.
    #[serde(rename = "@Type", default)]
    pub pop_type: String,
    /// Starting population count.
    #[serde(rename = "$text", default)]
    pub count: f32,
    /// Maximum population cap, from `@Max`.
    #[serde(rename = "@Max")]
    pub max: Option<f32>,
}

/// Parse all leaders from a `leaders.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or a leader cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<Leader>> {
    parse_table(doc, "Leaders", "Leader")
}
