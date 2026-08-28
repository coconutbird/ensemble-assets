//! Parser for `objects.xml.xmb` — the master proto-object database.
//!
//! Each `<Object>` element describes a game entity: unit, building, projectile, etc.

use alloc::string::String;
use alloc::vec::Vec;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::node_ext::expect_root;

use super::Vector3;

/// A single proto-object definition from `objects.xml`.
///
/// Every field below is confirmed present in the engine's `BProtoObject::load`
/// (IDA @ `0x140341620`) and cross-referenced against the 2006 source leak
/// (`BProtoObject::load`). Fields are sorted alphabetically within each group.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProtoObject {
    /// Object name (unique key), e.g. `"unsc_veh_warthog_01"`.
    #[serde(rename = "@name", default)]
    pub name: String,

    /// Numeric ID assigned in the XML (`id` attribute, often 0).
    #[serde(rename = "@id")]
    pub id: Option<i32>,

    /// Unused leftover from development — typo for `@id`, ignored by the engine.
    #[serde(rename = "@is")]
    pub is: Option<i32>,

    /// Database ID assigned by the engine (`dbid` attribute).
    #[serde(rename = "@dbid")]
    pub dbid: Option<i32>,

    /// Whether this is an update entry (from `objects_update.xml.xmb`).
    #[serde(rename = "@update")]
    pub update: Option<bool>,

    /// Ability trigger script names.
    #[serde(rename = "Ability", default)]
    pub ability: Vec<String>,

    /// Ability command ID (resolved via `BDatabase::getAbilityIDFromName`).
    #[serde(rename = "AbilityCommand")]
    pub ability_command: Option<String>,

    /// Acceleration override for projectiles. Stored on `mpStaticData`.
    #[serde(rename = "Acceleration")]
    pub acceleration: Option<f32>,

    /// Chance (0.0–1.0) the unit will perform an active scan. Has `@radiusScale` sub-attr.
    #[serde(rename = "ActiveScanChance")]
    pub active_scan_chance: Option<ActiveScanChance>,

    /// Resource type and amount to add on creation. Has `@Amount` sub-attr.
    #[serde(rename = "AddResource")]
    pub add_resource: Option<AddedResource>,

    /// AI asset value adjustment factor for AI threat calculations.
    #[serde(rename = "AIAssetValueAdjust")]
    pub ai_asset_value_adjust: Option<f32>,

    /// Maximum ammo capacity.
    #[serde(rename = "AmmoMax")]
    pub ammo_max: Option<f32>,

    /// Ammo regeneration rate per second.
    #[serde(rename = "AmmoRegenRate")]
    pub ammo_regen_rate: Option<f32>,

    /// DPS grade for attack rating display. Stored as `String` due to malformed data (`"190.9.0"`).
    #[serde(rename = "AttackGradeDPS")]
    pub attack_grade_dps: Option<String>,

    /// Auto lock-down type enum (resolved via `BDatabase::getAutoLockDownType`).
    #[serde(rename = "AutoLockDown")]
    pub auto_lock_down: Option<String>,

    /// Proto object name for the auto-spawned parking lot. Has `@Rotation` and `@Offset` sub-attrs.
    #[serde(rename = "AutoParkingLot")]
    pub auto_parking_lot: Option<AutoParkingLot>,

    /// Proto squad to auto-train when the building finishes construction.
    #[serde(rename = "AutoTrainOnBuilt")]
    pub auto_train_on_built: Option<String>,

    /// Object type ID for the beam head visual (e.g. Scarab beam). (unsure)
    #[serde(rename = "BeamHead")]
    pub beam_head: Option<String>,

    /// Object type ID for the beam tail visual (e.g. Scarab beam). (unsure)
    #[serde(rename = "BeamTail")]
    pub beam_tail: Option<String>,

    /// Proto object ref that blocks this object's movement.
    #[serde(rename = "BlockMovementObject")]
    pub block_movement_object: Option<String>,

    /// Bounty value awarded on kill.
    #[serde(rename = "Bounty")]
    pub bounty: Option<f32>,

    /// Build offset vector (x, y, z) for building placement.
    #[serde(rename = "BuildOffset")]
    pub build_offset: Option<Vector3>,

    /// Construction hit points. Only set if `mBuildPoints == 0.0`.
    #[serde(rename = "BuildPoints")]
    pub build_points: Option<f32>,

    /// Rotation angle for building placement.
    #[serde(rename = "BuildRotation")]
    pub build_rotation: Option<f32>,

    /// Proto object ref used for build stats display (e.g. shows another object's stats).
    #[serde(rename = "BuildStatsObject")]
    pub build_stats_object: Option<String>,

    /// Building strength display ID (resolved via `BDatabase::getProtoBuildingStrengthID`).
    #[serde(rename = "BuildingStrengthDisplay")]
    pub building_strength_display: Option<String>,

    /// Capture cost per civ. Complex sub-element with `@Civ` attr.
    #[serde(rename = "CaptureCost", default)]
    pub capture_costs: Vec<CaptureCost>,

    /// Child objects (parking lots, sockets, rally points, etc.). Contains `<Object>` sub-elements.
    #[serde(rename = "ChildObjects")]
    pub child_objects: Option<ChildObjects>,

    /// Damage scalar applied to child objects attached to this object.
    #[serde(rename = "ChildObjectDamageTakenScalar")]
    pub child_object_damage_taken_scalar: Option<f32>,

    /// Combat value used for AI scoring and game statistics.
    #[serde(rename = "CombatValue")]
    pub combat_value: Option<f32>,

    /// Commands available on this building (Train, Research, Build, etc.). Has `@Type`/`@Position` sub-attrs.
    #[serde(rename = "Command", default)]
    pub commands: Vec<ObjectCommand>,

    /// Object types this unit can contain/transport.
    #[serde(rename = "Contain", default)]
    pub contain: Vec<String>,

    /// Corpse death visual (proto visual name).
    #[serde(rename = "CorpseDeath")]
    pub corpse_death: Option<String>,

    /// Cost element — loaded via `BCost::load`. Contains resource sub-elements.
    #[serde(rename = "Cost", default)]
    pub costs: Vec<ResourceCost>,

    /// Cost escalation factor — multiplied per existing instance.
    #[serde(rename = "CostEscalation")]
    pub cost_escalation: Option<f32>,

    /// Proto object names whose count drives cost escalation.
    #[serde(rename = "CostEscalationObject", default)]
    pub cost_escalation_object: Vec<String>,

    /// Damage type sub-element (loaded via `loadDamageType`).
    #[serde(rename = "DamageType")]
    pub damage_type: Option<String>,

    /// Unused leftover from development — `DazeResist` is read from squads, not objects.
    #[serde(rename = "DazeResist")]
    pub daze_resist: Option<f32>,

    /// Death fade delay before fading begins (seconds).
    #[serde(rename = "DeathFadeDelayTime")]
    pub death_fade_delay_time: Option<f32>,

    /// Death fade duration (seconds).
    #[serde(rename = "DeathFadeTime")]
    pub death_fade_time: Option<f32>,

    /// Proto object ref to spawn on death.
    #[serde(rename = "DeathReplacement")]
    pub death_replacement: Option<String>,

    /// Proto squad to spawn on death. Has `@CheckPos` and `@MaxPopCount` sub-attrs.
    #[serde(rename = "DeathSpawnSquad")]
    pub death_spawn_squad: Option<DeathSpawnSquad>,

    /// Display name localized string ID.
    #[serde(rename = "DisplayNameID")]
    pub display_name_id: Option<i32>,

    /// Enemy rollover text localized string ID.
    #[serde(rename = "EnemyRolloverTextID")]
    pub enemy_rollover_text_id: Option<i32>,

    /// Exist sound bone attachment name. Has `@bone` sub-attr.
    #[serde(rename = "ExistSound")]
    pub exist_sound: Option<ExistSound>,

    /// Direction index for exiting garrisoned units. (unsure)
    #[serde(rename = "ExitFromDirection")]
    pub exit_from_direction: Option<i32>,

    /// Extended sound bank name for additional audio events.
    #[serde(rename = "ExtendedSoundBank")]
    pub extended_sound_bank: Option<String>,

    /// Boolean flags (e.g. `"ForceToGaiaPlayer"`, `"Invulnerable"`, `"NoRender"`).
    #[serde(rename = "Flag", default)]
    pub flags: Vec<String>,

    /// Unused leftover from development — loaded by the Flash UI system, not the object loader.
    #[serde(rename = "FlashUI")]
    pub flash_ui: Option<String>,

    /// Terrain flatten region 0 max X bound (building placement).
    #[serde(rename = "FlattenMaxX0")]
    pub flatten_max_x0: Option<f32>,

    /// Terrain flatten region 1 max X bound (building placement).
    #[serde(rename = "FlattenMaxX1")]
    pub flatten_max_x1: Option<f32>,

    /// Terrain flatten region 0 max Z bound (building placement).
    #[serde(rename = "FlattenMaxZ0")]
    pub flatten_max_z0: Option<f32>,

    /// Terrain flatten region 1 max Z bound (building placement).
    #[serde(rename = "FlattenMaxZ1")]
    pub flatten_max_z1: Option<f32>,

    /// Terrain flatten region 0 min X bound (building placement).
    #[serde(rename = "FlattenMinX0")]
    pub flatten_min_x0: Option<f32>,

    /// Terrain flatten region 1 min X bound (building placement).
    #[serde(rename = "FlattenMinX1")]
    pub flatten_min_x1: Option<f32>,

    /// Terrain flatten region 0 min Z bound (building placement).
    #[serde(rename = "FlattenMinZ0")]
    pub flatten_min_z0: Option<f32>,

    /// Terrain flatten region 1 min Z bound (building placement).
    #[serde(rename = "FlattenMinZ1")]
    pub flatten_min_z1: Option<f32>,

    /// Flight level height for air units.
    #[serde(rename = "FlightLevel")]
    pub flight_level: Option<f32>,

    /// Fuel capacity for projectiles (determines range).
    #[serde(rename = "Fuel")]
    pub fuel: Option<f32>,

    /// Gaia rollover text localized string ID. Has optional `@civ` sub-attr.
    #[serde(rename = "GaiaRolloverTextID", default)]
    pub gaia_rollover_text_ids: Vec<GaiaRolloverTextId>,

    /// Squad mode for garrisoned units (resolved via `BDatabase::getSquadMode`).
    #[serde(rename = "GarrisonSquadMode")]
    pub garrison_squad_mode: Option<String>,

    /// Time in seconds for garrison action. (unsure)
    #[serde(rename = "GarrisonTime")]
    pub garrison_time: Option<f32>,

    /// Gather link — object type for resource linking. Has `@Radius`, `@Target`, `@Self` sub-attrs.
    #[serde(rename = "GatherLink")]
    pub gather_link: Option<GatherLink>,

    /// Max number of gatherers that can work on this resource.
    #[serde(rename = "GathererLimit")]
    pub gatherer_limit: Option<i32>,

    /// Goto type enum: where the unit goes to (resolved via `BDatabase::getGotoType`).
    #[serde(rename = "GotoType")]
    pub goto_type: Option<String>,

    /// Ground IK bone name and settings. Has `@factor` sub-attr. (unsure)
    #[serde(rename = "GroundIK", default)]
    pub ground_ik: Vec<GroundIk>,

    /// Ground IK tilt settings. Has `@factor` sub-attr and bone name as text.
    #[serde(rename = "GroundIKTilt")]
    pub ground_ik_tilt: Option<GroundIkTilt>,

    /// Hardpoints (turret mount points).
    #[serde(rename = "Hardpoint", default)]
    pub hardpoints: Vec<Hardpoint>,

    /// Hit points.
    #[serde(rename = "Hitpoints")]
    pub hitpoints: Option<f32>,

    /// Hit zones for per-bone damage. Has `@Hitpoints`, `@Shieldpoints`, `@Active`, `@HasShields` sub-attrs.
    #[serde(rename = "HitZone", default)]
    pub hit_zones: Vec<HitZone>,

    /// Hovering rumble controller data. Has `@LeftRumbleType`, `@RightRumbleType`, `@Duration`, etc.
    #[serde(rename = "HoveringRumble")]
    pub hovering_rumble: Option<HoveringRumble>,

    /// HP bar display ID. Has `@sizeX`, `@sizeY`, `@offset` sub-attrs.
    #[serde(rename = "HPBar")]
    pub hp_bar: Option<HpBar>,

    /// Impact decal configuration. Has `@sizeX`, `@sizeZ`, `@timeFullyOpaque`, `@fadeOutTime`, `@orientation` sub-attrs.
    #[serde(rename = "ImpactDecal")]
    pub impact_decal: Option<ImpactDecal>,

    /// Kill beam proto object ref (Scarab scenario hack).
    #[serde(rename = "KillBeam")]
    pub kill_beam: Option<String>,

    /// Proto object level (e.g. tech level). Read as int.
    #[serde(rename = "Level")]
    pub level: Option<i32>,

    /// Level-up visual effect proto object ref.
    #[serde(rename = "LevelUpEffect")]
    pub level_up_effect: Option<String>,

    /// Lifespan in seconds (read as float, engine converts to milliseconds).
    #[serde(rename = "Lifespan")]
    pub lifespan: Option<f32>,

    /// Line of sight radius.
    #[serde(rename = "LOS")]
    pub los: Option<f32>,

    /// Maximum number of contained/transported units.
    #[serde(rename = "MaxContained")]
    pub max_contained: Option<i32>,

    /// Maximum number of simultaneous flame effects on this object.
    #[serde(rename = "MaxFlameEffects")]
    pub max_flame_effects: Option<i32>,

    /// Maximum projectile height for arc calculations.
    #[serde(rename = "MaxProjectileHeight")]
    pub max_projectile_height: Option<f32>,

    /// Maximum velocity (overrides auto-calculated `Velocity * cMaxVelocityMultiplier`).
    #[serde(rename = "MaxVelocity")]
    pub max_velocity: Option<f32>,

    /// Minimap color. Has `@red`, `@green`, `@blue` sub-attrs.
    #[serde(rename = "MinimapColor")]
    pub minimap_color: Option<MinimapColor>,

    /// Minimap icon path. Has `@size` sub-attr.
    #[serde(rename = "MinimapIcon")]
    pub minimap_icon: Option<MinimapIcon>,

    /// Unused leftover from development — not read by `BProtoObject::load`.
    #[serde(rename = "MinimapIconName")]
    pub minimap_icon_name: Option<String>,

    /// Movement type string: `"Land"`, `"Air"`, `"Flood"`, etc.
    #[serde(rename = "MovementType")]
    pub movement_type: Option<String>,

    /// Number of conversions (Flood mechanic). (unsure)
    #[serde(rename = "NumConversions")]
    pub num_conversions: Option<i32>,

    /// Number of stasis fields required to immobilize this unit.
    #[serde(rename = "NumStasisFieldsToStop")]
    pub num_stasis_fields_to_stop: Option<i32>,

    /// Object class: `"Object"`, `"Unit"`, `"Squad"`, `"Building"`, `"Projectile"`.
    #[serde(rename = "ObjectClass")]
    pub object_class: Option<String>,

    /// Object type tags (e.g. `"Military"`, `"CovInfantry"`). Resolved via `BDatabase::getObjectType`.
    #[serde(rename = "ObjectType", default)]
    pub object_types: Vec<String>,

    /// Obstruction radius X (half-width for pathfinding collision box).
    #[serde(rename = "ObstructionRadiusX")]
    pub obstruction_radius_x: Option<f32>,

    /// Obstruction radius Y (height for pathfinding collision box).
    #[serde(rename = "ObstructionRadiusY")]
    pub obstruction_radius_y: Option<f32>,

    /// Obstruction radius Z (half-depth for pathfinding collision box).
    #[serde(rename = "ObstructionRadiusZ")]
    pub obstruction_radius_z: Option<f32>,

    /// Parking lot max X bound (building placement).
    #[serde(rename = "ParkingMaxX")]
    pub parking_max_x: Option<f32>,

    /// Parking lot max Z bound (building placement).
    #[serde(rename = "ParkingMaxZ")]
    pub parking_max_z: Option<f32>,

    /// Parking lot min X bound (building placement).
    #[serde(rename = "ParkingMinX")]
    pub parking_min_x: Option<f32>,

    /// Parking lot min Z bound (building placement).
    #[serde(rename = "ParkingMinZ")]
    pub parking_min_z: Option<f32>,

    /// Initial velocity perturbance for projectiles. Sets `mFlagPerturbOnce`. Has `@minTime`/`@maxTime` sub-attrs.
    #[serde(rename = "PerturbInitialVelocity")]
    pub perturb_initial_velocity: Option<PerturbInitialVelocity>,

    /// Chance (0.0–1.0) of projectile perturbance per tick.
    #[serde(rename = "PerturbanceChance")]
    pub perturbance_chance: Option<f32>,

    /// Maximum perturbance time interval (seconds).
    #[serde(rename = "PerturbanceMaxTime")]
    pub perturbance_max_time: Option<f32>,

    /// Minimum perturbance time interval (seconds).
    #[serde(rename = "PerturbanceMinTime")]
    pub perturbance_min_time: Option<f32>,

    /// Perturbance velocity magnitude.
    #[serde(rename = "PerturbanceVelocity")]
    pub perturbance_velocity: Option<f32>,

    /// Physics info name (resolved via `gPhysicsInfoManager::getOrCreate`).
    #[serde(rename = "PhysicsInfo")]
    pub physics_info: Option<String>,

    /// Physics replacement info name (used on death/destruction).
    #[serde(rename = "PhysicsReplacementInfo")]
    pub physics_replacement_info: Option<String>,

    /// Pick offset — Y offset for mouse picking ray test.
    #[serde(rename = "PickOffset")]
    pub pick_offset: Option<f32>,

    /// Pick priority enum (resolved via `BDatabase::getPickPriority`).
    #[serde(rename = "PickPriority")]
    pub pick_priority: Option<String>,

    /// Pick radius — radius for mouse picking ray test.
    #[serde(rename = "PickRadius")]
    pub pick_radius: Option<f32>,

    /// Placement rules enum (resolved via `BDatabase::getPlacementRules`).
    #[serde(rename = "PlacementRules")]
    pub placement_rules: Option<String>,

    /// Population cost entries. Has `@type` sub-attr for pop type name.
    #[serde(rename = "Pop", default)]
    pub population: Vec<PopulationAmount>,

    /// Population cap addition entries. Has `@type` sub-attr for pop type name.
    #[serde(rename = "PopCapAddition", default)]
    pub population_cap_additions: Vec<PopulationAmount>,

    /// Portrait icon path.
    #[serde(rename = "PortraitIcon")]
    pub portrait_icon: Option<String>,

    /// Proto power ID (resolved via `BDatabase::getProtoPowerIDByName`).
    #[serde(rename = "Power")]
    pub power: Option<String>,

    /// Prereq text localized string ID.
    #[serde(rename = "PrereqTextID")]
    pub prereq_text_id: Option<i32>,

    /// Rally point type: `"Military"` or `"Civilian"`.
    #[serde(rename = "RallyPoint")]
    pub rally_point: Option<String>,

    /// Ram dodge factor for ram-attack dodging. (unsure)
    #[serde(rename = "RamDodgeFactor")]
    pub ram_dodge_factor: Option<f32>,

    /// Rate element — resource gather/work rate. Has `@rate` sub-attr for rate type name.
    #[serde(rename = "Rate")]
    pub rate: Option<ObjectRate>,

    /// Recovering visual effect proto object ref.
    #[serde(rename = "RecoveringEffect")]
    pub recovering_effect: Option<String>,

    /// Repair points (construction-like HP for repair actions).
    #[serde(rename = "RepairPoints")]
    pub repair_points: Option<f32>,

    /// Resource amount this object provides when gathered.
    #[serde(rename = "ResourceAmount")]
    pub resource_amount: Option<f32>,

    /// Reveal radius — fog-of-war reveal distance (distinct from LOS).
    #[serde(rename = "RevealRadius")]
    pub reveal_radius: Option<f32>,

    /// Reverse movement speed.
    #[serde(rename = "ReverseSpeed")]
    pub reverse_speed: Option<f32>,

    /// Role text localized string ID.
    #[serde(rename = "RoleTextID")]
    pub role_text_id: Option<i32>,

    /// Rollover text localized string ID.
    #[serde(rename = "RolloverTextID")]
    pub rollover_text_id: Option<i32>,

    /// Select type enum: `"Building"`, `"Resource"`, `"Unit"`, `"Rally"`.
    #[serde(rename = "SelectType")]
    pub select_type: Option<String>,

    /// Selected radius X (visual selection circle half-width).
    #[serde(rename = "SelectedRadiusX")]
    pub selected_radius_x: Option<f32>,

    /// Selected radius Z (visual selection circle half-depth).
    #[serde(rename = "SelectedRadiusZ")]
    pub selected_radius_z: Option<f32>,

    /// Shield hit points.
    #[serde(rename = "Shieldpoints")]
    pub shieldpoints: Option<f32>,

    /// Shield type — object type ID for shield (e.g. energy shield visual).
    #[serde(rename = "ShieldType")]
    pub shield_type: Option<String>,

    /// Single bone IK settings. (unsure)
    #[serde(rename = "SingleBoneIK", default)]
    pub single_bone_ik: Vec<String>,

    /// Socket object type and settings. Has `@player` scope and `@AutoSocket` sub-attrs.
    #[serde(rename = "Socket")]
    pub socket: Option<Socket>,

    /// Sound entries (Create, Death, Select, etc.). Has `@Type`, `@Squad`, `@Action` sub-attrs.
    #[serde(rename = "Sound", default)]
    pub sounds: Vec<ObjectSound>,

    /// Squad mode animation overrides. Has `@Mode` sub-attr.
    #[serde(rename = "SquadModeAnim", default)]
    pub squad_mode_animations: Vec<SquadModeAnimation>,

    /// Starting velocity for projectiles.
    #[serde(rename = "StartingVelocity")]
    pub starting_velocity: Option<f32>,

    /// Stats name localized string ID (used in post-game stats).
    #[serde(rename = "StatsNameID")]
    pub stats_name_id: Option<i32>,

    /// Sub-select sort priority index.
    #[serde(rename = "SubSelectSort")]
    pub sub_select_sort: Option<i32>,

    /// Surface type enum (resolved via `BDatabase::getSurfaceType`).
    #[serde(rename = "SurfaceType")]
    pub surface_type: Option<String>,

    /// Sweet spot IK settings. (unsure)
    #[serde(rename = "SweetSpotIK", default)]
    pub sweet_spot_ik: Vec<SweetSpotIk>,

    /// Tactics file base name (resolves to `data\tactics\{name}.xmb`).
    #[serde(rename = "Tactics")]
    pub tactics: Option<String>,

    /// Target beam proto object ref (Scarab scenario hack).
    #[serde(rename = "TargetBeam")]
    pub target_beam: Option<String>,

    /// Terrain height tolerance for placement (default 10.0 in 2006 source).
    #[serde(rename = "TerrainHeightTolerance")]
    pub terrain_height_tolerance: Option<f32>,

    /// Tracking delay in seconds (engine multiplies by 1000 and stores as DWORD ms).
    #[serde(rename = "TrackingDelay")]
    pub tracking_delay: Option<f32>,

    /// Unused leftover from development — `TrackInterceptDistance` is read from gamedata, not objects.
    #[serde(rename = "TrackInterceptDistance")]
    pub track_intercept_distance: Option<f32>,

    /// Train animation type (resolved via `gVisualManager::getAnimType`).
    #[serde(rename = "TrainAnim")]
    pub train_anim: Option<String>,

    /// Trainer type (int). Has optional `@ApplyFormation` bool sub-attr.
    #[serde(rename = "TrainerType")]
    pub trainer_type: Option<TrainerType>,

    /// Train limits per unit/squad type. Has `@Type`, `@Count`, `@Bucket` sub-attrs.
    #[serde(rename = "TrainLimit", default)]
    pub train_limits: Vec<TrainLimit>,

    /// True LOS height — height at which LOS checks originate. (unsure)
    #[serde(rename = "TrueLOSHeight")]
    pub true_los_height: Option<f32>,

    /// Turn rate (degrees per second).
    #[serde(rename = "TurnRate")]
    pub turn_rate: Option<f32>,

    /// Unused leftover from development — not read by `BProtoObject::load`.
    #[serde(rename = "UIVisual")]
    pub ui_visual: Option<String>,

    /// Desired velocity. Also auto-sets `MaxVelocity` and `Acceleration`.
    #[serde(rename = "Velocity")]
    pub velocity: Option<f32>,

    /// Veterancy level definitions with XP thresholds and stat bonuses.
    #[serde(rename = "Veterancy", default)]
    pub veterancy: Vec<VeterancyLevel>,

    /// Visual proto name (resolved via `gVisualManager::getOrCreateProtoVisual`).
    #[serde(rename = "Visual")]
    pub visual: Option<String>,

    /// Visual display priority (resolved via `BDatabase::getVisualDisplayPriority`).
    #[serde(rename = "VisualDisplayPriority")]
    pub visual_display_priority: Option<String>,
}

/// Active-scan probability plus the optional scan-radius multiplier.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ActiveScanChance {
    #[serde(rename = "$text", default)]
    pub chance: f32,
    #[serde(rename = "@radiusScale")]
    pub radius_scale: Option<f32>,
}

/// A resource granted when the object is created.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AddedResource {
    #[serde(rename = "$text", default)]
    pub resource_type: String,
    #[serde(rename = "@Amount")]
    pub amount: Option<f32>,
}

/// An automatically spawned parking-lot object and its transform.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AutoParkingLot {
    #[serde(rename = "$text", default)]
    pub proto_object: String,
    #[serde(rename = "@Rotation")]
    pub rotation: Option<f32>,
    #[serde(rename = "@Offset")]
    pub offset: Option<Vector3>,
}

/// A cost assigned to one of the engine's resource slots.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ResourceCost {
    #[serde(rename = "@ResourceType", alias = "@resourcetype", default)]
    pub resource_type: String,
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// A capture cost, optionally applied to just one civilization.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CaptureCost {
    #[serde(rename = "@Civ")]
    pub civilization: Option<String>,
    #[serde(rename = "@ResourceType", alias = "@resourcetype", default)]
    pub resource_type: String,
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// Wrapper for the `<ChildObjects>` collection.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChildObjects {
    #[serde(rename = "Object", default)]
    pub objects: Vec<ChildObject>,
}

/// Runtime attachment behavior for a child object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildObjectType {
    /// Ordinary attached proto-object.
    Object,
    ParkingLot,
    Socket,
    Rally,
    OneTimeSpawnSquad,
    Unit,
    Foundation,
}

impl<'de> Deserialize<'de> for ChildObjectType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "Object" => Ok(Self::Object),
            "ParkingLot" => Ok(Self::ParkingLot),
            "Socket" => Ok(Self::Socket),
            "Rally" => Ok(Self::Rally),
            "OneTimeSpawnSquad" => Ok(Self::OneTimeSpawnSquad),
            "Unit" => Ok(Self::Unit),
            "Foundation" => Ok(Self::Foundation),
            value => Err(D::Error::custom(format_args!(
                "unknown child-object type `{value}`"
            ))),
        }
    }
}

impl Serialize for ChildObjectType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::Object => "Object",
            Self::ParkingLot => "ParkingLot",
            Self::Socket => "Socket",
            Self::Rally => "Rally",
            Self::OneTimeSpawnSquad => "OneTimeSpawnSquad",
            Self::Unit => "Unit",
            Self::Foundation => "Foundation",
        })
    }
}

/// One object attached through a `<ChildObjects>` entry.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChildObject {
    #[serde(rename = "$text", default)]
    pub proto_object: String,
    #[serde(rename = "@Type")]
    pub child_type: Option<ChildObjectType>,
    #[serde(rename = "@AttachBone")]
    pub attach_bone: Option<String>,
    #[serde(rename = "@Offset")]
    pub offset: Option<Vector3>,
    /// XML degrees; the engine converts this value to radians.
    #[serde(rename = "@Rotation")]
    pub rotation: Option<f32>,
    #[serde(rename = "@UserCiv")]
    pub user_civilization: Option<String>,
}

/// A command-button entry on a proto object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectCommand {
    #[serde(rename = "$text", default)]
    pub target: String,
    #[serde(rename = "@Type")]
    pub command_type: Option<String>,
    #[serde(rename = "@Position")]
    pub position: Option<i32>,
    #[serde(rename = "@AutoClose")]
    pub auto_close: Option<bool>,
}

/// A squad spawned when this object dies.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DeathSpawnSquad {
    #[serde(rename = "$text", default)]
    pub proto_squad: String,
    /// The retail loader treats the attribute's presence as enabled.
    #[serde(rename = "@CheckPos")]
    pub check_position: Option<bool>,
    #[serde(rename = "@MaxPopCount")]
    pub max_population_count: Option<i32>,
}

/// Attachment bone for an object's persistent sound.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExistSound {
    #[serde(rename = "@bone")]
    pub bone: Option<String>,
}

/// A Gaia rollover string, optionally overriding one civilization slot.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GaiaRolloverTextId {
    #[serde(rename = "$text", default)]
    pub text_id: i32,
    #[serde(rename = "@civ")]
    pub civilization: Option<String>,
}

/// Links this object to a gather target and resource behavior.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GatherLink {
    #[serde(rename = "$text", default)]
    pub object_type: String,
    #[serde(rename = "@Radius")]
    pub radius: Option<f32>,
    #[serde(rename = "@Target")]
    pub target: Option<String>,
    #[serde(rename = "@Self")]
    pub self_link: Option<bool>,
}

/// One ground inverse-kinematics chain.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GroundIk {
    #[serde(rename = "$text", default)]
    pub bone: String,
    #[serde(rename = "@ikRange")]
    pub ik_range: Option<f32>,
    #[serde(
        rename = "@linkCount",
        default,
        deserialize_with = "super::types::deserialize_optional_u8",
        serialize_with = "super::types::serialize_optional_u8"
    )]
    pub link_count: Option<u8>,
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@z")]
    pub z: Option<f32>,
}

/// Ground-IK tilt bone and multiplier.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GroundIkTilt {
    #[serde(rename = "$text", default)]
    pub bone: String,
    #[serde(rename = "@factor")]
    pub factor: Option<f32>,
}

/// Per-bone hit-point and shield overrides.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HitZone {
    #[serde(rename = "$text", default)]
    pub bone: String,
    #[serde(rename = "@Hitpoints")]
    pub hitpoints: Option<f32>,
    #[serde(rename = "@Shieldpoints")]
    pub shieldpoints: Option<f32>,
    #[serde(rename = "@Active")]
    pub active: Option<bool>,
    #[serde(rename = "@HasShields")]
    pub has_shields: Option<bool>,
}

/// Controller rumble emitted by a hovering object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HoveringRumble {
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

/// HP-bar style plus optional geometry overrides.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HpBar {
    #[serde(rename = "$text")]
    pub style: Option<String>,
    #[serde(rename = "@sizeX")]
    pub size_x: Option<f32>,
    #[serde(rename = "@sizeY")]
    pub size_y: Option<f32>,
    #[serde(rename = "@offset")]
    pub offset: Option<Vector3>,
}

/// Supported impact-decal orientation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactDecalOrientation {
    Random,
    Forward,
}

impl<'de> Deserialize<'de> for ImpactDecalOrientation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.eq_ignore_ascii_case("random") {
            Ok(Self::Random)
        } else if value.eq_ignore_ascii_case("forward") {
            Ok(Self::Forward)
        } else {
            Err(D::Error::custom(format_args!(
                "unknown impact-decal orientation `{value}`"
            )))
        }
    }
}

impl Serialize for ImpactDecalOrientation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::Random => "random",
            Self::Forward => "forward",
        })
    }
}

/// Decal material, dimensions, timing, and orientation.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ImpactDecal {
    #[serde(rename = "$text", default)]
    pub material: String,
    #[serde(rename = "@sizeX")]
    pub size_x: Option<f32>,
    #[serde(rename = "@sizeZ")]
    pub size_z: Option<f32>,
    #[serde(rename = "@timeFullyOpaque")]
    pub time_fully_opaque: Option<f32>,
    #[serde(rename = "@fadeOutTime")]
    pub fade_out_time: Option<f32>,
    #[serde(rename = "@orientation")]
    pub orientation: Option<ImpactDecalOrientation>,
}

/// Minimap color represented by the loader's three float attributes.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MinimapColor {
    #[serde(rename = "@red")]
    pub red: Option<f32>,
    #[serde(rename = "@green")]
    pub green: Option<f32>,
    #[serde(rename = "@blue")]
    pub blue: Option<f32>,
}

/// Minimap icon identifier and size override.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MinimapIcon {
    #[serde(rename = "$text")]
    pub icon: Option<String>,
    #[serde(rename = "@size")]
    pub size: Option<f32>,
}

/// One-time projectile velocity perturbation settings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PerturbInitialVelocity {
    #[serde(rename = "$text", default)]
    pub velocity: f32,
    #[serde(rename = "@minTime")]
    pub min_time: Option<f32>,
    #[serde(rename = "@maxTime")]
    pub max_time: Option<f32>,
}

/// Population cost or cap amount associated with a population type.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PopulationAmount {
    #[serde(rename = "@type")]
    pub population_type: Option<String>,
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// Resource gather/work rate and its rate-table key.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectRate {
    #[serde(rename = "@rate")]
    pub rate_type: Option<String>,
    #[serde(rename = "$text", default)]
    pub value: f32,
}

/// Socket object type, owner scope, and auto-socket flag.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Socket {
    #[serde(rename = "$text", default)]
    pub object_type: String,
    #[serde(rename = "@player")]
    pub player: Option<String>,
    #[serde(rename = "@AutoSocket")]
    pub auto_socket: Option<bool>,
}

/// An object sound cue and its optional runtime filters.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectSound {
    #[serde(rename = "$text", default)]
    pub cue: String,
    #[serde(rename = "@Type")]
    pub sound_type: Option<String>,
    #[serde(rename = "@Squad")]
    pub squad: Option<String>,
    #[serde(rename = "@Action")]
    pub action: Option<String>,
}

/// Animation mapped to a squad mode.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SquadModeAnimation {
    #[serde(rename = "$text", default)]
    pub animation: String,
    #[serde(rename = "@Mode")]
    pub mode: Option<String>,
}

/// A sweet-spot IK bone and its chain length.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SweetSpotIk {
    #[serde(rename = "$text", default)]
    pub bone: String,
    #[serde(
        rename = "@linkCount",
        default,
        deserialize_with = "super::types::deserialize_optional_u8",
        serialize_with = "super::types::serialize_optional_u8"
    )]
    pub link_count: Option<u8>,
}

/// Trainer category and formation behavior.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TrainerType {
    #[serde(rename = "$text", default)]
    pub trainer_type: i32,
    #[serde(rename = "@ApplyFormation")]
    pub apply_formation: Option<bool>,
}

/// Target database selected by a train-limit entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainLimitType {
    Unit,
    Squad,
}

impl<'de> Deserialize<'de> for TrainLimitType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "Unit" => Ok(Self::Unit),
            "Squad" => Ok(Self::Squad),
            value => Err(D::Error::custom(format_args!(
                "unknown train-limit type `{value}`"
            ))),
        }
    }
}

impl Serialize for TrainLimitType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::Unit => "Unit",
            Self::Squad => "Squad",
        })
    }
}

/// A train-limit target with count and shared-bucket constraints.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TrainLimit {
    #[serde(rename = "$text", default)]
    pub target: String,
    #[serde(rename = "@Type")]
    pub limit_type: Option<TrainLimitType>,
    #[serde(
        rename = "@Count",
        default,
        deserialize_with = "super::types::deserialize_optional_u8",
        serialize_with = "super::types::serialize_optional_u8"
    )]
    pub count: Option<u8>,
    #[serde(
        rename = "@Bucket",
        default,
        deserialize_with = "super::types::deserialize_optional_u8",
        serialize_with = "super::types::serialize_optional_u8"
    )]
    pub bucket: Option<u8>,
}

/// A hardpoint on a proto-object (turret mount point).
///
/// Loaded by `BHardpoint::load`. Each hardpoint has a yaw and pitch bone
/// attachment, rotation rates, angle limits, and optional sound cues.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Hardpoint {
    /// Hardpoint name (unique within the object).
    #[serde(rename = "@name", default)]
    pub name: String,

    /// Whether the turret auto-centers when idle (default `true`).
    #[serde(rename = "@autocenter")]
    pub autocenter: Option<bool>,

    /// Combined yaw+pitch on a single bone. Auto-set `true` when yaw and pitch
    /// attachments are the same bone (unless `useYawAndPitchAsTolerance` is set).
    #[serde(rename = "@combined")]
    pub combined: Option<bool>,

    /// Use hard (clamped) pitch limits instead of soft limits.
    #[serde(rename = "@hardpitchlimits")]
    pub hard_pitch_limits: Option<bool>,

    /// Skip rotation rate limiting when the hardpoint has an active target.
    #[serde(rename = "@infiniteRateWhenHasTarget")]
    pub infinite_rate_when_has_target: Option<bool>,

    /// Bone attachment name for pitch rotation (resolved via `gVisualManager::getAttachmentType`).
    #[serde(rename = "@pitchattachment")]
    pub pitch_attachment: Option<String>,

    /// Maximum pitch angle (radians). `pitchMinAngle` defaults to `-pitchMaxAngle`.
    #[serde(rename = "@pitchMaxAngle")]
    pub pitch_max_angle: Option<f32>,

    /// Minimum pitch angle (radians). Defaults to `-pitchMaxAngle` if not set.
    #[serde(rename = "@pitchMinAngle")]
    pub pitch_min_angle: Option<f32>,

    /// Pitch rotation rate (radians/sec, default `cPiOver12`).
    #[serde(rename = "@pitchrate")]
    pub pitch_rate: Option<f32>,

    /// Turret rotation is relative to the unit's facing direction.
    #[serde(rename = "@relativeToUnit")]
    pub relative_to_unit: Option<bool>,

    /// Use single-bone IK for this hardpoint.
    #[serde(rename = "@singleboneik")]
    pub single_bone_ik: Option<bool>,

    /// Sound cue played when pitch rotation starts.
    #[serde(rename = "@StartPitchSound")]
    pub start_pitch_sound: Option<String>,

    /// Sound cue played when yaw rotation starts.
    #[serde(rename = "@StartYawSound")]
    pub start_yaw_sound: Option<String>,

    /// Sound cue played when pitch rotation stops.
    #[serde(rename = "@StopPitchSound")]
    pub stop_pitch_sound: Option<String>,

    /// Sound cue played when yaw rotation stops.
    #[serde(rename = "@StopYawSound")]
    pub stop_yaw_sound: Option<String>,

    /// Treat yaw/pitch max angles as tolerance values instead of hard limits.
    #[serde(rename = "@useYawAndPitchAsTolerance")]
    pub use_yaw_and_pitch_as_tolerance: Option<bool>,

    /// Bone attachment name for yaw rotation (resolved via `gVisualManager::getAttachmentType`).
    #[serde(rename = "@yawattachment")]
    pub yaw_attachment: Option<String>,

    /// Symmetric yaw limit (radians). Sets both left and right max angles to `±value`.
    #[serde(rename = "@yawMaxAngle")]
    pub yaw_max_angle: Option<f32>,

    /// Left yaw limit (radians, default `-π`). Overrides the left side of `yawMaxAngle`.
    #[serde(rename = "@YawLeftMaxAngle")]
    pub yaw_left_max_angle: Option<f32>,

    /// Right yaw limit (radians, default `π`). Overrides the right side of `yawMaxAngle`.
    #[serde(rename = "@YawRightMaxAngle")]
    pub yaw_right_max_angle: Option<f32>,

    /// Yaw rotation rate (radians/sec, default `cPiOver12`).
    #[serde(rename = "@yawrate")]
    pub yaw_rate: Option<f32>,
}

/// A veterancy (experience) level entry (`BProtoObjectLevel`).
///
/// Each level defines an XP threshold and stat multipliers stored as half-floats
/// in the engine. Only levels with `Level > 0` are loaded.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VeterancyLevel {
    /// Veterancy level number (must be > 0 to be loaded).
    #[serde(rename = "@Level", default)]
    pub level: i32,

    /// XP threshold required to reach this level.
    #[serde(rename = "@XP")]
    pub xp: Option<f32>,

    /// Damage output multiplier at this level.
    #[serde(rename = "@Damage")]
    pub damage: Option<f32>,

    /// Damage taken multiplier at this level (< 1.0 = more resistant).
    #[serde(rename = "@DamageTaken")]
    pub damage_taken: Option<f32>,

    /// Movement velocity multiplier at this level.
    #[serde(rename = "@Velocity")]
    pub velocity: Option<f32>,

    /// Accuracy multiplier at this level.
    #[serde(rename = "@Accuracy")]
    pub accuracy: Option<f32>,

    /// Work rate multiplier at this level (gathering, building, etc.).
    #[serde(rename = "@WorkRate")]
    pub work_rate: Option<f32>,

    /// Weapon range multiplier at this level.
    #[serde(rename = "@WeaponRange")]
    pub weapon_range: Option<f32>,
}

/// Parse all proto-objects from an `objects.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or an object cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<ProtoObject>> {
    let root = expect_root(doc, "Objects")?;
    let objects: Vec<ProtoObject> = root
        .children
        .iter()
        .filter(|c| c.name == "Object")
        .map(bdt_serde::from_node)
        .collect::<Result<_, _>>()?;
    Ok(objects)
}
