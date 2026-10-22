//! Parser for `techs.xml.xmb` — tech tree definitions.
//!
//! Techs represent upgrades, unlocks, and game state modifications.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::parse_table;

/// A tech definition from `techs.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Tech {
    /// Tech name (unique key), e.g. `"Unsc_warthog_upgrade1"`.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Tech type: `"Normal"`, etc.
    #[serde(rename = "@type")]
    pub tech_type: Option<String>,
    /// Database ID.
    #[serde(rename = "DBID")]
    pub dbid: Option<i32>,
    /// Research time in seconds.
    #[serde(rename = "ResearchPoints")]
    pub research_points: Option<f32>,
    /// Status: `"OBTAINABLE"`, etc.
    #[serde(rename = "Status")]
    pub status: Option<String>,
    /// Flags.
    #[serde(rename = "Flag", default)]
    pub flags: Vec<String>,
    /// Effects wrapper.
    #[serde(rename = "Effects")]
    pub effects: Option<EffectsWrapper>,
    /// Display name string ID.
    #[serde(rename = "DisplayNameID")]
    pub display_name_id: Option<i32>,
    /// Rollover text string ID.
    #[serde(rename = "RolloverTextID")]
    pub rollover_text_id: Option<i32>,
    /// Prerequisite text string ID.
    #[serde(rename = "PrereqTextID")]
    pub prereq_text_id: Option<i32>,
    /// Prerequisites wrapper.
    #[serde(rename = "Prereqs")]
    pub prereqs: Option<PrereqsWrapper>,
    /// Alternate prerequisite group supported by the game loader.
    #[serde(rename = "OrPrereqs")]
    pub or_prereqs: Option<PrereqsWrapper>,
    /// Research animation.
    #[serde(rename = "ResearchAnim")]
    pub research_anim: Option<String>,
    /// Research complete sound event.
    #[serde(rename = "ResearchCompleteSound")]
    pub research_complete_sound: Option<String>,
    /// Alpha flag (attribute on Tech node).
    #[serde(rename = "@Alpha")]
    pub alpha: Option<i32>,
    /// Resource costs.
    #[serde(rename = "Cost", default)]
    pub costs: Vec<TechCost>,
    /// Icon path.
    #[serde(rename = "Icon")]
    pub icon: Option<String>,
    /// Stats object reference.
    #[serde(rename = "StatsObject")]
    pub stats_object: Option<String>,
}

/// Wrapper for the `<Effects>` element containing `<Effect>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EffectsWrapper {
    /// Effects from the `<Effect>` children.
    #[serde(rename = "Effect", default)]
    pub entries: Vec<TechEffect>,
}

/// A single tech effect.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TechEffect {
    /// Effect type: `"Data"`, etc.
    #[serde(rename = "@type", default)]
    pub effect_type: String,
    /// Numeric amount.
    #[serde(rename = "@amount")]
    pub amount: Option<f32>,
    /// Subtype: `"Bounty"`, `"AbilityDisabled"`, etc.
    #[serde(rename = "@subtype")]
    pub subtype: Option<String>,
    /// Relativity: `"Percent"`, `"Absolute"`, etc.
    #[serde(rename = "@relativity")]
    pub relativity: Option<String>,
    /// Target element.
    #[serde(rename = "Target")]
    pub target: Option<EffectTarget>,
    /// Text content of the effect element.
    #[serde(rename = "$text")]
    pub value: Option<String>,
    /// Action name.
    #[serde(rename = "@action")]
    pub action: Option<String>,
    /// All actions flag.
    #[serde(rename = "@allactions")]
    pub allactions: Option<bool>,
    /// Command data.
    #[serde(rename = "@CommandData")]
    pub command_data: Option<String>,
    /// Command type.
    #[serde(rename = "@commandType")]
    pub command_type: Option<String>,
    /// Rate-table reference used by rate-related data effects.
    #[serde(rename = "@Rate")]
    pub rate: Option<String>,
    /// Weapon-type reference used by weapon damage modifier effects.
    #[serde(rename = "@WeaponType")]
    pub weapon_type: Option<String>,
    /// Damage-type reference used by weapon damage modifier effects.
    #[serde(rename = "@DamageType")]
    pub damage_type: Option<String>,
    /// String-table identifier used by data effects that target localized text.
    #[serde(rename = "@StringID")]
    pub string_id: Option<u32>,
    /// From type (for conversion effects).
    #[serde(rename = "@FromType")]
    pub from_type: Option<String>,
    /// To type (for conversion effects).
    #[serde(rename = "@ToType")]
    pub to_type: Option<String>,
    /// Hardpoint name.
    #[serde(rename = "@Hardpoint")]
    pub hardpoint: Option<String>,
    /// HP bar override.
    #[serde(rename = "@hpbar")]
    pub hpbar: Option<String>,
    /// Icon name.
    #[serde(rename = "@iconName")]
    pub icon_name: Option<String>,
    /// Icon type.
    #[serde(rename = "@iconType")]
    pub icon_type: Option<String>,
    /// Impact effect name.
    #[serde(rename = "@impactEffect")]
    pub impact_effect: Option<String>,
    /// Population type.
    #[serde(rename = "@popType")]
    pub pop_type: Option<String>,
    /// Power name reference.
    #[serde(rename = "@power")]
    pub power: Option<String>,
    /// Resource type.
    #[serde(rename = "@Resource")]
    pub resource: Option<String>,
    /// Squad name reference.
    #[serde(rename = "@squadName")]
    pub squad_name: Option<String>,
    /// Squad type used by squad-targeted and attach-squad effects.
    #[serde(rename = "@SquadType")]
    pub squad_type: Option<String>,
    /// Unit type reference.
    #[serde(rename = "@unitType")]
    pub unit_type: Option<String>,
    /// Ability name reference.
    #[serde(rename = "@Ability")]
    pub ability: Option<String>,
}

/// Target element within an effect: `<Target type="...">value</Target>`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EffectTarget {
    /// Target kind from `@type`, for example `ProtoUnit` or `ProtoSquad`.
    #[serde(rename = "@type")]
    pub target_type: Option<String>,
    /// Name of the target.
    #[serde(rename = "$text")]
    pub value: Option<String>,
}

/// A resource cost entry for a tech.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TechCost {
    /// Resource name from `@resourcetype`.
    #[serde(rename = "@resourcetype", default)]
    pub resource_type: String,
    /// Cost amount.
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// Prerequisites wrapper containing tech status entries.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PrereqsWrapper {
    /// Required tech states, from the `<TechStatus>` children.
    #[serde(rename = "TechStatus", default)]
    pub entries: Vec<TechStatusEntry>,
    /// Required unit counts, from the `<TypeCount>` children.
    #[serde(rename = "TypeCount", default)]
    pub type_counts: Vec<TypeCountEntry>,
}

/// A prerequisite tech status entry.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TechStatusEntry {
    /// Name of the required tech.
    #[serde(rename = "@tech", default)]
    pub tech: String,
    /// Required status of the tech, for example `Active`.
    #[serde(rename = "@status", default)]
    pub status: String,
    /// Text content of the element.
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

/// A prerequisite requiring a count of a unit type.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TypeCountEntry {
    /// Unit type to count.
    #[serde(rename = "@unit", default)]
    pub unit: String,
    /// Compare operator for the count, from `@operator`.
    #[serde(rename = "@operator")]
    pub operator: Option<String>,
    /// Unit count to compare against.
    #[serde(rename = "@count")]
    pub count: Option<i32>,
}

/// Parse all techs from a `techs.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or a tech cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<Tech>> {
    parse_table(doc, "TechTree", "Tech")
}
