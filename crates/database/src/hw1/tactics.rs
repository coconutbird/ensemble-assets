//! Parser for `.tactics.xmb` combat definitions.
//!
//! This mirrors the `BWeapon`, `BProtoAction`, and `BTactic` XML handlers,
//! including engine-supported fields unused by the retail data set.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::expect_root;

/// A complete tactics definition from a `.tactics.xmb` file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TacticData {
    #[serde(rename = "Weapon", default)]
    pub weapons: Vec<Weapon>,
    #[serde(rename = "State", default)]
    pub states: Vec<TacticState>,
    #[serde(rename = "Action", default)]
    pub actions: Vec<Action>,
    #[serde(rename = "Tactic")]
    pub tactic: Option<TacticRules>,
}

/// A weapon definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Weapon {
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "DamagePerSecond")]
    pub damage_per_second: Option<f32>,
    #[serde(rename = "DPSRamp")]
    pub dps_ramp: Option<DpsRamp>,
    #[serde(rename = "DOTrate")]
    pub dot_rate: Option<f32>,
    #[serde(rename = "DOTduration")]
    pub dot_duration: Option<f32>,
    #[serde(rename = "DOTEffect")]
    pub dot_effect: Option<DotEffect>,
    #[serde(rename = "Reapply")]
    pub reapply: Option<f32>,
    #[serde(rename = "Apply")]
    pub apply: Option<f32>,
    #[serde(rename = "PostAttackCooldownMin")]
    pub post_attack_cooldown_min: Option<f32>,
    #[serde(rename = "PostAttackCooldownMax")]
    pub post_attack_cooldown_max: Option<f32>,
    #[serde(rename = "PreAttackCooldownMin")]
    pub pre_attack_cooldown_min: Option<f32>,
    #[serde(rename = "PreAttackCooldownMax")]
    pub pre_attack_cooldown_max: Option<f32>,
    #[serde(rename = "AttackRate")]
    pub attack_rate: Option<f32>,
    #[serde(rename = "Projectile")]
    pub projectile: Option<String>,
    #[serde(rename = "ImpactEffect")]
    pub impact_effect: Option<ImpactEffect>,
    #[serde(rename = "ImpactCameraShake")]
    pub impact_camera_shake: Option<CameraShake>,
    #[serde(rename = "ImpactRumble")]
    pub impact_rumble: Option<RumbleEvent>,
    #[serde(rename = "ImpactCameraEffect")]
    pub impact_camera_effect: Option<CameraEffect>,
    #[serde(rename = "WeaponType")]
    pub weapon_type: Option<String>,
    #[serde(rename = "VisualAmmo")]
    pub visual_ammo: Option<u32>,
    #[serde(rename = "AOERadius")]
    pub aoe_radius: Option<f32>,
    #[serde(rename = "AOEPrimaryTargetFactor")]
    pub aoe_primary_target_factor: Option<f32>,
    #[serde(rename = "AOEDistanceFactor")]
    pub aoe_distance_factor: Option<f32>,
    #[serde(rename = "AOEDamageFactor")]
    pub aoe_damage_factor: Option<f32>,
    #[serde(
        rename = "AOELinearDamage",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub aoe_linear_damage: Option<bool>,
    #[serde(rename = "PhysicsLaunchAngleMin")]
    pub physics_launch_angle_min: Option<f32>,
    #[serde(rename = "PhysicsLaunchAngleMax")]
    pub physics_launch_angle_max: Option<f32>,
    #[serde(
        rename = "PhysicsLaunchAxial",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub physics_launch_axial: Option<bool>,
    #[serde(rename = "PhysicsForceMin")]
    pub physics_force_min: Option<f32>,
    #[serde(rename = "PhysicsForceMax")]
    pub physics_force_max: Option<f32>,
    #[serde(rename = "PhysicsForceMaxAngle")]
    pub physics_force_max_angle: Option<f32>,
    #[serde(
        rename = "ThrowUnits",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub throw_units: Option<bool>,
    #[serde(
        rename = "ThrowAliveUnits",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub throw_alive_units: Option<bool>,
    #[serde(
        rename = "ThrowDamageParts",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub throw_damage_parts: Option<bool>,
    #[serde(
        rename = "FlailThrownUnits",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub flail_thrown_units: Option<bool>,
    #[serde(
        rename = "Dodgeable",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub dodgeable: Option<bool>,
    #[serde(
        rename = "Deflectable",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub deflectable: Option<bool>,
    #[serde(
        rename = "SmallArmsDeflectable",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub small_arms_deflectable: Option<bool>,
    #[serde(
        rename = "OverridesRevive",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub overrides_revive: Option<bool>,
    #[serde(
        rename = "PullUnits",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub pull_units: Option<bool>,
    #[serde(
        rename = "UseDPSasDPA",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub use_dps_as_dpa: Option<bool>,
    #[serde(
        rename = "UseGroupRange",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub use_group_range: Option<bool>,
    #[serde(
        rename = "CarriedObjectAsProjectileVisual",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub carried_object_as_projectile_visual: Option<bool>,
    #[serde(rename = "Hardpoint")]
    pub hardpoint: Option<String>,
    #[serde(rename = "AlternateHardpoint")]
    pub alternate_hardpoint: Option<String>,
    #[serde(rename = "TriggerScript")]
    pub trigger_script: Option<String>,
    #[serde(
        rename = "AllowFriendlyFire",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub allow_friendly_fire: Option<bool>,
    #[serde(
        rename = "EnableHeightBonusDamage",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub enable_height_bonus_damage: Option<bool>,
    #[serde(rename = "MinRange")]
    pub min_range: Option<f32>,
    #[serde(rename = "MaxRange")]
    pub max_range: Option<f32>,
    #[serde(
        rename = "UsesAmmo",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub uses_ammo: Option<bool>,
    #[serde(rename = "Beam")]
    pub beam: Option<WeaponBeam>,
    #[serde(rename = "MaxDamagePerRam")]
    pub max_damage_per_ram: Option<f32>,
    #[serde(rename = "ReflectDamageFactor")]
    pub reflect_damage_factor: Option<f32>,
    #[serde(rename = "Accuracy")]
    pub accuracy: Option<f32>,
    #[serde(rename = "MovingAccuracy")]
    pub moving_accuracy: Option<f32>,
    #[serde(rename = "MaxDeviation")]
    pub max_deviation: Option<f32>,
    #[serde(rename = "MovingMaxDeviation")]
    pub moving_max_deviation: Option<f32>,
    #[serde(rename = "AccuracyDistanceFactor")]
    pub accuracy_distance_factor: Option<f32>,
    #[serde(rename = "AccuracyDeviationFactor")]
    pub accuracy_deviation_factor: Option<f32>,
    #[serde(rename = "MaxVelocityLead")]
    pub max_velocity_lead: Option<f32>,
    #[serde(rename = "AirBurstSpan")]
    pub air_burst_span: Option<f32>,
    #[serde(rename = "DamageRatingOverride", default)]
    pub damage_rating_overrides: Vec<DamageRatingOverride>,
    #[serde(rename = "TargetPriority", default)]
    pub target_priorities: Vec<TargetPriority>,
    #[serde(
        rename = "TargetsFootOfUnit",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub targets_foot_of_unit: Option<bool>,
    #[serde(
        rename = "KeepDPSRamp",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub keep_dps_ramp: Option<bool>,
    #[serde(rename = "CausePhysicsExplosion")]
    pub cause_physics_explosion: Option<PhysicsExplosion>,
    #[serde(rename = "Stasis")]
    pub stasis: Option<Stasis>,
    #[serde(
        rename = "StasisDrain",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub stasis_drain: Option<bool>,
    #[serde(
        rename = "StasisBomb",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub stasis_bomb: Option<bool>,
    #[serde(rename = "StasisHealToDrainRatio")]
    pub stasis_heal_to_drain_ratio: Option<f32>,
    #[serde(rename = "ThrowOffsetAngle")]
    pub throw_offset_angle: Option<f32>,
    #[serde(rename = "ThrowVelocity")]
    pub throw_velocity: Option<f32>,
    #[serde(
        rename = "ApplyKnockback",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub apply_knockback: Option<bool>,
    #[serde(
        rename = "Tentacle",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub tentacle: Option<bool>,
    #[serde(rename = "Daze")]
    pub daze: Option<Daze>,
    #[serde(
        rename = "AOEIgnoresYAxis",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub aoe_ignores_y_axis: Option<bool>,
    #[serde(
        rename = "Bounces",
        default,
        deserialize_with = "super::types::deserialize_optional_i8",
        serialize_with = "super::types::serialize_optional_i8"
    )]
    pub bounces: Option<i8>,
    #[serde(rename = "BounceRange")]
    pub bounce_range: Option<f32>,
    /// The misspelling is part of the game's XML contract.
    #[serde(rename = "CameraRubleShakeScalarNotLocal")]
    pub camera_ruble_shake_scalar_not_local: Option<f32>,
    #[serde(
        rename = "AirBurst",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub air_burst: Option<bool>,
    #[serde(rename = "MaxPullRange")]
    pub max_pull_range: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DpsRamp {
    #[serde(rename = "@startDPSPercent")]
    pub start_dps_percent: Option<f32>,
    #[serde(rename = "@finalDPSPercent")]
    pub final_dps_percent: Option<f32>,
    #[serde(rename = "@rampTime")]
    pub ramp_time: Option<f32>,
    #[serde(rename = "@shieldDrain")]
    pub shield_drain: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DotEffect {
    #[serde(rename = "@small")]
    pub small: Option<String>,
    #[serde(rename = "@medium")]
    pub medium: Option<String>,
    #[serde(rename = "@large")]
    pub large: Option<String>,
    #[serde(rename = "$text", default)]
    pub default_effect: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ImpactEffect {
    #[serde(rename = "@size")]
    pub size: Option<String>,
    #[serde(rename = "@doShockwaveAction")]
    pub do_shockwave_action: Option<bool>,
    #[serde(rename = "$text", default)]
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CameraShake {
    #[serde(rename = "@Duration")]
    pub duration: Option<f32>,
    #[serde(rename = "@Strength")]
    pub strength: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RumbleEvent {
    #[serde(rename = "@LeftRumbleType")]
    pub left_rumble_type: Option<String>,
    #[serde(rename = "@RightRumbleType")]
    pub right_rumble_type: Option<String>,
    #[serde(rename = "@Duration")]
    pub duration: Option<f32>,
    #[serde(rename = "@LeftStrength")]
    pub left_strength: Option<f32>,
    #[serde(rename = "@RightStrength")]
    pub right_strength: Option<f32>,
}

/// A time-keyed interpolation table used by inline camera effects.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InterpolationTable<T> {
    #[serde(rename = "Value", default)]
    pub values: Vec<InterpolationValue<T>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InterpolationValue<T> {
    #[serde(rename = "@time", default)]
    pub time: f32,
    #[serde(rename = "$text")]
    pub value: T,
}

/// Inline `BCameraEffectData` accepted by `ImpactCameraEffect`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CameraEffect {
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "ColorTransformFactor")]
    pub color_transform_factor: Option<InterpolationTable<f32>>,
    #[serde(rename = "BlurFactor")]
    pub blur_factor: Option<InterpolationTable<f32>>,
    #[serde(rename = "ColorTransformR")]
    pub color_transform_r: Option<InterpolationTable<super::Vector3>>,
    #[serde(rename = "ColorTransformG")]
    pub color_transform_g: Option<InterpolationTable<super::Vector3>>,
    #[serde(rename = "ColorTransformB")]
    pub color_transform_b: Option<InterpolationTable<super::Vector3>>,
    #[serde(rename = "BlurXY")]
    pub blur_xy: Option<InterpolationTable<super::Vector2>>,
    #[serde(rename = "RadialBlurFactor")]
    pub radial_blur_factor: Option<RadialBlurTable>,
    #[serde(rename = "FOV")]
    pub fov: Option<InterpolationTable<f32>>,
    #[serde(rename = "Zoom")]
    pub zoom: Option<InterpolationTable<f32>>,
    #[serde(rename = "Yaw")]
    pub yaw: Option<InterpolationTable<f32>>,
    #[serde(rename = "Pitch")]
    pub pitch: Option<InterpolationTable<f32>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RadialBlurTable {
    #[serde(rename = "@use3DPos")]
    pub use_3d_pos: Option<bool>,
    #[serde(rename = "@useHoverPointAs3DPos")]
    pub use_hover_point_as_3d_pos: Option<bool>,
    #[serde(rename = "Value", default)]
    pub values: Vec<InterpolationValue<super::Vector2>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct WeaponBeam {
    #[serde(rename = "@activationSpeed")]
    pub activation_speed: Option<f32>,
    #[serde(rename = "@trackingSpeed")]
    pub tracking_speed: Option<f32>,
    #[serde(rename = "@turnRate")]
    pub turn_rate: Option<f32>,
    #[serde(rename = "@waitTime")]
    pub wait_time: Option<f32>,
    #[serde(rename = "@searchRadius")]
    pub search_radius: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DamageRatingOverride {
    #[serde(rename = "@type", default)]
    pub damage_type: String,
    #[serde(rename = "@halfKillCutoffFactor")]
    pub half_kill_cutoff_factor: Option<f32>,
    #[serde(rename = "$text", default)]
    pub damage_rating: f32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TargetPriority {
    #[serde(rename = "@type", default)]
    pub target_type: String,
    #[serde(rename = "$text", default)]
    pub priority: f32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PhysicsExplosion {
    #[serde(rename = "@particle")]
    pub particle: Option<String>,
    #[serde(rename = "@victimType")]
    pub victim_type: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Stasis {
    #[serde(rename = "@SmartTargeting")]
    pub smart_targeting: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Daze {
    #[serde(rename = "@TargetType")]
    pub target_type: Option<String>,
    #[serde(rename = "@MovementModifier")]
    pub movement_modifier: Option<f32>,
    #[serde(rename = "@AOEDaze")]
    pub aoe_daze: Option<bool>,
    #[serde(rename = "@SmartTargeting")]
    pub smart_targeting: Option<bool>,
    #[serde(rename = "$text", default)]
    pub duration: f32,
}

/// An action a unit can perform.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Action {
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "ActionType")]
    pub action_type: Option<String>,
    #[serde(rename = "ProjectileSpread")]
    pub projectile_spread: Option<f32>,
    #[serde(
        rename = "StopAttackingWhenAmmoDepleted",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub stop_attacking_when_ammo_depleted: Option<bool>,
    #[serde(
        rename = "DontLoopAttackAnim",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub dont_loop_attack_anim: Option<bool>,
    #[serde(
        rename = "DontAutoRestart",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub dont_auto_restart: Option<bool>,
    #[serde(
        rename = "DontCheckOrientTolerance",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub dont_check_orient_tolerance: Option<bool>,
    #[serde(
        rename = "KillSelfOnAttack",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub kill_self_on_attack: Option<bool>,
    #[serde(
        rename = "MeleeRange",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub melee_range: Option<bool>,
    #[serde(
        rename = "InstantAttack",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub instant_attack: Option<bool>,
    #[serde(
        rename = "Infection",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub infection: Option<bool>,
    #[serde(
        rename = "CanOrientOwner",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub can_orient_owner: Option<bool>,
    #[serde(
        rename = "MainAttack",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub main_attack: Option<bool>,
    #[serde(
        rename = "Stationary",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub stationary: Option<bool>,
    #[serde(rename = "SquadType")]
    pub squad_type: Option<String>,
    #[serde(rename = "Weapon")]
    pub weapon: Option<String>,
    #[serde(rename = "LinkedAction")]
    pub linked_action: Option<String>,
    #[serde(rename = "Shockwave")]
    pub shockwave: Option<Shockwave>,
    #[serde(rename = "SquadMode")]
    pub squad_mode: Option<String>,
    #[serde(rename = "NewSquadMode")]
    pub new_squad_mode: Option<String>,
    #[serde(rename = "NewTacticState")]
    pub new_tactic_state: Option<String>,
    #[serde(rename = "Anim")]
    pub anim: Option<AnimationRef>,
    #[serde(rename = "AnimLowDetail")]
    pub anim_low_detail: Option<LowDetailAnimationRef>,
    #[serde(rename = "SupportAnim")]
    pub support_anim: Option<String>,
    #[serde(rename = "ReloadAnim")]
    pub reload_anim: Option<String>,
    #[serde(rename = "WorkRate")]
    pub work_rate: Option<f32>,
    #[serde(rename = "WorkRateVariance")]
    pub work_rate_variance: Option<f32>,
    #[serde(rename = "WorkRange")]
    pub work_range: Option<f32>,
    #[serde(rename = "DodgeChanceMax")]
    pub dodge_chance_max: Option<f32>,
    #[serde(rename = "DodgeChanceMin")]
    pub dodge_chance_min: Option<f32>,
    #[serde(rename = "DodgeMaxAngle")]
    pub dodge_max_angle: Option<f32>,
    #[serde(rename = "DodgeCooldown")]
    pub dodge_cooldown: Option<f32>,
    #[serde(rename = "DodgePhysicsImpulse")]
    pub dodge_physics_impulse: Option<f32>,
    #[serde(
        rename = "WaitForDeflectCooldown",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub wait_for_deflect_cooldown: Option<bool>,
    #[serde(
        rename = "MultiDeflect",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub multi_deflect: Option<bool>,
    #[serde(
        rename = "SmallArms",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub small_arms: Option<bool>,
    #[serde(
        rename = "DoShakeOnAttackTag",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub do_shake_on_attack_tag: Option<bool>,
    #[serde(
        rename = "HideSpawnUntilRelease",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub hide_spawn_until_release: Option<bool>,
    #[serde(
        rename = "AvoidOnly",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub avoid_only: Option<bool>,
    #[serde(rename = "DeflectChanceMax")]
    pub deflect_chance_max: Option<f32>,
    #[serde(rename = "DeflectChanceMin")]
    pub deflect_chance_min: Option<f32>,
    #[serde(rename = "DeflectMaxAngle")]
    pub deflect_max_angle: Option<f32>,
    #[serde(rename = "DeflectCooldown")]
    pub deflect_cooldown: Option<f32>,
    #[serde(rename = "DeflectMaxDmg")]
    pub deflect_max_damage: Option<f32>,
    #[serde(rename = "DeflectTimeout")]
    pub deflect_timeout: Option<f32>,
    #[serde(
        rename = "WaitForDodgeCooldown",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub wait_for_dodge_cooldown: Option<bool>,
    #[serde(rename = "Strafing")]
    pub strafing: Option<Strafing>,
    #[serde(rename = "HoverAltitudeOffset")]
    pub hover_altitude_offset: Option<f32>,
    #[serde(rename = "MaxTgtDepressionAngle")]
    pub max_target_depression_angle: Option<f32>,
    #[serde(rename = "MaxPitch")]
    pub max_pitch: Option<f32>,
    #[serde(rename = "MaxRoll")]
    pub max_roll: Option<f32>,
    #[serde(rename = "DamageModifiers")]
    pub damage_modifiers: Option<DamageModifiers>,
    #[serde(rename = "JoinType")]
    pub join_type: Option<JoinType>,
    #[serde(rename = "ParticleEffect")]
    pub particle_effect: Option<String>,
    #[serde(rename = "MergeType")]
    pub merge_type: Option<String>,
    #[serde(rename = "StartAnim")]
    pub start_anim: Option<AnimationRef>,
    #[serde(rename = "EndAnim")]
    pub end_anim: Option<EndAnimationRef>,
    #[serde(rename = "MinIdleDuration")]
    pub min_idle_duration: Option<f32>,
    #[serde(
        rename = "ClearTacticState",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub clear_tactic_state: Option<bool>,
    #[serde(rename = "Beam")]
    pub beam: Option<ActionBeam>,
    #[serde(
        rename = "TargetAir",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub target_air: Option<bool>,
    #[serde(rename = "Resource")]
    pub resource: Option<String>,
    #[serde(
        rename = "Default",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub default: Option<bool>,
    #[serde(
        rename = "NoAutoTarget",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub no_auto_target: Option<bool>,
    #[serde(
        rename = "DieOnBuilt",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub die_on_built: Option<bool>,
    #[serde(
        rename = "TeamShare",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub team_share: Option<bool>,
    #[serde(rename = "ExposedAction")]
    pub exposed_action: Option<i32>,
    #[serde(
        rename = "TrampleAttack",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub trample_attack: Option<bool>,
    #[serde(rename = "SlaveAttackAction")]
    pub slave_attack_action: Option<String>,
    #[serde(rename = "BaseDPSWeapon")]
    pub base_dps_weapon: Option<String>,
    #[serde(
        rename = "DisableAttackWaitTimer",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub disable_attack_wait_timer: Option<bool>,
    #[serde(
        rename = "FindBetterAction",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub find_better_action: Option<bool>,
    #[serde(
        rename = "PersistBetweenOpps",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub persist_between_opportunities: Option<bool>,
    #[serde(
        rename = "AllowReinforce",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub allow_reinforce: Option<bool>,
    #[serde(
        rename = "Overrun",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub overrun: Option<bool>,
    #[serde(rename = "PersistentActionType")]
    pub persistent_action_type: Option<String>,
    #[serde(rename = "Fatality", default)]
    pub fatalities: Vec<Fatality>,
    #[serde(
        rename = "StartDisabled",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub start_disabled: Option<bool>,
    #[serde(
        rename = "RequiresLockdown",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub requires_lockdown: Option<bool>,
    #[serde(rename = "Duration")]
    pub duration: Option<ActionDuration>,
    #[serde(rename = "DetonateThrow")]
    pub detonate_throw: Option<DetonateThrow>,
    #[serde(rename = "DetonateFromPhysics")]
    pub detonate_from_physics: Option<PhysicsDetonation>,
    #[serde(
        rename = "DetonateWhenInRange",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub detonate_when_in_range: Option<bool>,
    #[serde(rename = "AutoRepair")]
    pub auto_repair: Option<AutoRepair>,
    #[serde(rename = "InvalidTarget", default)]
    pub invalid_targets: Vec<String>,
    #[serde(
        rename = "HealTarget",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub heal_target: Option<bool>,
    #[serde(
        rename = "TargetOfTarget",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub target_of_target: Option<bool>,
    #[serde(
        rename = "UseTeleporter",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub use_teleporter: Option<bool>,
    #[serde(rename = "VelocityScalar")]
    pub velocity_scalar: Option<f32>,
    #[serde(
        rename = "DetonateOnDeath",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub detonate_on_death: Option<bool>,
    #[serde(
        rename = "PickupObject",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub pickup_object: Option<bool>,
    #[serde(
        rename = "PickupObjectOnKill",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub pickup_object_on_kill: Option<bool>,
    #[serde(rename = "AttackRunDelayMin")]
    pub attack_run_delay_min: Option<f32>,
    #[serde(rename = "AttackRunDelayMax")]
    pub attack_run_delay_max: Option<f32>,
    #[serde(
        rename = "AlertWhenComplete",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub alert_when_complete: Option<bool>,
    #[serde(
        rename = "AutoJoin",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub auto_join: Option<bool>,
    #[serde(rename = "ReviveDelay")]
    pub revive_delay: Option<f32>,
    #[serde(rename = "HibernateReviveDelay")]
    pub hibernate_revive_delay: Option<f32>,
    #[serde(rename = "ReviveRate")]
    pub revive_rate: Option<f32>,
    #[serde(rename = "ProtoObject")]
    pub proto_object: Option<ProtoObjectRef>,
    #[serde(rename = "Count")]
    pub count: Option<i32>,
    #[serde(rename = "MaxNumUnitsPerformAction")]
    pub max_num_units_perform_action: Option<i32>,
    #[serde(rename = "DamageCharge")]
    pub damage_charge: Option<f32>,
    #[serde(
        rename = "ChargeOnTaken",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub charge_on_taken: Option<bool>,
    #[serde(
        rename = "ChargeOnDealt",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub charge_on_dealt: Option<bool>,
    #[serde(
        rename = "Chargable",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub chargable: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Shockwave {
    #[serde(rename = "@duration")]
    pub duration: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AnimationRef {
    #[serde(rename = "@noInterrupt")]
    pub no_interrupt: Option<bool>,
    #[serde(rename = "$text", default)]
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LowDetailAnimationRef {
    #[serde(rename = "@actionCountThreshold")]
    pub action_count_threshold: Option<i32>,
    #[serde(rename = "$text", default)]
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Strafing {
    #[serde(rename = "@maxDistance")]
    pub max_distance: Option<f32>,
    #[serde(rename = "@turnRate")]
    pub turn_rate: Option<f32>,
    #[serde(rename = "@trackingSpeed")]
    pub tracking_speed: Option<f32>,
    #[serde(rename = "@jitter")]
    pub jitter: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DamageModifiers {
    #[serde(rename = "@damage")]
    pub damage: Option<f32>,
    #[serde(rename = "@damageTaken")]
    pub damage_taken: Option<f32>,
    #[serde(rename = "@byCombatValue")]
    pub by_combat_value: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct JoinType {
    #[serde(rename = "@revertDamagePct")]
    pub revert_damage_pct: Option<f32>,
    #[serde(rename = "@veterancyOverride")]
    pub veterancy_override: Option<bool>,
    #[serde(rename = "@boardTime")]
    pub board_time: Option<f32>,
    #[serde(rename = "@boardAnim")]
    pub board_anim: Option<String>,
    #[serde(rename = "@unjoinMaxDist")]
    pub unjoin_max_dist: Option<f32>,
    #[serde(rename = "@levels")]
    pub levels: Option<i32>,
    #[serde(rename = "$text", default)]
    pub kind: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EndAnimationRef {
    #[serde(rename = "@noInterrupt")]
    pub no_interrupt: Option<bool>,
    #[serde(rename = "@SoundCue")]
    pub sound_cue: Option<String>,
    #[serde(rename = "$text", default)]
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ActionBeam {
    #[serde(rename = "@collideWithUnits")]
    pub collide_with_units: Option<bool>,
    #[serde(rename = "@collideWithTerrain")]
    pub collide_with_terrain: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Fatality {
    #[serde(rename = "@attacker")]
    pub attacker: Option<String>,
    #[serde(rename = "@target")]
    pub target_animation: Option<String>,
    #[serde(rename = "@cooldown")]
    pub cooldown: Option<f32>,
    #[serde(rename = "@offsetTolerance")]
    pub offset_tolerance: Option<f32>,
    #[serde(rename = "@orientationOffset")]
    pub orientation_offset: Option<f32>,
    #[serde(rename = "@toBoneRelative")]
    pub to_bone_relative: Option<bool>,
    #[serde(rename = "@transitionBeforeAnimating")]
    pub transition_before_animating: Option<bool>,
    #[serde(rename = "@air")]
    pub air: Option<bool>,
    #[serde(rename = "@airHeight")]
    pub air_height: Option<f32>,
    #[serde(rename = "@airApexPercent")]
    pub air_apex_percent: Option<f32>,
    #[serde(rename = "@transitionDelay")]
    pub transition_delay: Option<f32>,
    #[serde(rename = "@transitionTime")]
    pub transition_time: Option<f32>,
    #[serde(rename = "@noMove")]
    pub no_move: Option<bool>,
    #[serde(rename = "@lastUnitOnly")]
    pub last_unit_only: Option<bool>,
    #[serde(rename = "@boarding")]
    pub boarding: Option<bool>,
    #[serde(rename = "@attackerEnd")]
    pub attacker_end: Option<String>,
    #[serde(rename = "@targetEnd")]
    pub target_end: Option<String>,
    #[serde(rename = "$text", default)]
    pub target_proto_object: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ActionDuration {
    #[serde(rename = "@DurationSpread")]
    pub spread: Option<f32>,
    #[serde(rename = "$text", default)]
    pub seconds: f32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DetonateThrow {
    #[serde(rename = "@HorizontalMax")]
    pub horizontal_max: Option<f32>,
    #[serde(rename = "@VerticalMax")]
    pub vertical_max: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PhysicsDetonation {
    #[serde(rename = "@PhysicsDetonationThreshold")]
    pub threshold: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AutoRepair {
    #[serde(rename = "@AutoRepairIdleTime")]
    pub idle_time: Option<u32>,
    #[serde(rename = "@AutoRepairThreshold")]
    pub threshold: Option<f32>,
    #[serde(rename = "@AutoRepairSearchDistance")]
    pub search_distance: Option<f32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProtoObjectRef {
    /// Presence of this attribute selects the proto-squad namespace.
    #[serde(rename = "@Squad")]
    pub squad: Option<String>,
    #[serde(rename = "@bone")]
    pub bone: Option<String>,
    #[serde(rename = "$text", default)]
    pub name: String,
}

/// A named tactic state.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TacticState {
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "IdleAnim")]
    pub idle_anim: Option<String>,
    #[serde(rename = "WalkAnim")]
    pub walk_anim: Option<String>,
    #[serde(rename = "JogAnim")]
    pub jog_anim: Option<String>,
    #[serde(rename = "RunAnim")]
    pub run_anim: Option<String>,
    #[serde(rename = "DeathAnim")]
    pub death_anim: Option<String>,
    #[serde(rename = "Action", default)]
    pub actions: Vec<String>,
}

/// The single `<Tactic>` rule block.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TacticRules {
    #[serde(rename = "TargetRule", default)]
    pub target_rules: Vec<TargetRule>,
    #[serde(rename = "PersistentAction", default)]
    pub persistent_actions: Vec<String>,
    #[serde(rename = "PersistentSquadAction", default)]
    pub persistent_squad_actions: Vec<String>,
}

/// One target-selection rule.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TargetRule {
    #[serde(rename = "Relation")]
    pub relation: Option<String>,
    #[serde(rename = "SquadMode")]
    pub squad_mode: Option<String>,
    #[serde(rename = "AutoTargetSquadMode")]
    pub auto_target_squad_mode: Option<String>,
    #[serde(
        rename = "TargetsGround",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub targets_ground: Option<bool>,
    #[serde(
        rename = "ContainsUnits",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub contains_units: Option<bool>,
    #[serde(rename = "DamageType", default)]
    pub damage_types: Vec<String>,
    #[serde(rename = "TargetType", default)]
    pub target_types: Vec<String>,
    #[serde(rename = "Action")]
    pub action: Option<String>,
    #[serde(rename = "TargetState", default)]
    pub target_states: Vec<String>,
    #[serde(rename = "Ability")]
    pub ability: Option<String>,
    #[serde(rename = "OptionalAbility")]
    pub optional_ability: Option<String>,
    #[serde(
        rename = "GaiaOwned",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub gaia_owned: Option<bool>,
    #[serde(
        rename = "MergeSquads",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub merge_squads: Option<bool>,
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
