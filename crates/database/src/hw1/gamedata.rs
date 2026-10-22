//! Parser for `gamedata.xml.xmb` — global game constants and settings.
//!
//! Contains resource definitions, population types, difficulty settings,
//! code proto-object mappings, and various gameplay constants.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use super::RgbColor;
use crate::node_ext::expect_root;

/// Global game data from `gamedata.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GameData {
    /// Resource definitions wrapper.
    #[serde(rename = "Resources")]
    pub resources: Option<ResourcesWrapper>,
    /// Rate definitions wrapper.
    #[serde(rename = "Rates")]
    pub rates: Option<RatesWrapper>,
    /// Population type names wrapper.
    #[serde(rename = "Pops")]
    pub pops: Option<PopsWrapper>,
    /// Ref count names wrapper.
    #[serde(rename = "RefCounts")]
    pub ref_counts: Option<RefCountsWrapper>,
    /// HUD item names wrapper.
    #[serde(rename = "HUDItems")]
    pub hud_items: Option<HUDItemsWrapper>,
    /// Flashable item names wrapper.
    #[serde(rename = "FlashableItems")]
    pub flashable_items: Option<FlashableItemsWrapper>,
    /// Unit flag names wrapper.
    #[serde(rename = "UnitFlags")]
    pub unit_flags: Option<UnitFlagsWrapper>,
    /// Squad flag names wrapper.
    #[serde(rename = "SquadFlags")]
    pub squad_flags: Option<SquadFlagsWrapper>,
    /// Player state names wrapper.
    #[serde(rename = "PlayerStates")]
    pub player_states: Option<PlayerStatesWrapper>,
    /// Code proto-object mappings wrapper.
    #[serde(rename = "CodeProtoObjects")]
    pub code_proto_objects: Option<CodeProtoObjectsWrapper>,
    /// Code object type mappings wrapper.
    #[serde(rename = "CodeObjectTypes")]
    pub code_object_types: Option<CodeObjectTypesWrapper>,

    // Difficulty settings
    /// Difficulty value for Easy (`DifficultyEasy`).
    #[serde(rename = "DifficultyEasy")]
    pub difficulty_easy: Option<f32>,
    /// Difficulty value for Normal (`DifficultyNormal`).
    #[serde(rename = "DifficultyNormal", alias = "DifficultyNormali")]
    pub difficulty_normal: Option<f32>,
    /// Difficulty value for Hard (`DifficultyHard`).
    #[serde(rename = "DifficultyHard")]
    pub difficulty_hard: Option<f32>,
    /// Difficulty value for Legendary (`DifficultyLegendary`).
    #[serde(rename = "DifficultyLegendary")]
    pub difficulty_legendary: Option<f32>,
    /// Default difficulty value (`DifficultyDefault`).
    #[serde(rename = "DifficultyDefault")]
    pub difficulty_default: Option<f32>,

    // Supply pad settings
    /// Supply bonus of a UNSC supply pad (`UnscSupplyPadBonus`).
    #[serde(rename = "UnscSupplyPadBonus")]
    pub unsc_supply_pad_bonus: Option<f32>,
    /// Break-even point of a UNSC supply pad (`UnscSupplyPadBreakEvenPoint`).
    #[serde(rename = "UnscSupplyPadBreakEvenPoint")]
    pub unsc_supply_pad_break_even_point: Option<f32>,
    /// Supply bonus of a Covenant supply pad (`CovSupplyPadBonus`).
    #[serde(rename = "CovSupplyPadBonus")]
    pub cov_supply_pad_bonus: Option<f32>,
    /// Break-even point of a Covenant supply pad (`CovSupplyPadBreakEvenPoint`).
    #[serde(rename = "CovSupplyPadBreakEvenPoint")]
    pub cov_supply_pad_break_even_point: Option<f32>,

    // Transport settings
    /// Maximum number of squads in one transport (`TransportMax`).
    #[serde(rename = "TransportMax")]
    pub transport_max: Option<u32>,

    // Cryo/freeze settings
    /// Time for a frozen unit to thaw (`TimeFrozenToThaw`).
    #[serde(rename = "TimeFrozenToThaw")]
    pub time_frozen_to_thaw: Option<f32>,
    /// Time for a freezing unit to thaw (`TimeFreezingToThaw`).
    #[serde(rename = "TimeFreezingToThaw")]
    pub time_freezing_to_thaw: Option<f32>,
    /// Default cryo points of a unit (`DefaultCryoPoints`).
    #[serde(rename = "DefaultCryoPoints")]
    pub default_cryo_points: Option<f32>,
    /// Default thaw speed (`DefaultThawSpeed`).
    #[serde(rename = "DefaultThawSpeed")]
    pub default_thaw_speed: Option<f32>,
    /// Speed multiplier for a freezing unit (`FreezingSpeedModifier`).
    #[serde(rename = "FreezingSpeedModifier")]
    pub freezing_speed_modifier: Option<f32>,
    /// Damage multiplier for a freezing unit (`FreezingDamageModifier`).
    #[serde(rename = "FreezingDamageModifier")]
    pub freezing_damage_modifier: Option<f32>,
    /// Damage multiplier for a frozen unit (`FrozenDamageModifier`).
    #[serde(rename = "FrozenDamageModifier")]
    pub frozen_damage_modifier: Option<f32>,

    // Leader power charge
    /// Resource that charges leader powers (`LeaderPowerChargeResource`).
    #[serde(rename = "LeaderPowerChargeResource")]
    pub leader_power_charge_resource: Option<String>,
    /// Rate that charges leader powers (`LeaderPowerChargeRate`).
    #[serde(rename = "LeaderPowerChargeRate")]
    pub leader_power_charge_rate: Option<String>,

    // AI difficulty
    /// Default difficulty of the skirmish AI (`DifficultySPCAIDefault`).
    #[serde(rename = "DifficultySPCAIDefault")]
    pub difficulty_spcai_default: Option<f32>,

    // Combat modifiers
    /// Damage multiplier for garrisoned units (`GarrisonDamageMultiplier`).
    #[serde(rename = "GarrisonDamageMultiplier")]
    pub garrison_damage_multiplier: Option<f32>,
    /// Damage multiplier for buildings under construction (`ConstructionDamageMultiplier`).
    #[serde(rename = "ConstructionDamageMultiplier")]
    pub construction_damage_multiplier: Option<f32>,
    /// Decay rate of capture progress (`CaptureDecayRate`).
    #[serde(rename = "CaptureDecayRate")]
    pub capture_decay_rate: Option<f32>,
    /// Maximum distance a unit moves from its leash point (`UnitLeashLength`).
    #[serde(rename = "UnitLeashLength")]
    pub unit_leash_length: Option<f32>,

    // Cloaking
    /// Delay before a unit cloaks (`CloakingDelay`).
    #[serde(rename = "CloakingDelay")]
    pub cloaking_delay: Option<f32>,
    /// Delay before a unit cloaks again (`ReCloakDelay`).
    #[serde(rename = "ReCloakDelay")]
    pub recloak_delay: Option<f32>,
    /// Frequency of cloak detection checks (`CloakDetectFrequency`).
    #[serde(rename = "CloakDetectFrequency")]
    pub cloak_detect_frequency: Option<f32>,

    // Shields
    /// Color of the shield bar (`ShieldBarColor`).
    #[serde(rename = "ShieldBarColor")]
    pub shield_bar_color: Option<RgbColor>,
    /// Delay before shields start to regenerate (`ShieldRegenDelay`).
    #[serde(rename = "ShieldRegenDelay")]
    pub shield_regen_delay: Option<f32>,
    /// Time for shields to fully regenerate (`ShieldRegenTime`).
    #[serde(rename = "ShieldRegenTime")]
    pub shield_regen_time: Option<f32>,
    /// Color of the ammo bar (`AmmoBarColor`).
    #[serde(rename = "AmmoBarColor")]
    pub ammo_bar_color: Option<RgbColor>,

    // Projectiles
    /// Gravity for projectiles (`ProjectileGravity`).
    #[serde(rename = "ProjectileGravity")]
    pub projectile_gravity: Option<f32>,
    /// Bonus damage for attacks from higher ground (`HeightBonusDamage`).
    #[serde(rename = "HeightBonusDamage")]
    pub height_bonus_damage: Option<f32>,
    /// Tumble rate of projectiles (`ProjectileTumbleRate`).
    #[serde(rename = "ProjectileTumbleRate")]
    pub projectile_tumble_rate: Option<f32>,
    /// Distance for intercepting a tracked target (`TrackInterceptDistance`).
    #[serde(rename = "TrackInterceptDistance")]
    pub track_intercept_distance: Option<f32>,

    // Attack tolerances
    /// Attack tolerance angle against a stationary target.
    #[serde(rename = "StationaryTargetAttackToleranceAngle")]
    pub stationary_target_attack_tolerance_angle: Option<f32>,
    /// Attack tolerance angle against a moving target.
    #[serde(rename = "MovingTargetAttackToleranceAngle")]
    pub moving_target_attack_tolerance_angle: Option<f32>,
    /// Attack tolerance angle against a moving target that the weapon tracks.
    #[serde(rename = "MovingTargetTrackingAttackToleranceAngle")]
    pub moving_target_tracking_attack_tolerance_angle: Option<f32>,
    /// Range multiplier against a moving target (`MovingTargetRangeMultiplier`).
    #[serde(rename = "MovingTargetRangeMultiplier")]
    pub moving_target_range_multiplier: Option<f32>,

    // Revealers
    /// Line of sight of the revealer on an attacked unit (`AttackedRevealerLOS`).
    #[serde(rename = "AttackedRevealerLOS")]
    pub attacked_revealer_los: Option<f32>,
    /// Lifespan of the revealer on an attacked unit (`AttackedRevealerLifespan`).
    #[serde(rename = "AttackedRevealerLifespan")]
    pub attacked_revealer_lifespan: Option<f32>,
    /// Line of sight of the revealer on an attacking unit (`AttackRevealerLOS`).
    #[serde(rename = "AttackRevealerLOS")]
    pub attack_revealer_los: Option<f32>,
    /// Lifespan of the revealer on an attacking unit (`AttackRevealerLifespan`).
    #[serde(rename = "AttackRevealerLifespan")]
    pub attack_revealer_lifespan: Option<f32>,
    /// Minimum size of a revealer (`MinimumRevealerSize`).
    #[serde(rename = "MinimumRevealerSize")]
    pub minimum_revealer_size: Option<f32>,

    // Ratings
    /// Multiplier for attack ratings (`AttackRatingMultiplier`).
    #[serde(rename = "AttackRatingMultiplier")]
    pub attack_rating_multiplier: Option<f32>,
    /// Multiplier for defense ratings (`DefenseRatingMultiplier`).
    #[serde(rename = "DefenseRatingMultiplier")]
    pub defense_rating_multiplier: Option<f32>,
    /// Minimum attack rating for a "good against" grade (`GoodAgainstMinAttackRating`).
    #[serde(
        rename = "GoodAgainstMinAttackRating",
        alias = "GoodAgainstMinAttackGrade"
    )]
    pub good_against_min_attack_rating: Option<u32>,

    // Opportunity
    /// Priority bonus for a target that attacks the unit (`OpportunityBeingAttackedPriBonus`).
    #[serde(rename = "OpportunityBeingAttackedPriBonus")]
    pub opportunity_being_attacked_pri_bonus: Option<f32>,
    /// Distance factor for opportunity target priority (`OpportunityDistPriFactor`).
    #[serde(rename = "OpportunityDistPriFactor")]
    pub opportunity_dist_pri_factor: Option<f32>,

    // Misc combat
    /// Chance that a killed unit is thrown by a rocket hit (`ChanceToRocket`).
    #[serde(rename = "ChanceToRocket")]
    pub chance_to_rocket: Option<f32>,
    /// Time for a building to self-destruct (`BuildingSelfDestructTime`).
    #[serde(rename = "BuildingSelfDestructTime")]
    pub building_self_destruct_time: Option<f32>,
    /// Reticle grades for each effectiveness band (`GoodAgainstReticle`).
    #[serde(rename = "GoodAgainstReticle")]
    pub good_against_reticle: Option<GoodAgainstReticle>,

    // Economy
    /// Amount that one tribute sends (`TributeAmount`).
    #[serde(rename = "TributeAmount")]
    pub tribute_amount: Option<f32>,
    /// Cost of one tribute (`TributeCost`).
    #[serde(rename = "TributeCost")]
    pub tribute_cost: Option<f32>,
    /// Loiter time of an air strike (`AirStrikeLoiterTime`).
    #[serde(rename = "AirStrikeLoiterTime")]
    pub air_strike_loiter_time: Option<f32>,

    // Damage bank
    /// Maximum percent adjustment of the damage bank (`MaxDamageBankPctAdjust`).
    #[serde(rename = "MaxDamageBankPctAdjust")]
    pub max_damage_bank_pct_adjust: Option<f32>,
    /// Timer of the damage bank (`DamageBankTimer`).
    #[serde(rename = "DamageBankTimer")]
    pub damage_bank_timer: Option<f32>,

    // Corpses
    /// Maximum number of corpses (`MaxNumCorpses`).
    #[serde(rename = "MaxNumCorpses")]
    pub max_num_corpses: Option<i32>,
    /// Maximum number of corpses to remove at one time (`MaxCorpseDisposalCount`).
    #[serde(rename = "MaxCorpseDisposalCount")]
    pub max_corpse_disposal_count: Option<i32>,
    /// Decay time of an infantry corpse (`InfantryCorpseDecayTime`).
    #[serde(rename = "InfantryCorpseDecayTime")]
    pub infantry_corpse_decay_time: Option<f32>,
    /// Spacing of sinking corpses (`CorpseSinkingSpacing`).
    #[serde(rename = "CorpseSinkingSpacing")]
    pub corpse_sinking_spacing: Option<f32>,

    // Infection
    /// Infection forms for each base proto-object (`InfectionMap`).
    #[serde(rename = "InfectionMap")]
    pub infection_map: Option<InfectionMapWrapper>,

    // Pathing
    /// Maximum squad path requests for each frame (`MaxSquadPathsPerFrame`).
    #[serde(rename = "MaxSquadPathsPerFrame")]
    pub max_squad_paths_per_frame: Option<u32>,
    /// Maximum platoon path requests for each frame (`MaxPlatoonPathsPerFrame`).
    #[serde(rename = "MaxPlatoonPathsPerFrame")]
    pub max_platoon_paths_per_frame: Option<u32>,

    // XP
    /// XP factor for received damage (`DamageReceivedXPFactor`).
    #[serde(rename = "DamageReceivedXPFactor")]
    pub damage_received_xp_factor: Option<f32>,

    // Fatalities
    /// Scale of the fatality transition (`FatalityTransitionScale`).
    #[serde(rename = "FatalityTransitionScale")]
    pub fatality_transition_scale: Option<f32>,
    /// Maximum time of the fatality transition (`FatalityMaxTransitionTime`).
    #[serde(rename = "FatalityMaxTransitionTime")]
    pub fatality_max_transition_time: Option<f32>,
    /// Cooldown between fatalities (`FatalityCooldown`).
    #[serde(rename = "FatalityCooldown")]
    pub fatality_cooldown: Option<f32>,
    /// Position offset tolerance for a fatality (`FatalityPositionOffsetTolerance`).
    #[serde(rename = "FatalityPositionOffsetTolerance")]
    pub fatality_position_offset_tolerance: Option<f32>,
    /// Orientation offset tolerance for a fatality (`FatalityOrientationOffsetTolerance`).
    #[serde(rename = "FatalityOrientationOffsetTolerance")]
    pub fatality_orientation_offset_tolerance: Option<f32>,
    /// Exclusion range around a fatality (`FatalityExclusionRange`).
    #[serde(rename = "FatalityExclusionRange")]
    pub fatality_exclusion_range: Option<f32>,

    // Recycle
    /// Refund rate when a building is recycled (`RecyleRefundRate`, spelled as in the data).
    #[serde(rename = "RecyleRefundRate")]
    pub recyle_refund_rate: Option<f32>,

    // Burning
    /// Limits of burning effects (`BurningEffectLimits`).
    #[serde(rename = "BurningEffectLimits")]
    pub burning_effect_limits: Option<BurningEffectLimitsWrapper>,

    // Base rebuild
    /// Timer before a base rebuilds (`BaseRebuildTimer`).
    #[serde(rename = "BaseRebuildTimer")]
    pub base_rebuild_timer: Option<f32>,

    // Objective arrows
    /// Radial offset of objective arrows (`ObjectiveArrowRadialOffset`).
    #[serde(rename = "ObjectiveArrowRadialOffset")]
    pub objective_arrow_radial_offset: Option<f32>,
    /// Switch offset of objective arrows (`ObjectiveArrowSwitchOffset`).
    #[serde(rename = "ObjectiveArrowSwitchOffset")]
    pub objective_arrow_switch_offset: Option<f32>,
    /// Maximum index of objective arrows (`ObjectiveArrowMaxIndex`).
    #[serde(
        rename = "ObjectiveArrowMaxIndex",
        default,
        deserialize_with = "super::types::deserialize_optional_u8",
        serialize_with = "super::types::serialize_optional_u8"
    )]
    pub objective_arrow_max_index: Option<u8>,
    /// Vertical offset of objective arrows (`ObjectiveArrowYOffset`).
    #[serde(rename = "ObjectiveArrowYOffset")]
    pub objective_arrow_y_offset: Option<f32>,

    // Overrun
    /// Minimum speed for an overrun (`OverrunMinVel`).
    #[serde(rename = "OverrunMinVel")]
    pub overrun_min_vel: Option<f32>,
    /// Jump force of an overrun (`OverrunJumpForce`).
    #[serde(rename = "OverrunJumpForce")]
    pub overrun_jump_force: Option<f32>,
    /// Distance of an overrun (`OverrunDistance`).
    #[serde(rename = "OverrunDistance")]
    pub overrun_distance: Option<f32>,

    // Co-op
    /// Resource split rate in co-op games (`CoopResourceSplitRate`).
    #[serde(rename = "CoopResourceSplitRate")]
    pub coop_resource_split_rate: Option<f32>,

    // Game over
    /// Delay before the game-over screen (`GameOverDelay`).
    #[serde(rename = "GameOverDelay")]
    pub game_over_delay: Option<f32>,

    // Hero settings
    /// Line of sight of a downed hero (`HeroDownedLOS`).
    #[serde(rename = "HeroDownedLOS")]
    pub hero_downed_los: Option<f32>,
    /// HP regeneration time of a hero (`HeroHPRegenTime`).
    #[serde(rename = "HeroHPRegenTime")]
    pub hero_hp_regen_time: Option<f32>,
    /// Distance at which a hero revives (`HeroRevivalDistance`).
    #[serde(rename = "HeroRevivalDistance")]
    pub hero_revival_distance: Option<f32>,
    /// Percent of HP at which a hero revives (`HeroPercentHPRevivalThreshhold`).
    #[serde(rename = "HeroPercentHPRevivalThreshhold")]
    pub hero_percent_hp_revival_threshhold: Option<f32>,
    /// Maximum transport distance for a dead hero (`MaxDeadHeroTransportDist`).
    #[serde(rename = "MaxDeadHeroTransportDist")]
    pub max_dead_hero_transport_dist: Option<f32>,

    // Transport settings (extended)
    /// Scale of the transport clear radius (`TransportClearRadiusScale`).
    #[serde(rename = "TransportClearRadiusScale")]
    pub transport_clear_radius_scale: Option<f32>,
    /// Scale of the maximum transport search radius (`TransportMaxSearchRadiusScale`).
    #[serde(rename = "TransportMaxSearchRadiusScale")]
    pub transport_max_search_radius_scale: Option<f32>,
    /// Maximum number of transport search locations (`TransportMaxSearchLocations`).
    #[serde(rename = "TransportMaxSearchLocations")]
    pub transport_max_search_locations: Option<u32>,
    /// Transport block time (`TransportBlockTime`).
    #[serde(rename = "TransportBlockTime")]
    pub transport_block_time: Option<u32>,
    /// Transport load block time (`TransportLoadBlockTime`).
    #[serde(rename = "TransportLoadBlockTime")]
    pub transport_load_block_time: Option<u32>,
    /// Maximum number of transport block attempts (`TransportMaxBlockAttempts`).
    #[serde(rename = "TransportMaxBlockAttempts")]
    pub transport_max_block_attempts: Option<u32>,
    /// Height of an incoming transport (`TransportIncomingHeight`).
    #[serde(rename = "TransportIncomingHeight")]
    pub transport_incoming_height: Option<f32>,
    /// Offset of an incoming transport (`TransportIncomingOffset`).
    #[serde(rename = "TransportIncomingOffset")]
    pub transport_incoming_offset: Option<f32>,
    /// Height of an outgoing transport (`TransportOutgoingHeight`).
    #[serde(rename = "TransportOutgoingHeight")]
    pub transport_outgoing_height: Option<f32>,
    /// Offset of an outgoing transport (`TransportOutgoingOffset`).
    #[serde(rename = "TransportOutgoingOffset")]
    pub transport_outgoing_offset: Option<f32>,
    /// Height of a transport at pickup (`TransportPickupHeight`).
    #[serde(rename = "TransportPickupHeight")]
    pub transport_pickup_height: Option<f32>,
    /// Height of a transport at drop-off (`TransportDropoffHeight`).
    #[serde(rename = "TransportDropoffHeight")]
    pub transport_dropoff_height: Option<f32>,

    // Hitch
    /// Hitch offset (`HitchOffset`).
    #[serde(rename = "HitchOffset")]
    pub hitch_offset: Option<f32>,

    // Animal life (AL) settings
    /// Maximum wander frequency of animal life (`ALMaxWanderFrequency`).
    #[serde(rename = "ALMaxWanderFrequency")]
    pub al_max_wander_frequency: Option<f32>,
    /// Predator check frequency of animal life (`ALPredatorCheckFrequency`).
    #[serde(rename = "ALPredatorCheckFrequency")]
    pub al_predator_check_frequency: Option<f32>,
    /// Prey check frequency of animal life (`ALPreyCheckFrequency`).
    #[serde(rename = "ALPreyCheckFrequency")]
    pub al_prey_check_frequency: Option<f32>,
    /// Opponent check radius of animal life (`ALOppCheckRadius`).
    #[serde(rename = "ALOppCheckRadius")]
    pub al_opp_check_radius: Option<f32>,
    /// Flee distance of animal life (`ALFleeDistance`).
    #[serde(rename = "ALFleeDistance")]
    pub al_flee_distance: Option<f32>,
    /// Movement multiplier of fleeing animal life (`ALFleeMovementModifier`).
    #[serde(rename = "ALFleeMovementModifier")]
    pub al_flee_movement_modifier: Option<f32>,
    /// Minimum wander distance of animal life (`ALMinWanderDistance`).
    #[serde(rename = "ALMinWanderDistance")]
    pub al_min_wander_distance: Option<f32>,
    /// Maximum wander distance of animal life (`ALMaxWanderDistance`).
    #[serde(rename = "ALMaxWanderDistance")]
    pub al_max_wander_distance: Option<f32>,
    /// Spawner check frequency of animal life (`ALSpawnerCheckFrequency`).
    #[serde(rename = "ALSpawnerCheckFrequency")]
    pub al_spawner_check_frequency: Option<f32>,

    // Dot damage
    /// Minimap dot sizes (`Dot`).
    #[serde(rename = "Dot")]
    pub dot: Option<DotSettings>,

    // Squad leash
    /// Maximum distance a squad moves from its leash point (`SquadLeashLength`).
    #[serde(rename = "SquadLeashLength")]
    pub squad_leash_length: Option<f32>,
    /// Aggro distance of a squad (`SquadAggroLength`).
    #[serde(rename = "SquadAggroLength")]
    pub squad_aggro_length: Option<f32>,
}

/// Wrapper for `<BurningEffectLimits>`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BurningEffectLimitsWrapper {
    /// Default limit for effect types without an entry (`@DefaultLimit`).
    #[serde(rename = "@DefaultLimit")]
    pub default_limit: Option<i32>,
    /// Limits for each effect type, from the `<BurningEffectLimitEntry>` children.
    #[serde(rename = "BurningEffectLimitEntry", default)]
    pub entries: Vec<BurningEffectLimitEntry>,
}

/// A burning effect limit entry.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BurningEffectLimitEntry {
    /// Limit for this entry (`@Limit`).
    #[serde(rename = "@Limit")]
    pub limit: Option<i32>,
    /// Name from the element text that the limit applies to.
    #[serde(rename = "$text", default)]
    pub value: String,
}

/// Wrapper for `<InfectionMap>`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InfectionMapWrapper {
    /// Entries from the `<InfectionMapEntry>` children.
    #[serde(rename = "InfectionMapEntry", default)]
    pub entries: Vec<InfectionMapEntry>,
}

/// Maps a base proto-object to its infected proto-object and squad forms.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InfectionMapEntry {
    /// Proto-object that becomes infected (`@base`).
    #[serde(rename = "@base", default)]
    pub base: String,
    /// Infected proto-object (`@infected`).
    #[serde(rename = "@infected", default)]
    pub infected: String,
    /// Infected proto-squad (`@infectedSquad`).
    #[serde(rename = "@infectedSquad", default)]
    pub infected_squad: String,
}

/// Reticle grades used for the five attack-effectiveness bands.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GoodAgainstReticle {
    /// Reticle grade for no effect.
    #[serde(rename = "NoEffect")]
    pub no_effect: Option<u32>,
    /// Reticle grade for weak effect.
    #[serde(rename = "Weak")]
    pub weak: Option<u32>,
    /// Reticle grade for fair effect.
    #[serde(rename = "Fair")]
    pub fair: Option<u32>,
    /// Reticle grade for good effect.
    #[serde(rename = "Good")]
    pub good: Option<u32>,
    /// Reticle grade for extreme effect.
    #[serde(rename = "Extreme")]
    pub extreme: Option<u32>,
}

/// Minimap dot sizes for small and medium units.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DotSettings {
    /// Dot size for small units (`@small`).
    #[serde(rename = "@small")]
    pub small: Option<f32>,
    /// Dot size for medium units (`@medium`).
    #[serde(rename = "@medium")]
    pub medium: Option<f32>,
}

/// Wrapper for `<Resources>` containing `<Resource>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ResourcesWrapper {
    /// Resource definitions, one per `<Resource>` child.
    #[serde(rename = "Resource", default)]
    pub entries: Vec<ResourceDef>,
}

/// A resource definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ResourceDef {
    /// Resource name from the element text.
    #[serde(rename = "$text", default)]
    pub name: String,
    /// When `true`, the resource is deductable (`@Deductable`).
    #[serde(rename = "@Deductable")]
    pub deductable: Option<bool>,
}

/// Wrapper for `<Rates>` containing `<Rate>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RatesWrapper {
    /// Rate names, one per `<Rate>` child.
    #[serde(rename = "Rate", default)]
    pub entries: Vec<String>,
}

/// Wrapper for `<Pops>` containing `<Pop>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PopsWrapper {
    /// Population names, one per `<Pop>` child.
    #[serde(rename = "Pop", default)]
    pub entries: Vec<String>,
}

/// Wrapper for `<RefCounts>` containing `<RefCount>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RefCountsWrapper {
    /// Reference count names, one per `<RefCount>` child.
    #[serde(rename = "RefCount", default)]
    pub entries: Vec<String>,
}

/// Wrapper for `<HUDItems>` containing `<HUDItem>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HUDItemsWrapper {
    /// HUD item names, one per `<HUDItem>` child.
    #[serde(rename = "HUDItem", default)]
    pub entries: Vec<String>,
}

/// Wrapper for `<FlashableItems>` containing `<Item>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FlashableItemsWrapper {
    /// Flashable item names, one per `<Item>` child.
    #[serde(rename = "Item", default)]
    pub entries: Vec<String>,
}

/// Wrapper for `<UnitFlags>` containing `<UnitFlag>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UnitFlagsWrapper {
    /// Unit flag names, one per `<UnitFlag>` child.
    #[serde(rename = "UnitFlag", default)]
    pub entries: Vec<String>,
}

/// Wrapper for `<SquadFlags>` containing `<SquadFlag>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SquadFlagsWrapper {
    /// Squad flag names, one per `<SquadFlag>` child.
    #[serde(rename = "SquadFlag", default)]
    pub entries: Vec<String>,
}

/// Wrapper for `<PlayerStates>` containing `<PlayerState>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlayerStatesWrapper {
    /// Player state names, one per `<PlayerState>` child.
    #[serde(rename = "PlayerState", default)]
    pub entries: Vec<String>,
}

/// Wrapper for `<CodeProtoObjects>`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CodeProtoObjectsWrapper {
    /// Mappings, one per `<CodeProtoObject>` child.
    #[serde(rename = "CodeProtoObject", default)]
    pub entries: Vec<CodeProtoObject>,
}

/// A code proto-object mapping.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CodeProtoObject {
    /// Code type name from `@Type`.
    #[serde(rename = "@Type", default)]
    pub object_type: String,
    /// Proto-object name from the element text.
    #[serde(rename = "$text", default)]
    pub proto_name: String,
}

/// Wrapper for `<CodeObjectTypes>`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CodeObjectTypesWrapper {
    /// Mappings, one per `<CodeObjectType>` child.
    #[serde(rename = "CodeObjectType", default)]
    pub entries: Vec<CodeObjectType>,
}

/// A code object type mapping.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CodeObjectType {
    /// Code type name from `@Type`.
    #[serde(rename = "@Type", default)]
    pub object_type: String,
    /// Object type name from the element text.
    #[serde(rename = "$text", default)]
    pub value: String,
}

/// Parse game data from a `gamedata.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or the game data cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<GameData> {
    let root = expect_root(doc, "GameData")?;
    let gd: GameData = bdt_serde::from_node(root)?;
    Ok(gd)
}
