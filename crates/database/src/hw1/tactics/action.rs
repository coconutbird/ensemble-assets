//! Action definitions from the `Action` elements of a tactics file.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// An action a unit can perform.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Action {
    /// The action name that states and rules refer to (`Name`).
    #[serde(rename = "Name", default)]
    pub name: String,
    /// The action type, for example `RangedAttack` (`ActionType`).
    #[serde(rename = "ActionType")]
    pub action_type: Option<String>,
    /// The projectile spread (`ProjectileSpread`).
    #[serde(rename = "ProjectileSpread")]
    pub projectile_spread: Option<f32>,
    /// Stops the attack when the ammo is empty (`StopAttackingWhenAmmoDepleted`).
    #[serde(
        rename = "StopAttackingWhenAmmoDepleted",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub stop_attacking_when_ammo_depleted: Option<bool>,
    /// Plays the attack animation one time only (`DontLoopAttackAnim`).
    #[serde(
        rename = "DontLoopAttackAnim",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub dont_loop_attack_anim: Option<bool>,
    /// Prevents an automatic restart of the action (`DontAutoRestart`).
    #[serde(
        rename = "DontAutoRestart",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub dont_auto_restart: Option<bool>,
    /// Skips the orientation tolerance check (`DontCheckOrientTolerance`).
    #[serde(
        rename = "DontCheckOrientTolerance",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub dont_check_orient_tolerance: Option<bool>,
    /// Kills the unit when it attacks (`KillSelfOnAttack`).
    #[serde(
        rename = "KillSelfOnAttack",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub kill_self_on_attack: Option<bool>,
    /// Uses the melee range (`MeleeRange`).
    #[serde(
        rename = "MeleeRange",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub melee_range: Option<bool>,
    /// Makes the attack instant (`InstantAttack`).
    #[serde(
        rename = "InstantAttack",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub instant_attack: Option<bool>,
    /// Makes the attack infect the target (`Infection`).
    #[serde(
        rename = "Infection",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub infection: Option<bool>,
    /// Lets the action turn the owner (`CanOrientOwner`).
    #[serde(
        rename = "CanOrientOwner",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub can_orient_owner: Option<bool>,
    /// Marks the action as the main attack (`MainAttack`).
    #[serde(
        rename = "MainAttack",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub main_attack: Option<bool>,
    /// Requires the unit to stand still (`Stationary`).
    #[serde(
        rename = "Stationary",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub stationary: Option<bool>,
    /// The squad type for the action (`SquadType`).
    #[serde(rename = "SquadType")]
    pub squad_type: Option<String>,
    /// The weapon name that the action uses (`Weapon`).
    #[serde(rename = "Weapon")]
    pub weapon: Option<String>,
    /// The name of a linked action (`LinkedAction`).
    #[serde(rename = "LinkedAction")]
    pub linked_action: Option<String>,
    /// The shockwave settings (`Shockwave`).
    #[serde(rename = "Shockwave")]
    pub shockwave: Option<Shockwave>,
    /// The squad mode for the action (`SquadMode`).
    #[serde(rename = "SquadMode")]
    pub squad_mode: Option<String>,
    /// The squad mode to set when the action starts (`NewSquadMode`).
    #[serde(rename = "NewSquadMode")]
    pub new_squad_mode: Option<String>,
    /// The tactic state to set when the action starts (`NewTacticState`).
    #[serde(rename = "NewTacticState")]
    pub new_tactic_state: Option<String>,
    /// The main animation (`Anim`).
    #[serde(rename = "Anim")]
    pub anim: Option<AnimationRef>,
    /// The low-detail animation (`AnimLowDetail`).
    #[serde(rename = "AnimLowDetail")]
    pub anim_low_detail: Option<LowDetailAnimationRef>,
    /// The support animation (`SupportAnim`).
    #[serde(rename = "SupportAnim")]
    pub support_anim: Option<String>,
    /// The reload animation (`ReloadAnim`).
    #[serde(rename = "ReloadAnim")]
    pub reload_anim: Option<String>,
    /// The work rate (`WorkRate`).
    #[serde(rename = "WorkRate")]
    pub work_rate: Option<f32>,
    /// The random variance of the work rate (`WorkRateVariance`).
    #[serde(rename = "WorkRateVariance")]
    pub work_rate_variance: Option<f32>,
    /// The work range (`WorkRange`).
    #[serde(rename = "WorkRange")]
    pub work_range: Option<f32>,
    /// The maximum dodge chance (`DodgeChanceMax`).
    #[serde(rename = "DodgeChanceMax")]
    pub dodge_chance_max: Option<f32>,
    /// The minimum dodge chance (`DodgeChanceMin`).
    #[serde(rename = "DodgeChanceMin")]
    pub dodge_chance_min: Option<f32>,
    /// The maximum dodge angle (`DodgeMaxAngle`).
    #[serde(rename = "DodgeMaxAngle")]
    pub dodge_max_angle: Option<f32>,
    /// The dodge cooldown in seconds (`DodgeCooldown`).
    #[serde(rename = "DodgeCooldown")]
    pub dodge_cooldown: Option<f32>,
    /// The physics impulse of a dodge (`DodgePhysicsImpulse`).
    #[serde(rename = "DodgePhysicsImpulse")]
    pub dodge_physics_impulse: Option<f32>,
    /// Waits for the deflect cooldown (`WaitForDeflectCooldown`).
    #[serde(
        rename = "WaitForDeflectCooldown",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub wait_for_deflect_cooldown: Option<bool>,
    /// Lets the unit deflect more than one attack (`MultiDeflect`).
    #[serde(
        rename = "MultiDeflect",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub multi_deflect: Option<bool>,
    /// Marks the action as small-arms fire (`SmallArms`).
    #[serde(
        rename = "SmallArms",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub small_arms: Option<bool>,
    /// Shakes the camera on the attack tag (`DoShakeOnAttackTag`).
    #[serde(
        rename = "DoShakeOnAttackTag",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub do_shake_on_attack_tag: Option<bool>,
    /// Hides the spawned object until release (`HideSpawnUntilRelease`).
    #[serde(
        rename = "HideSpawnUntilRelease",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub hide_spawn_until_release: Option<bool>,
    /// Makes the action only avoid attacks (`AvoidOnly`).
    #[serde(
        rename = "AvoidOnly",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub avoid_only: Option<bool>,
    /// The maximum deflect chance (`DeflectChanceMax`).
    #[serde(rename = "DeflectChanceMax")]
    pub deflect_chance_max: Option<f32>,
    /// The minimum deflect chance (`DeflectChanceMin`).
    #[serde(rename = "DeflectChanceMin")]
    pub deflect_chance_min: Option<f32>,
    /// The maximum deflect angle (`DeflectMaxAngle`).
    #[serde(rename = "DeflectMaxAngle")]
    pub deflect_max_angle: Option<f32>,
    /// The deflect cooldown in seconds (`DeflectCooldown`).
    #[serde(rename = "DeflectCooldown")]
    pub deflect_cooldown: Option<f32>,
    /// The maximum damage that the unit can deflect (`DeflectMaxDmg`).
    #[serde(rename = "DeflectMaxDmg")]
    pub deflect_max_damage: Option<f32>,
    /// The deflect timeout in seconds (`DeflectTimeout`).
    #[serde(rename = "DeflectTimeout")]
    pub deflect_timeout: Option<f32>,
    /// Waits for the dodge cooldown (`WaitForDodgeCooldown`).
    #[serde(
        rename = "WaitForDodgeCooldown",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub wait_for_dodge_cooldown: Option<bool>,
    /// The strafing settings (`Strafing`).
    #[serde(rename = "Strafing")]
    pub strafing: Option<Strafing>,
    /// The hover altitude offset (`HoverAltitudeOffset`).
    #[serde(rename = "HoverAltitudeOffset")]
    pub hover_altitude_offset: Option<f32>,
    /// The maximum depression angle to the target (`MaxTgtDepressionAngle`).
    #[serde(rename = "MaxTgtDepressionAngle")]
    pub max_target_depression_angle: Option<f32>,
    /// The maximum pitch (`MaxPitch`).
    #[serde(rename = "MaxPitch")]
    pub max_pitch: Option<f32>,
    /// The maximum roll (`MaxRoll`).
    #[serde(rename = "MaxRoll")]
    pub max_roll: Option<f32>,
    /// The damage modifiers (`DamageModifiers`).
    #[serde(rename = "DamageModifiers")]
    pub damage_modifiers: Option<DamageModifiers>,
    /// The join settings (`JoinType`).
    #[serde(rename = "JoinType")]
    pub join_type: Option<JoinType>,
    /// The particle effect name (`ParticleEffect`).
    #[serde(rename = "ParticleEffect")]
    pub particle_effect: Option<String>,
    /// The merge type (`MergeType`).
    #[serde(rename = "MergeType")]
    pub merge_type: Option<String>,
    /// The start animation (`StartAnim`).
    #[serde(rename = "StartAnim")]
    pub start_anim: Option<AnimationRef>,
    /// The end animation (`EndAnim`).
    #[serde(rename = "EndAnim")]
    pub end_anim: Option<EndAnimationRef>,
    /// The minimum idle duration in seconds (`MinIdleDuration`).
    #[serde(rename = "MinIdleDuration")]
    pub min_idle_duration: Option<f32>,
    /// Clears the tactic state when the action ends (`ClearTacticState`).
    #[serde(
        rename = "ClearTacticState",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub clear_tactic_state: Option<bool>,
    /// The beam collision settings (`Beam`).
    #[serde(rename = "Beam")]
    pub beam: Option<ActionBeam>,
    /// Lets the action target air units (`TargetAir`).
    #[serde(
        rename = "TargetAir",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub target_air: Option<bool>,
    /// The resource name for the action (`Resource`).
    #[serde(rename = "Resource")]
    pub resource: Option<String>,
    /// Marks the action as the default action (`Default`).
    #[serde(
        rename = "Default",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub default: Option<bool>,
    /// Prevents automatic targeting (`NoAutoTarget`).
    #[serde(
        rename = "NoAutoTarget",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub no_auto_target: Option<bool>,
    /// Kills the unit when the build completes (`DieOnBuilt`).
    #[serde(
        rename = "DieOnBuilt",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub die_on_built: Option<bool>,
    /// Shares the result with the team (`TeamShare`).
    #[serde(
        rename = "TeamShare",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub team_share: Option<bool>,
    /// The index of the exposed action (`ExposedAction`).
    #[serde(rename = "ExposedAction")]
    pub exposed_action: Option<i32>,
    /// Marks the action as a trample attack (`TrampleAttack`).
    #[serde(
        rename = "TrampleAttack",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub trample_attack: Option<bool>,
    /// The name of the slave attack action (`SlaveAttackAction`).
    #[serde(rename = "SlaveAttackAction")]
    pub slave_attack_action: Option<String>,
    /// The weapon name for the base damage per second (`BaseDPSWeapon`).
    #[serde(rename = "BaseDPSWeapon")]
    pub base_dps_weapon: Option<String>,
    /// Disables the attack wait timer (`DisableAttackWaitTimer`).
    #[serde(
        rename = "DisableAttackWaitTimer",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub disable_attack_wait_timer: Option<bool>,
    /// Looks for a better action while the action runs (`FindBetterAction`).
    #[serde(
        rename = "FindBetterAction",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub find_better_action: Option<bool>,
    /// Keeps the action between opportunities (`PersistBetweenOpps`).
    #[serde(
        rename = "PersistBetweenOpps",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub persist_between_opportunities: Option<bool>,
    /// Allows reinforcement during the action (`AllowReinforce`).
    #[serde(
        rename = "AllowReinforce",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub allow_reinforce: Option<bool>,
    /// Marks the action as an overrun (`Overrun`).
    #[serde(
        rename = "Overrun",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub overrun: Option<bool>,
    /// The persistent action type (`PersistentActionType`).
    #[serde(rename = "PersistentActionType")]
    pub persistent_action_type: Option<String>,
    /// The fatality animations (`Fatality`).
    #[serde(rename = "Fatality", default)]
    pub fatalities: Vec<Fatality>,
    /// Disables the action at the start (`StartDisabled`).
    #[serde(
        rename = "StartDisabled",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub start_disabled: Option<bool>,
    /// Requires the unit to be in lockdown (`RequiresLockdown`).
    #[serde(
        rename = "RequiresLockdown",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub requires_lockdown: Option<bool>,
    /// The action duration (`Duration`).
    #[serde(rename = "Duration")]
    pub duration: Option<ActionDuration>,
    /// The throw limits for a detonation (`DetonateThrow`).
    #[serde(rename = "DetonateThrow")]
    pub detonate_throw: Option<DetonateThrow>,
    /// The physics detonation settings (`DetonateFromPhysics`).
    #[serde(rename = "DetonateFromPhysics")]
    pub detonate_from_physics: Option<PhysicsDetonation>,
    /// Detonates the unit when the target is in range (`DetonateWhenInRange`).
    #[serde(
        rename = "DetonateWhenInRange",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub detonate_when_in_range: Option<bool>,
    /// The automatic repair settings (`AutoRepair`).
    #[serde(rename = "AutoRepair")]
    pub auto_repair: Option<AutoRepair>,
    /// The names of targets that the action cannot use (`InvalidTarget`).
    #[serde(rename = "InvalidTarget", default)]
    pub invalid_targets: Vec<String>,
    /// Makes the action heal the target (`HealTarget`).
    #[serde(
        rename = "HealTarget",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub heal_target: Option<bool>,
    /// Uses the target of the target (`TargetOfTarget`).
    #[serde(
        rename = "TargetOfTarget",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub target_of_target: Option<bool>,
    /// Makes the action use a teleporter (`UseTeleporter`).
    #[serde(
        rename = "UseTeleporter",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub use_teleporter: Option<bool>,
    /// The velocity multiplier (`VelocityScalar`).
    #[serde(rename = "VelocityScalar")]
    pub velocity_scalar: Option<f32>,
    /// Detonates the unit when it dies (`DetonateOnDeath`).
    #[serde(
        rename = "DetonateOnDeath",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub detonate_on_death: Option<bool>,
    /// Makes the unit pick up an object (`PickupObject`).
    #[serde(
        rename = "PickupObject",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub pickup_object: Option<bool>,
    /// Makes the unit pick up the object that it kills (`PickupObjectOnKill`).
    #[serde(
        rename = "PickupObjectOnKill",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub pickup_object_on_kill: Option<bool>,
    /// The minimum delay between attack runs in seconds (`AttackRunDelayMin`).
    #[serde(rename = "AttackRunDelayMin")]
    pub attack_run_delay_min: Option<f32>,
    /// The maximum delay between attack runs in seconds (`AttackRunDelayMax`).
    #[serde(rename = "AttackRunDelayMax")]
    pub attack_run_delay_max: Option<f32>,
    /// Alerts the player when the action completes (`AlertWhenComplete`).
    #[serde(
        rename = "AlertWhenComplete",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub alert_when_complete: Option<bool>,
    /// Joins the unit automatically (`AutoJoin`).
    #[serde(
        rename = "AutoJoin",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub auto_join: Option<bool>,
    /// The revive delay in seconds (`ReviveDelay`).
    #[serde(rename = "ReviveDelay")]
    pub revive_delay: Option<f32>,
    /// The revive delay in seconds during hibernation (`HibernateReviveDelay`).
    #[serde(rename = "HibernateReviveDelay")]
    pub hibernate_revive_delay: Option<f32>,
    /// The revive rate (`ReviveRate`).
    #[serde(rename = "ReviveRate")]
    pub revive_rate: Option<f32>,
    /// The proto-object that the action uses (`ProtoObject`).
    #[serde(rename = "ProtoObject")]
    pub proto_object: Option<ProtoObjectRef>,
    /// The count of objects for the action (`Count`).
    #[serde(rename = "Count")]
    pub count: Option<i32>,
    /// The maximum number of units that can do the action (`MaxNumUnitsPerformAction`).
    #[serde(rename = "MaxNumUnitsPerformAction")]
    pub max_num_units_perform_action: Option<i32>,
    /// The damage that charges the action (`DamageCharge`).
    #[serde(rename = "DamageCharge")]
    pub damage_charge: Option<f32>,
    /// Charges the action from damage taken (`ChargeOnTaken`).
    #[serde(
        rename = "ChargeOnTaken",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub charge_on_taken: Option<bool>,
    /// Charges the action from damage dealt (`ChargeOnDealt`).
    #[serde(
        rename = "ChargeOnDealt",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub charge_on_dealt: Option<bool>,
    /// Makes the action chargeable (`Chargable`).
    #[serde(
        rename = "Chargable",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub chargable: Option<bool>,
}

/// A shockwave that the action causes (`Shockwave`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Shockwave {
    /// The shockwave duration in seconds (`duration`).
    #[serde(rename = "@duration")]
    pub duration: Option<f32>,
}

/// A reference to an animation, with an interrupt flag.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AnimationRef {
    /// Prevents an interrupt of the animation (`noInterrupt`).
    #[serde(rename = "@noInterrupt")]
    pub no_interrupt: Option<bool>,
    /// The animation name (element text).
    #[serde(rename = "$text", default)]
    pub name: String,
}

/// A low-detail animation that replaces the main animation when many units do the action (`AnimLowDetail`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LowDetailAnimationRef {
    /// The action count at which the low-detail animation starts (`actionCountThreshold`).
    #[serde(rename = "@actionCountThreshold")]
    pub action_count_threshold: Option<i32>,
    /// The animation name (element text).
    #[serde(rename = "$text", default)]
    pub name: String,
}

/// The strafing settings of an action (`Strafing`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Strafing {
    /// The maximum strafe distance (`maxDistance`).
    #[serde(rename = "@maxDistance")]
    pub max_distance: Option<f32>,
    /// The strafe turn rate (`turnRate`).
    #[serde(rename = "@turnRate")]
    pub turn_rate: Option<f32>,
    /// The strafe tracking speed (`trackingSpeed`).
    #[serde(rename = "@trackingSpeed")]
    pub tracking_speed: Option<f32>,
    /// The strafe jitter (`jitter`).
    #[serde(rename = "@jitter")]
    pub jitter: Option<f32>,
}

/// The damage multipliers that an action applies (`DamageModifiers`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DamageModifiers {
    /// The multiplier for damage dealt (`damage`).
    #[serde(rename = "@damage")]
    pub damage: Option<f32>,
    /// The multiplier for damage taken (`damageTaken`).
    #[serde(rename = "@damageTaken")]
    pub damage_taken: Option<f32>,
    /// Scales the modifiers by combat value (`byCombatValue`).
    #[serde(rename = "@byCombatValue")]
    pub by_combat_value: Option<bool>,
}

/// The join settings of a join action (`JoinType`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct JoinType {
    /// The damage percent at which the join reverts (`revertDamagePct`).
    #[serde(rename = "@revertDamagePct")]
    pub revert_damage_pct: Option<f32>,
    /// Overrides the veterancy of the joined unit (`veterancyOverride`).
    #[serde(rename = "@veterancyOverride")]
    pub veterancy_override: Option<bool>,
    /// The board time in seconds (`boardTime`).
    #[serde(rename = "@boardTime")]
    pub board_time: Option<f32>,
    /// The board animation (`boardAnim`).
    #[serde(rename = "@boardAnim")]
    pub board_anim: Option<String>,
    /// The maximum distance for an unjoin (`unjoinMaxDist`).
    #[serde(rename = "@unjoinMaxDist")]
    pub unjoin_max_dist: Option<f32>,
    /// The number of join levels (`levels`).
    #[serde(rename = "@levels")]
    pub levels: Option<i32>,
    /// The join type name (element text).
    #[serde(rename = "$text", default)]
    pub kind: String,
}

/// A reference to the end animation of an action, with a sound cue (`EndAnim`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EndAnimationRef {
    /// Prevents an interrupt of the animation (`noInterrupt`).
    #[serde(rename = "@noInterrupt")]
    pub no_interrupt: Option<bool>,
    /// The sound cue that plays with the animation (`SoundCue`).
    #[serde(rename = "@SoundCue")]
    pub sound_cue: Option<String>,
    /// The animation name (element text).
    #[serde(rename = "$text", default)]
    pub name: String,
}

/// The collision settings of a beam action (`Beam`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ActionBeam {
    /// Makes the beam collide with units (`collideWithUnits`).
    #[serde(rename = "@collideWithUnits")]
    pub collide_with_units: Option<bool>,
    /// Makes the beam collide with terrain (`collideWithTerrain`).
    #[serde(rename = "@collideWithTerrain")]
    pub collide_with_terrain: Option<bool>,
}

/// A paired fatality animation between an attacker and a target (`Fatality`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Fatality {
    /// The attacker animation (`attacker`).
    #[serde(rename = "@attacker")]
    pub attacker: Option<String>,
    /// The target animation (`target`).
    #[serde(rename = "@target")]
    pub target_animation: Option<String>,
    /// The fatality cooldown in seconds (`cooldown`).
    #[serde(rename = "@cooldown")]
    pub cooldown: Option<f32>,
    /// The position offset tolerance (`offsetTolerance`).
    #[serde(rename = "@offsetTolerance")]
    pub offset_tolerance: Option<f32>,
    /// The orientation offset (`orientationOffset`).
    #[serde(rename = "@orientationOffset")]
    pub orientation_offset: Option<f32>,
    /// Positions the target relative to a bone (`toBoneRelative`).
    #[serde(rename = "@toBoneRelative")]
    pub to_bone_relative: Option<bool>,
    /// Moves the units into place before the animation (`transitionBeforeAnimating`).
    #[serde(rename = "@transitionBeforeAnimating")]
    pub transition_before_animating: Option<bool>,
    /// Marks the fatality as an air fatality (`air`).
    #[serde(rename = "@air")]
    pub air: Option<bool>,
    /// The height of an air fatality (`airHeight`).
    #[serde(rename = "@airHeight")]
    pub air_height: Option<f32>,
    /// The apex point of an air fatality, as a percent (`airApexPercent`).
    #[serde(rename = "@airApexPercent")]
    pub air_apex_percent: Option<f32>,
    /// The delay before the transition in seconds (`transitionDelay`).
    #[serde(rename = "@transitionDelay")]
    pub transition_delay: Option<f32>,
    /// The transition time in seconds (`transitionTime`).
    #[serde(rename = "@transitionTime")]
    pub transition_time: Option<f32>,
    /// Prevents the units from moving into place (`noMove`).
    #[serde(rename = "@noMove")]
    pub no_move: Option<bool>,
    /// Allows the fatality only on the last unit of a squad (`lastUnitOnly`).
    #[serde(rename = "@lastUnitOnly")]
    pub last_unit_only: Option<bool>,
    /// Marks the fatality as a boarding fatality (`boarding`).
    #[serde(rename = "@boarding")]
    pub boarding: Option<bool>,
    /// The end animation of the attacker (`attackerEnd`).
    #[serde(rename = "@attackerEnd")]
    pub attacker_end: Option<String>,
    /// The end animation of the target (`targetEnd`).
    #[serde(rename = "@targetEnd")]
    pub target_end: Option<String>,
    /// The proto-object name of the target (element text).
    #[serde(rename = "$text", default)]
    pub target_proto_object: String,
}

/// The duration of an action, with a random spread (`Duration`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ActionDuration {
    /// The random spread of the duration (`DurationSpread`).
    #[serde(rename = "@DurationSpread")]
    pub spread: Option<f32>,
    /// The duration in seconds (element text).
    #[serde(rename = "$text", default)]
    pub seconds: f32,
}

/// The throw limits for a detonation (`DetonateThrow`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DetonateThrow {
    /// The maximum horizontal throw (`HorizontalMax`).
    #[serde(rename = "@HorizontalMax")]
    pub horizontal_max: Option<f32>,
    /// The maximum vertical throw (`VerticalMax`).
    #[serde(rename = "@VerticalMax")]
    pub vertical_max: Option<f32>,
}

/// The physics impact that detonates the unit (`DetonateFromPhysics`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PhysicsDetonation {
    /// The physics impact threshold for detonation (`PhysicsDetonationThreshold`).
    #[serde(rename = "@PhysicsDetonationThreshold")]
    pub threshold: Option<f32>,
}

/// The automatic repair settings of an action (`AutoRepair`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AutoRepair {
    /// The idle time before the repair starts (`AutoRepairIdleTime`).
    #[serde(rename = "@AutoRepairIdleTime")]
    pub idle_time: Option<u32>,
    /// The health threshold for a repair (`AutoRepairThreshold`).
    #[serde(rename = "@AutoRepairThreshold")]
    pub threshold: Option<f32>,
    /// The search distance for a repair (`AutoRepairSearchDistance`).
    #[serde(rename = "@AutoRepairSearchDistance")]
    pub search_distance: Option<f32>,
}

/// A reference to a proto-object or proto-squad, with an optional bone (`ProtoObject`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProtoObjectRef {
    /// Presence of this attribute selects the proto-squad namespace.
    #[serde(rename = "@Squad")]
    pub squad: Option<String>,
    /// The bone name for the object (`bone`).
    #[serde(rename = "@bone")]
    pub bone: Option<String>,
    /// The proto-object name (element text).
    #[serde(rename = "$text", default)]
    pub name: String,
}
