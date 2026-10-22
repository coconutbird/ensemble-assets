//! Weapon definitions from the `Weapon` elements of a tactics file.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// A weapon definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Weapon {
    /// The weapon name that actions refer to (`Name`).
    #[serde(rename = "Name", default)]
    pub name: String,
    /// The damage per second (`DamagePerSecond`).
    #[serde(rename = "DamagePerSecond")]
    pub damage_per_second: Option<f32>,
    /// The damage ramp over time (`DPSRamp`).
    #[serde(rename = "DPSRamp")]
    pub dps_ramp: Option<DpsRamp>,
    /// The damage-over-time rate (`DOTrate`).
    #[serde(rename = "DOTrate")]
    pub dot_rate: Option<f32>,
    /// The damage-over-time duration in seconds (`DOTduration`).
    #[serde(rename = "DOTduration")]
    pub dot_duration: Option<f32>,
    /// The damage-over-time effect (`DOTEffect`).
    #[serde(rename = "DOTEffect")]
    pub dot_effect: Option<DotEffect>,
    /// The reapply interval for the weapon effect (`Reapply`).
    #[serde(rename = "Reapply")]
    pub reapply: Option<f32>,
    /// The apply interval for the weapon effect (`Apply`).
    #[serde(rename = "Apply")]
    pub apply: Option<f32>,
    /// The minimum cooldown after an attack, in seconds (`PostAttackCooldownMin`).
    #[serde(rename = "PostAttackCooldownMin")]
    pub post_attack_cooldown_min: Option<f32>,
    /// The maximum cooldown after an attack, in seconds (`PostAttackCooldownMax`).
    #[serde(rename = "PostAttackCooldownMax")]
    pub post_attack_cooldown_max: Option<f32>,
    /// The minimum cooldown before an attack, in seconds (`PreAttackCooldownMin`).
    #[serde(rename = "PreAttackCooldownMin")]
    pub pre_attack_cooldown_min: Option<f32>,
    /// The maximum cooldown before an attack, in seconds (`PreAttackCooldownMax`).
    #[serde(rename = "PreAttackCooldownMax")]
    pub pre_attack_cooldown_max: Option<f32>,
    /// The attack rate (`AttackRate`).
    #[serde(rename = "AttackRate")]
    pub attack_rate: Option<f32>,
    /// The proto-object name of the projectile (`Projectile`).
    #[serde(rename = "Projectile")]
    pub projectile: Option<String>,
    /// The effect that plays on impact (`ImpactEffect`).
    #[serde(rename = "ImpactEffect")]
    pub impact_effect: Option<ImpactEffect>,
    /// The camera shake on impact (`ImpactCameraShake`).
    #[serde(rename = "ImpactCameraShake")]
    pub impact_camera_shake: Option<CameraShake>,
    /// The controller rumble on impact (`ImpactRumble`).
    #[serde(rename = "ImpactRumble")]
    pub impact_rumble: Option<RumbleEvent>,
    /// The inline camera effect on impact (`ImpactCameraEffect`).
    #[serde(rename = "ImpactCameraEffect")]
    pub impact_camera_effect: Option<CameraEffect>,
    /// The weapon type name from `weapontypes` (`WeaponType`).
    #[serde(rename = "WeaponType")]
    pub weapon_type: Option<String>,
    /// The number of visual ammo rounds (`VisualAmmo`).
    #[serde(rename = "VisualAmmo")]
    pub visual_ammo: Option<u32>,
    /// The area-of-effect radius (`AOERadius`).
    #[serde(rename = "AOERadius")]
    pub aoe_radius: Option<f32>,
    /// The area-of-effect damage factor for the primary target (`AOEPrimaryTargetFactor`).
    #[serde(rename = "AOEPrimaryTargetFactor")]
    pub aoe_primary_target_factor: Option<f32>,
    /// The area-of-effect distance factor (`AOEDistanceFactor`).
    #[serde(rename = "AOEDistanceFactor")]
    pub aoe_distance_factor: Option<f32>,
    /// The area-of-effect damage factor (`AOEDamageFactor`).
    #[serde(rename = "AOEDamageFactor")]
    pub aoe_damage_factor: Option<f32>,
    /// Makes area-of-effect damage fall off linearly with distance (`AOELinearDamage`).
    #[serde(
        rename = "AOELinearDamage",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub aoe_linear_damage: Option<bool>,
    /// The minimum physics launch angle (`PhysicsLaunchAngleMin`).
    #[serde(rename = "PhysicsLaunchAngleMin")]
    pub physics_launch_angle_min: Option<f32>,
    /// The maximum physics launch angle (`PhysicsLaunchAngleMax`).
    #[serde(rename = "PhysicsLaunchAngleMax")]
    pub physics_launch_angle_max: Option<f32>,
    /// Launches targets along the impact axis (`PhysicsLaunchAxial`).
    #[serde(
        rename = "PhysicsLaunchAxial",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub physics_launch_axial: Option<bool>,
    /// The minimum physics force on impact (`PhysicsForceMin`).
    #[serde(rename = "PhysicsForceMin")]
    pub physics_force_min: Option<f32>,
    /// The maximum physics force on impact (`PhysicsForceMax`).
    #[serde(rename = "PhysicsForceMax")]
    pub physics_force_max: Option<f32>,
    /// The maximum angle of the physics force (`PhysicsForceMaxAngle`).
    #[serde(rename = "PhysicsForceMaxAngle")]
    pub physics_force_max_angle: Option<f32>,
    /// Lets the weapon throw units (`ThrowUnits`).
    #[serde(
        rename = "ThrowUnits",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub throw_units: Option<bool>,
    /// Lets the weapon throw units that are still alive (`ThrowAliveUnits`).
    #[serde(
        rename = "ThrowAliveUnits",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub throw_alive_units: Option<bool>,
    /// Lets the weapon throw damage parts (`ThrowDamageParts`).
    #[serde(
        rename = "ThrowDamageParts",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub throw_damage_parts: Option<bool>,
    /// Makes thrown units flail (`FlailThrownUnits`).
    #[serde(
        rename = "FlailThrownUnits",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub flail_thrown_units: Option<bool>,
    /// Lets targets dodge the weapon (`Dodgeable`).
    #[serde(
        rename = "Dodgeable",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub dodgeable: Option<bool>,
    /// Lets targets deflect the weapon (`Deflectable`).
    #[serde(
        rename = "Deflectable",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub deflectable: Option<bool>,
    /// Lets small-arms deflection stop the weapon (`SmallArmsDeflectable`).
    #[serde(
        rename = "SmallArmsDeflectable",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub small_arms_deflectable: Option<bool>,
    /// Prevents the revive of units that the weapon kills (`OverridesRevive`).
    #[serde(
        rename = "OverridesRevive",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub overrides_revive: Option<bool>,
    /// Makes the weapon pull units (`PullUnits`).
    #[serde(
        rename = "PullUnits",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub pull_units: Option<bool>,
    /// Uses the damage per second as damage per attack (`UseDPSasDPA`).
    #[serde(
        rename = "UseDPSasDPA",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub use_dps_as_dpa: Option<bool>,
    /// Uses the range of the group (`UseGroupRange`).
    #[serde(
        rename = "UseGroupRange",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub use_group_range: Option<bool>,
    /// Shows the carried object as the projectile (`CarriedObjectAsProjectileVisual`).
    #[serde(
        rename = "CarriedObjectAsProjectileVisual",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub carried_object_as_projectile_visual: Option<bool>,
    /// The hardpoint name that fires the weapon (`Hardpoint`).
    #[serde(rename = "Hardpoint")]
    pub hardpoint: Option<String>,
    /// The alternate hardpoint name (`AlternateHardpoint`).
    #[serde(rename = "AlternateHardpoint")]
    pub alternate_hardpoint: Option<String>,
    /// The trigger script that the weapon runs (`TriggerScript`).
    #[serde(rename = "TriggerScript")]
    pub trigger_script: Option<String>,
    /// Lets the weapon damage friendly units (`AllowFriendlyFire`).
    #[serde(
        rename = "AllowFriendlyFire",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub allow_friendly_fire: Option<bool>,
    /// Adds bonus damage from a height advantage (`EnableHeightBonusDamage`).
    #[serde(
        rename = "EnableHeightBonusDamage",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub enable_height_bonus_damage: Option<bool>,
    /// The minimum range (`MinRange`).
    #[serde(rename = "MinRange")]
    pub min_range: Option<f32>,
    /// The maximum range (`MaxRange`).
    #[serde(rename = "MaxRange")]
    pub max_range: Option<f32>,
    /// Makes the weapon use ammo (`UsesAmmo`).
    #[serde(
        rename = "UsesAmmo",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub uses_ammo: Option<bool>,
    /// The beam settings (`Beam`).
    #[serde(rename = "Beam")]
    pub beam: Option<WeaponBeam>,
    /// The maximum damage of one ram (`MaxDamagePerRam`).
    #[serde(rename = "MaxDamagePerRam")]
    pub max_damage_per_ram: Option<f32>,
    /// The factor of damage that reflects back to the attacker (`ReflectDamageFactor`).
    #[serde(rename = "ReflectDamageFactor")]
    pub reflect_damage_factor: Option<f32>,
    /// The accuracy when stationary (`Accuracy`).
    #[serde(rename = "Accuracy")]
    pub accuracy: Option<f32>,
    /// The accuracy when moving (`MovingAccuracy`).
    #[serde(rename = "MovingAccuracy")]
    pub moving_accuracy: Option<f32>,
    /// The maximum shot deviation when stationary (`MaxDeviation`).
    #[serde(rename = "MaxDeviation")]
    pub max_deviation: Option<f32>,
    /// The maximum shot deviation when moving (`MovingMaxDeviation`).
    #[serde(rename = "MovingMaxDeviation")]
    pub moving_max_deviation: Option<f32>,
    /// The accuracy factor for distance (`AccuracyDistanceFactor`).
    #[serde(rename = "AccuracyDistanceFactor")]
    pub accuracy_distance_factor: Option<f32>,
    /// The accuracy factor for deviation (`AccuracyDeviationFactor`).
    #[serde(rename = "AccuracyDeviationFactor")]
    pub accuracy_deviation_factor: Option<f32>,
    /// The maximum velocity lead on moving targets (`MaxVelocityLead`).
    #[serde(rename = "MaxVelocityLead")]
    pub max_velocity_lead: Option<f32>,
    /// The span of an air burst (`AirBurstSpan`).
    #[serde(rename = "AirBurstSpan")]
    pub air_burst_span: Option<f32>,
    /// The damage rating overrides by damage type (`DamageRatingOverride`).
    #[serde(rename = "DamageRatingOverride", default)]
    pub damage_rating_overrides: Vec<DamageRatingOverride>,
    /// The target priorities by target type (`TargetPriority`).
    #[serde(rename = "TargetPriority", default)]
    pub target_priorities: Vec<TargetPriority>,
    /// Aims at the foot of the target unit (`TargetsFootOfUnit`).
    #[serde(
        rename = "TargetsFootOfUnit",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub targets_foot_of_unit: Option<bool>,
    /// Keeps the damage ramp between targets (`KeepDPSRamp`).
    #[serde(
        rename = "KeepDPSRamp",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub keep_dps_ramp: Option<bool>,
    /// The physics explosion on impact (`CausePhysicsExplosion`).
    #[serde(rename = "CausePhysicsExplosion")]
    pub cause_physics_explosion: Option<PhysicsExplosion>,
    /// The stasis settings (`Stasis`).
    #[serde(rename = "Stasis")]
    pub stasis: Option<Stasis>,
    /// Makes stasis drain the target (`StasisDrain`).
    #[serde(
        rename = "StasisDrain",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub stasis_drain: Option<bool>,
    /// Makes the weapon a stasis bomb (`StasisBomb`).
    #[serde(
        rename = "StasisBomb",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub stasis_bomb: Option<bool>,
    /// The ratio of heal to drain for stasis (`StasisHealToDrainRatio`).
    #[serde(rename = "StasisHealToDrainRatio")]
    pub stasis_heal_to_drain_ratio: Option<f32>,
    /// The offset angle for a throw (`ThrowOffsetAngle`).
    #[serde(rename = "ThrowOffsetAngle")]
    pub throw_offset_angle: Option<f32>,
    /// The velocity of a throw (`ThrowVelocity`).
    #[serde(rename = "ThrowVelocity")]
    pub throw_velocity: Option<f32>,
    /// Applies knockback to the target (`ApplyKnockback`).
    #[serde(
        rename = "ApplyKnockback",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub apply_knockback: Option<bool>,
    /// Marks the weapon as a tentacle attack (`Tentacle`).
    #[serde(
        rename = "Tentacle",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub tentacle: Option<bool>,
    /// The daze settings (`Daze`).
    #[serde(rename = "Daze")]
    pub daze: Option<Daze>,
    /// Ignores height in area-of-effect distance checks (`AOEIgnoresYAxis`).
    #[serde(
        rename = "AOEIgnoresYAxis",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub aoe_ignores_y_axis: Option<bool>,
    /// The number of projectile bounces (`Bounces`).
    #[serde(
        rename = "Bounces",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_i8",
        serialize_with = "crate::hw1::types::serialize_optional_i8"
    )]
    pub bounces: Option<i8>,
    /// The range of each bounce (`BounceRange`).
    #[serde(rename = "BounceRange")]
    pub bounce_range: Option<f32>,
    /// The misspelling is part of the game's XML contract.
    #[serde(rename = "CameraRubleShakeScalarNotLocal")]
    pub camera_ruble_shake_scalar_not_local: Option<f32>,
    /// Makes the projectile burst in the air (`AirBurst`).
    #[serde(
        rename = "AirBurst",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_bool"
    )]
    pub air_burst: Option<bool>,
    /// The maximum pull range (`MaxPullRange`).
    #[serde(rename = "MaxPullRange")]
    pub max_pull_range: Option<f32>,
}

/// A damage ramp that raises or lowers weapon damage over time (`DPSRamp`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DpsRamp {
    /// The start damage, as a percent of the full damage (`startDPSPercent`).
    #[serde(rename = "@startDPSPercent")]
    pub start_dps_percent: Option<f32>,
    /// The final damage, as a percent of the full damage (`finalDPSPercent`).
    #[serde(rename = "@finalDPSPercent")]
    pub final_dps_percent: Option<f32>,
    /// The ramp time in seconds (`rampTime`).
    #[serde(rename = "@rampTime")]
    pub ramp_time: Option<f32>,
    /// Makes the ramp drain shields (`shieldDrain`).
    #[serde(rename = "@shieldDrain")]
    pub shield_drain: Option<bool>,
}

/// The effect names for damage over time, by target size (`DOTEffect`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DotEffect {
    /// The effect for small targets (`small`).
    #[serde(rename = "@small")]
    pub small: Option<String>,
    /// The effect for medium targets (`medium`).
    #[serde(rename = "@medium")]
    pub medium: Option<String>,
    /// The effect for large targets (`large`).
    #[serde(rename = "@large")]
    pub large: Option<String>,
    /// The default effect name (element text).
    #[serde(rename = "$text", default)]
    pub default_effect: String,
}

/// The effect that plays where a weapon hits (`ImpactEffect`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ImpactEffect {
    /// The impact size class (`size`).
    #[serde(rename = "@size")]
    pub size: Option<String>,
    /// Starts the shockwave action on impact (`doShockwaveAction`).
    #[serde(rename = "@doShockwaveAction")]
    pub do_shockwave_action: Option<bool>,
    /// The impact effect name (element text).
    #[serde(rename = "$text", default)]
    pub name: String,
}

/// A camera shake that plays on impact (`ImpactCameraShake`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CameraShake {
    /// The shake duration in seconds (`Duration`).
    #[serde(rename = "@Duration")]
    pub duration: Option<f32>,
    /// The shake strength (`Strength`).
    #[serde(rename = "@Strength")]
    pub strength: Option<f32>,
}

/// A controller rumble that plays on impact (`ImpactRumble`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RumbleEvent {
    /// The rumble type of the left motor (`LeftRumbleType`).
    #[serde(rename = "@LeftRumbleType")]
    pub left_rumble_type: Option<String>,
    /// The rumble type of the right motor (`RightRumbleType`).
    #[serde(rename = "@RightRumbleType")]
    pub right_rumble_type: Option<String>,
    /// The rumble duration in seconds (`Duration`).
    #[serde(rename = "@Duration")]
    pub duration: Option<f32>,
    /// The strength of the left motor (`LeftStrength`).
    #[serde(rename = "@LeftStrength")]
    pub left_strength: Option<f32>,
    /// The strength of the right motor (`RightStrength`).
    #[serde(rename = "@RightStrength")]
    pub right_strength: Option<f32>,
}

/// A time-keyed interpolation table used by inline camera effects.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InterpolationTable<T> {
    /// The keys of the table (`Value`).
    #[serde(rename = "Value", default)]
    pub values: Vec<InterpolationValue<T>>,
}

/// One key of an interpolation table (`Value`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InterpolationValue<T> {
    /// The key time (`time`).
    #[serde(rename = "@time", default)]
    pub time: f32,
    /// The key value (element text).
    #[serde(rename = "$text", default)]
    pub value: T,
}

/// Inline `BCameraEffectData` accepted by `ImpactCameraEffect`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CameraEffect {
    /// The camera effect name (`Name`).
    #[serde(rename = "Name")]
    pub name: Option<String>,
    /// The color transform strength over time (`ColorTransformFactor`).
    #[serde(rename = "ColorTransformFactor")]
    pub color_transform_factor: Option<InterpolationTable<f32>>,
    /// The blur strength over time (`BlurFactor`).
    #[serde(rename = "BlurFactor")]
    pub blur_factor: Option<InterpolationTable<f32>>,
    /// The red row of the color transform over time (`ColorTransformR`).
    #[serde(rename = "ColorTransformR")]
    pub color_transform_r: Option<InterpolationTable<crate::hw1::Vector3>>,
    /// The green row of the color transform over time (`ColorTransformG`).
    #[serde(rename = "ColorTransformG")]
    pub color_transform_g: Option<InterpolationTable<crate::hw1::Vector3>>,
    /// The blue row of the color transform over time (`ColorTransformB`).
    #[serde(rename = "ColorTransformB")]
    pub color_transform_b: Option<InterpolationTable<crate::hw1::Vector3>>,
    /// The blur offset over time (`BlurXY`).
    #[serde(rename = "BlurXY")]
    pub blur_xy: Option<InterpolationTable<crate::hw1::Vector2>>,
    /// The radial blur over time (`RadialBlurFactor`).
    #[serde(rename = "RadialBlurFactor")]
    pub radial_blur_factor: Option<RadialBlurTable>,
    /// The field of view over time (`FOV`).
    #[serde(rename = "FOV")]
    pub fov: Option<InterpolationTable<f32>>,
    /// The camera zoom over time (`Zoom`).
    #[serde(rename = "Zoom")]
    pub zoom: Option<InterpolationTable<f32>>,
    /// The camera yaw over time (`Yaw`).
    #[serde(rename = "Yaw")]
    pub yaw: Option<InterpolationTable<f32>>,
    /// The camera pitch over time (`Pitch`).
    #[serde(rename = "Pitch")]
    pub pitch: Option<InterpolationTable<f32>>,
}

/// A radial blur table with position options (`RadialBlurFactor`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RadialBlurTable {
    /// Centers the blur on a 3D position (`use3DPos`).
    #[serde(rename = "@use3DPos")]
    pub use_3d_pos: Option<bool>,
    /// Uses the hover point as the 3D position (`useHoverPointAs3DPos`).
    #[serde(rename = "@useHoverPointAs3DPos")]
    pub use_hover_point_as_3d_pos: Option<bool>,
    /// The keys of the table (`Value`).
    #[serde(rename = "Value", default)]
    pub values: Vec<InterpolationValue<crate::hw1::Vector2>>,
}

/// The tracking settings of a beam weapon (`Beam`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct WeaponBeam {
    /// The beam activation speed (`activationSpeed`).
    #[serde(rename = "@activationSpeed")]
    pub activation_speed: Option<f32>,
    /// The beam tracking speed (`trackingSpeed`).
    #[serde(rename = "@trackingSpeed")]
    pub tracking_speed: Option<f32>,
    /// The beam turn rate (`turnRate`).
    #[serde(rename = "@turnRate")]
    pub turn_rate: Option<f32>,
    /// The beam wait time (`waitTime`).
    #[serde(rename = "@waitTime")]
    pub wait_time: Option<f32>,
    /// The beam search radius (`searchRadius`).
    #[serde(rename = "@searchRadius")]
    pub search_radius: Option<f32>,
}

/// A damage rating that replaces the computed rating for one damage type (`DamageRatingOverride`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DamageRatingOverride {
    /// The damage type name (`type`).
    #[serde(rename = "@type", default)]
    pub damage_type: String,
    /// The half-kill cutoff factor (`halfKillCutoffFactor`).
    #[serde(rename = "@halfKillCutoffFactor")]
    pub half_kill_cutoff_factor: Option<f32>,
    /// The damage rating (element text).
    #[serde(rename = "$text", default)]
    pub damage_rating: f32,
}

/// A target priority for one target type (`TargetPriority`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TargetPriority {
    /// The target type name (`type`).
    #[serde(rename = "@type", default)]
    pub target_type: String,
    /// The priority value (element text).
    #[serde(rename = "$text", default)]
    pub priority: f32,
}

/// A physics explosion that the weapon causes on impact (`CausePhysicsExplosion`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PhysicsExplosion {
    /// The particle effect name (`particle`).
    #[serde(rename = "@particle")]
    pub particle: Option<String>,
    /// The victim type name (`victimType`).
    #[serde(rename = "@victimType")]
    pub victim_type: Option<String>,
}

/// The stasis settings of a weapon (`Stasis`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Stasis {
    /// Selects stasis targets with smart targeting (`SmartTargeting`).
    #[serde(rename = "@SmartTargeting")]
    pub smart_targeting: Option<bool>,
}

/// The daze settings of a weapon (`Daze`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Daze {
    /// The target type that the daze affects (`TargetType`).
    #[serde(rename = "@TargetType")]
    pub target_type: Option<String>,
    /// The movement speed multiplier while dazed (`MovementModifier`).
    #[serde(rename = "@MovementModifier")]
    pub movement_modifier: Option<f32>,
    /// Applies the daze in the area of effect (`AOEDaze`).
    #[serde(rename = "@AOEDaze")]
    pub aoe_daze: Option<bool>,
    /// Selects daze targets with smart targeting (`SmartTargeting`).
    #[serde(rename = "@SmartTargeting")]
    pub smart_targeting: Option<bool>,
    /// The daze duration in seconds (element text).
    #[serde(rename = "$text", default)]
    pub duration: f32,
}
