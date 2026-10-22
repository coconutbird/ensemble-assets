//! Parser for `.tactics.xmb` combat definitions.
//!
//! This mirrors the `BWeapon`, `BProtoAction`, and `BTactic` XML handlers,
//! including engine-supported fields unused by the retail data set.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::expect_root;

mod action;
mod weapon;

pub use action::{
    Action, ActionBeam, ActionDuration, AnimationRef, AutoRepair, DamageModifiers, DetonateThrow,
    EndAnimationRef, Fatality, JoinType, LowDetailAnimationRef, PhysicsDetonation, ProtoObjectRef,
    Shockwave, Strafing,
};
pub use weapon::{
    CameraEffect, CameraShake, DamageRatingOverride, Daze, DotEffect, DpsRamp, ImpactEffect,
    InterpolationTable, InterpolationValue, PhysicsExplosion, RadialBlurTable, RumbleEvent, Stasis,
    TargetPriority, Weapon, WeaponBeam,
};

/// A complete tactics definition from a `.tactics.xmb` file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TacticData {
    /// The weapon definitions (`Weapon`).
    #[serde(rename = "Weapon", default)]
    pub weapons: Vec<Weapon>,
    /// The tactic states (`State`).
    #[serde(rename = "State", default)]
    pub states: Vec<TacticState>,
    /// The action definitions (`Action`).
    #[serde(rename = "Action", default)]
    pub actions: Vec<Action>,
    /// The target rules and persistent actions (`Tactic`).
    #[serde(rename = "Tactic")]
    pub tactic: Option<TacticRules>,
}

/// A named tactic state.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TacticState {
    /// The state name (`Name`).
    #[serde(rename = "Name", default)]
    pub name: String,
    /// The idle animation (`IdleAnim`).
    #[serde(rename = "IdleAnim")]
    pub idle_anim: Option<String>,
    /// The walk animation (`WalkAnim`).
    #[serde(rename = "WalkAnim")]
    pub walk_anim: Option<String>,
    /// The jog animation (`JogAnim`).
    #[serde(rename = "JogAnim")]
    pub jog_anim: Option<String>,
    /// The run animation (`RunAnim`).
    #[serde(rename = "RunAnim")]
    pub run_anim: Option<String>,
    /// The death animation (`DeathAnim`).
    #[serde(rename = "DeathAnim")]
    pub death_anim: Option<String>,
    /// The names of the actions in the state (`Action`).
    #[serde(rename = "Action", default)]
    pub actions: Vec<String>,
}

/// The single `<Tactic>` rule block.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TacticRules {
    /// The target rules (`TargetRule`).
    #[serde(rename = "TargetRule", default)]
    pub target_rules: Vec<TargetRule>,
    /// The names of the persistent unit actions (`PersistentAction`).
    #[serde(rename = "PersistentAction", default)]
    pub persistent_actions: Vec<String>,
    /// The names of the persistent squad actions (`PersistentSquadAction`).
    #[serde(rename = "PersistentSquadAction", default)]
    pub persistent_squad_actions: Vec<String>,
}

/// One target-selection rule.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TargetRule {
    /// The player relation of the target (`Relation`).
    #[serde(rename = "Relation")]
    pub relation: Option<String>,
    /// The squad mode that the rule needs (`SquadMode`).
    #[serde(rename = "SquadMode")]
    pub squad_mode: Option<String>,
    /// The squad mode for automatic targeting (`AutoTargetSquadMode`).
    #[serde(rename = "AutoTargetSquadMode")]
    pub auto_target_squad_mode: Option<String>,
    /// Matches ground targets (`TargetsGround`).
    #[serde(
        rename = "TargetsGround",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub targets_ground: Option<bool>,
    /// Matches targets that contain units (`ContainsUnits`).
    #[serde(
        rename = "ContainsUnits",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub contains_units: Option<bool>,
    /// The damage types that the target must have (`DamageType`).
    #[serde(rename = "DamageType", default)]
    pub damage_types: Vec<String>,
    /// The target types that the rule matches (`TargetType`).
    #[serde(rename = "TargetType", default)]
    pub target_types: Vec<String>,
    /// The action name that the rule selects (`Action`).
    #[serde(rename = "Action")]
    pub action: Option<String>,
    /// The target states that the rule matches (`TargetState`).
    #[serde(rename = "TargetState", default)]
    pub target_states: Vec<String>,
    /// The ability that the rule needs (`Ability`).
    #[serde(rename = "Ability")]
    pub ability: Option<String>,
    /// The optional ability for the rule (`OptionalAbility`).
    #[serde(rename = "OptionalAbility")]
    pub optional_ability: Option<String>,
    /// Matches targets that Gaia owns (`GaiaOwned`).
    #[serde(
        rename = "GaiaOwned",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub gaia_owned: Option<bool>,
    /// Matches targets for a squad merge (`MergeSquads`).
    #[serde(
        rename = "MergeSquads",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub merge_squads: Option<bool>,
    /// Matches when the attacker is a melee unit (`MeleeAttacker`).
    #[serde(
        rename = "MeleeAttacker",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub melee_attacker: Option<bool>,
}

/// Parse tactics from a `.tactics.xmb` document.
///
/// # Errors
///
/// Returns an error if the root or tactics data is invalid.
pub fn parse(doc: &xmb::Document) -> crate::Result<TacticData> {
    let root = expect_root(doc, "TacticData")?;
    let data: TacticData = bdt_serde::from_node(root)?;
    Ok(data)
}

/// Serialize a [`TacticData`] back into an XMB [`Document`](xmb::Document).
///
/// # Errors
///
/// Returns an error if the tactics data cannot be serialized.
pub fn to_document(td: &TacticData) -> crate::Result<xmb::Document> {
    let node = bdt_serde::to_node("TacticData", td)?;
    Ok(xmb::Document::with_root(node))
}
