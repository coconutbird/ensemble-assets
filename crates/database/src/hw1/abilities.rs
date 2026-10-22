//! Parser for `abilities.xml.xmb` — ability definitions.
//!
//! Each `<Ability>` element describes a unit ability (lockdown, ram, barrage, etc.).

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::parse_table;

/// A single ability definition from `abilities.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Ability {
    /// Ability name (unique key), e.g. `"UnscLockdown"`.
    #[serde(rename = "@Name", default)]
    pub name: String,
    /// Display name string ID.
    #[serde(rename = "DisplayNameID")]
    pub display_name_id: Option<i32>,
    /// Secondary display name string ID.
    #[serde(rename = "DisplayName2ID")]
    pub display_name_2_id: Option<i32>,
    /// Rollover text string ID.
    #[serde(rename = "RolloverTextID")]
    pub rollover_text_id: Option<i32>,
    /// Icon path used by the ability UI.
    #[serde(rename = "Icon")]
    pub icon: Option<String>,
    /// Ability type: `"Work"`, `"ChangeMode"`, `"Unload"`, `"CommandMenu"`, etc.
    #[serde(rename = "Type")]
    pub ability_type: Option<String>,
    /// Squad mode to enter, e.g. `"Lockdown"`, `"HitAndRun"`.
    #[serde(rename = "SquadMode")]
    pub squad_mode: Option<String>,
    /// Whether to keep the squad mode after ability ends.
    #[serde(
        rename = "KeepSquadMode",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub keep_squad_mode: Option<bool>,
    /// Target type: `"Unit"`, `"Location"`, `"UnitOrLocation"`.
    #[serde(rename = "TargetType")]
    pub target_type: Option<String>,
    /// Proto-object filters accepted by the ability. The loader appends every
    /// repeated `<Object>` entry.
    #[serde(rename = "Object", default)]
    pub objects: Vec<String>,
    /// Recovery start trigger: `"Attack"`.
    #[serde(rename = "RecoverStart")]
    pub recover_start: Option<String>,
    /// Recovery type: `"Ability"`.
    #[serde(rename = "RecoverType")]
    pub recover_type: Option<String>,
    /// Recovery time in seconds.
    #[serde(rename = "RecoverTime")]
    pub recover_time: Option<f32>,
    /// Duration in seconds.
    #[serde(rename = "Duration")]
    pub duration: Option<f32>,
    /// Ammunition consumed when the ability is used.
    #[serde(rename = "AmmoCost")]
    pub ammo_cost: Option<f32>,
    /// Whether attacking switches the squad into its configured attack mode.
    #[serde(
        rename = "AttackSquadMode",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub attack_squad_mode: Option<bool>,
    /// Movement speed modifier while active.
    #[serde(rename = "MovementSpeedModifier")]
    pub movement_speed_modifier: Option<f32>,
    /// Movement modifier type: `"Mode"`.
    #[serde(rename = "MovementModifierType")]
    pub movement_modifier_type: Option<String>,
    /// Whether this ability can be used in hetero-command groups.
    #[serde(
        rename = "CanHeteroCommand",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub can_hetero_command: Option<bool>,
    /// Whether to suppress the ability reticle.
    #[serde(
        rename = "NoAbilityReticle",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub no_ability_reticle: Option<bool>,
    /// Whether to avoid interrupting the current attack.
    #[serde(
        rename = "DontInterruptAttack",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub dont_interrupt_attack: Option<bool>,
    /// Whether the ability uses sprinting movement.
    #[serde(
        rename = "Sprinting",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub sprinting: Option<bool>,
    /// Damage taken modifier while ability is active.
    #[serde(rename = "DamageTakenModifier")]
    pub damage_taken_modifier: Option<f32>,
    /// Dodge chance modifier while ability is active.
    #[serde(rename = "DodgeModifier")]
    pub dodge_modifier: Option<f32>,
    /// Recovery animation attachment point.
    #[serde(rename = "RecoverAnimAttachment")]
    pub recover_anim_attachment: Option<String>,
    /// Recovery end animation.
    #[serde(rename = "RecoverEndAnim")]
    pub recover_end_anim: Option<String>,
    /// Recovery start animation.
    #[serde(rename = "RecoverStartAnim")]
    pub recover_start_anim: Option<String>,
    /// Smart target range.
    #[serde(rename = "SmartTargetRange")]
    pub smart_target_range: Option<f32>,
}

/// Parse all abilities from an `abilities.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or an ability cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<Ability>> {
    parse_table(doc, "Abilities", "Ability")
}
