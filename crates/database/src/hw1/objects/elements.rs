//! Child-element types of a `<Object>` entry in `objects.xml`.
//!
//! Each type maps one XML sub-element. The element text goes to one field and
//! the XML attributes go to the other fields.

use alloc::string::String;
use alloc::vec::Vec;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hw1::Vector3;

/// Active-scan probability plus the optional scan-radius multiplier.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ActiveScanChance {
    /// Chance (0.0–1.0) of an active scan. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub chance: f32,
    /// Multiplier for the scan radius (`@radiusScale`).
    #[serde(rename = "@radiusScale")]
    pub radius_scale: Option<f32>,
}

/// A resource granted when the object is created.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AddedResource {
    /// Resource type name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub resource_type: String,
    /// Quantity of the resource to add (`@Amount`).
    #[serde(rename = "@Amount")]
    pub amount: Option<f32>,
}

/// An automatically spawned parking-lot object and its transform.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AutoParkingLot {
    /// Proto-object name of the parking lot. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub proto_object: String,
    /// Rotation of the parking lot (`@Rotation`).
    #[serde(rename = "@Rotation")]
    pub rotation: Option<f32>,
    /// Position offset of the parking lot from the parent (`@Offset`).
    #[serde(rename = "@Offset")]
    pub offset: Option<Vector3>,
}

/// A cost assigned to one of the engine's resource slots.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ResourceCost {
    /// Resource type name (`@ResourceType` or `@resourcetype`).
    #[serde(rename = "@ResourceType", alias = "@resourcetype", default)]
    pub resource_type: String,
    /// Cost quantity. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// A capture cost, optionally applied to just one civilization.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CaptureCost {
    /// Civilization that this cost applies to (`@Civ`). `None` applies to all.
    #[serde(rename = "@Civ")]
    pub civilization: Option<String>,
    /// Resource type name (`@ResourceType` or `@resourcetype`).
    #[serde(rename = "@ResourceType", alias = "@resourcetype", default)]
    pub resource_type: String,
    /// Cost quantity. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// Wrapper for the `<ChildObjects>` collection.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChildObjects {
    /// The `<Object>` child entries.
    #[serde(rename = "Object", default)]
    pub objects: Vec<ChildObject>,
}

/// Runtime attachment behavior for a child object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildObjectType {
    /// Ordinary attached proto-object.
    Object,
    /// Parking lot for trained units (`ParkingLot`).
    ParkingLot,
    /// Socket where a building can attach (`Socket`).
    Socket,
    /// Rally point (`Rally`).
    Rally,
    /// Squad that spawns one time (`OneTimeSpawnSquad`).
    OneTimeSpawnSquad,
    /// Attached unit (`Unit`).
    Unit,
    /// Building foundation (`Foundation`).
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
    /// Proto-object or proto-squad name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub proto_object: String,
    /// Attachment behavior (`@Type`).
    #[serde(rename = "@Type")]
    pub child_type: Option<ChildObjectType>,
    /// Bone that the child attaches to (`@AttachBone`).
    #[serde(rename = "@AttachBone")]
    pub attach_bone: Option<String>,
    /// Position offset from the parent (`@Offset`).
    #[serde(rename = "@Offset")]
    pub offset: Option<Vector3>,
    /// XML degrees; the engine converts this value to radians.
    #[serde(rename = "@Rotation")]
    pub rotation: Option<f32>,
    /// Civilization of the user that gets this child (`@UserCiv`).
    #[serde(rename = "@UserCiv")]
    pub user_civilization: Option<String>,
}

/// A command-button entry on a proto object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectCommand {
    /// Command target, for example a squad or tech name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub target: String,
    /// Command kind, for example `Train` or `Research` (`@Type`).
    #[serde(rename = "@Type")]
    pub command_type: Option<String>,
    /// Button slot index (`@Position`).
    #[serde(rename = "@Position")]
    pub position: Option<i32>,
    /// Close the command menu after use (`@AutoClose`).
    #[serde(rename = "@AutoClose")]
    pub auto_close: Option<bool>,
}

/// A squad spawned when this object dies.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DeathSpawnSquad {
    /// Proto-squad name to spawn. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub proto_squad: String,
    /// The retail loader treats the attribute's presence as enabled.
    #[serde(rename = "@CheckPos")]
    pub check_position: Option<bool>,
    /// Population limit for the spawn (`@MaxPopCount`).
    #[serde(rename = "@MaxPopCount")]
    pub max_population_count: Option<i32>,
}

/// Attachment bone for an object's persistent sound.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExistSound {
    /// Bone name that the sound attaches to (`@bone`).
    #[serde(rename = "@bone")]
    pub bone: Option<String>,
}

/// A Gaia rollover string, optionally overriding one civilization slot.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GaiaRolloverTextId {
    /// Localized string ID. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub text_id: i32,
    /// Civilization slot that this text overrides (`@civ`).
    #[serde(rename = "@civ")]
    pub civilization: Option<String>,
}

/// Links this object to a gather target and resource behavior.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GatherLink {
    /// Object type to link to. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub object_type: String,
    /// Search radius for the link (`@Radius`).
    #[serde(rename = "@Radius")]
    pub radius: Option<f32>,
    /// Link target selector (`@Target`).
    #[serde(rename = "@Target")]
    pub target: Option<String>,
    /// Link the object to itself (`@Self`).
    #[serde(rename = "@Self")]
    pub self_link: Option<bool>,
}

/// One ground inverse-kinematics chain.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GroundIk {
    /// IK end bone name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub bone: String,
    /// IK reach range (`@ikRange`).
    #[serde(rename = "@ikRange")]
    pub ik_range: Option<f32>,
    /// Number of bones in the IK chain (`@linkCount`).
    #[serde(
        rename = "@linkCount",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_u8",
        serialize_with = "crate::hw1::types::serialize_optional_u8"
    )]
    pub link_count: Option<u8>,
    /// X offset of the ground contact point (`@x`).
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    /// Z offset of the ground contact point (`@z`).
    #[serde(rename = "@z")]
    pub z: Option<f32>,
}

/// Ground-IK tilt bone and multiplier.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GroundIkTilt {
    /// Tilt bone name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub bone: String,
    /// Tilt multiplier (`@factor`).
    #[serde(rename = "@factor")]
    pub factor: Option<f32>,
}

/// Per-bone hit-point and shield overrides.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HitZone {
    /// Bone name of the hit zone. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub bone: String,
    /// Hit points of the zone (`@Hitpoints`).
    #[serde(rename = "@Hitpoints")]
    pub hitpoints: Option<f32>,
    /// Shield points of the zone (`@Shieldpoints`).
    #[serde(rename = "@Shieldpoints")]
    pub shieldpoints: Option<f32>,
    /// Zone is active at start (`@Active`).
    #[serde(rename = "@Active")]
    pub active: Option<bool>,
    /// Zone has shields (`@HasShields`).
    #[serde(rename = "@HasShields")]
    pub has_shields: Option<bool>,
}

/// Controller rumble emitted by a hovering object.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HoveringRumble {
    /// Rumble type for the left motor (`@LeftRumbleType`).
    #[serde(rename = "@LeftRumbleType")]
    pub left_rumble_type: Option<String>,
    /// Rumble type for the right motor (`@RightRumbleType`).
    #[serde(rename = "@RightRumbleType")]
    pub right_rumble_type: Option<String>,
    /// Rumble duration (`@Duration`).
    #[serde(rename = "@Duration")]
    pub duration: Option<f32>,
    /// Strength of the left motor (`@LeftStrength`).
    #[serde(rename = "@LeftStrength")]
    pub left_strength: Option<f32>,
    /// Strength of the right motor (`@RightStrength`).
    #[serde(rename = "@RightStrength")]
    pub right_strength: Option<f32>,
}

/// HP-bar style plus optional geometry overrides.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HpBar {
    /// HP-bar style name. Comes from the element text.
    #[serde(rename = "$text")]
    pub style: Option<String>,
    /// Bar width override (`@sizeX`).
    #[serde(rename = "@sizeX")]
    pub size_x: Option<f32>,
    /// Bar height override (`@sizeY`).
    #[serde(rename = "@sizeY")]
    pub size_y: Option<f32>,
    /// Bar position offset (`@offset`).
    #[serde(rename = "@offset")]
    pub offset: Option<Vector3>,
}

/// Supported impact-decal orientation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactDecalOrientation {
    /// Random rotation (`random`).
    Random,
    /// Rotation follows the forward direction (`forward`).
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
    /// Decal material name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub material: String,
    /// Decal width (`@sizeX`).
    #[serde(rename = "@sizeX")]
    pub size_x: Option<f32>,
    /// Decal depth (`@sizeZ`).
    #[serde(rename = "@sizeZ")]
    pub size_z: Option<f32>,
    /// Time that the decal stays fully opaque (`@timeFullyOpaque`).
    #[serde(rename = "@timeFullyOpaque")]
    pub time_fully_opaque: Option<f32>,
    /// Time that the decal takes to fade out (`@fadeOutTime`).
    #[serde(rename = "@fadeOutTime")]
    pub fade_out_time: Option<f32>,
    /// Decal orientation mode (`@orientation`).
    #[serde(rename = "@orientation")]
    pub orientation: Option<ImpactDecalOrientation>,
}

/// Minimap color represented by the loader's three float attributes.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MinimapColor {
    /// Red channel (`@red`).
    #[serde(rename = "@red")]
    pub red: Option<f32>,
    /// Green channel (`@green`).
    #[serde(rename = "@green")]
    pub green: Option<f32>,
    /// Blue channel (`@blue`).
    #[serde(rename = "@blue")]
    pub blue: Option<f32>,
}

/// Minimap icon identifier and size override.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MinimapIcon {
    /// Icon identifier. Comes from the element text.
    #[serde(rename = "$text")]
    pub icon: Option<String>,
    /// Icon size override (`@size`).
    #[serde(rename = "@size")]
    pub size: Option<f32>,
}

/// One-time projectile velocity perturbation settings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PerturbInitialVelocity {
    /// Perturbation velocity magnitude. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub velocity: f32,
    /// Minimum perturbation time (`@minTime`).
    #[serde(rename = "@minTime")]
    pub min_time: Option<f32>,
    /// Maximum perturbation time (`@maxTime`).
    #[serde(rename = "@maxTime")]
    pub max_time: Option<f32>,
}

/// Population cost or cap amount associated with a population type.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PopulationAmount {
    /// Population type name (`@type`).
    #[serde(rename = "@type")]
    pub population_type: Option<String>,
    /// Population quantity. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// Resource gather/work rate and its rate-table key.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectRate {
    /// Rate-table key (`@rate`).
    #[serde(rename = "@rate")]
    pub rate_type: Option<String>,
    /// Rate value. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub value: f32,
}

/// Socket object type, owner scope, and auto-socket flag.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Socket {
    /// Object type that can use the socket. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub object_type: String,
    /// Player scope of the socket (`@player`).
    #[serde(rename = "@player")]
    pub player: Option<String>,
    /// Fill the socket automatically (`@AutoSocket`).
    #[serde(rename = "@AutoSocket")]
    pub auto_socket: Option<bool>,
}

/// An object sound cue and its optional runtime filters.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectSound {
    /// Sound cue name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub cue: String,
    /// Sound event, for example `Create` or `Death` (`@Type`).
    #[serde(rename = "@Type")]
    pub sound_type: Option<String>,
    /// Squad filter (`@Squad`).
    #[serde(rename = "@Squad")]
    pub squad: Option<String>,
    /// Action filter (`@Action`).
    #[serde(rename = "@Action")]
    pub action: Option<String>,
}

/// Animation mapped to a squad mode.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SquadModeAnimation {
    /// Animation name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub animation: String,
    /// Squad mode name (`@Mode`).
    #[serde(rename = "@Mode")]
    pub mode: Option<String>,
}

/// A sweet-spot IK bone and its chain length.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SweetSpotIk {
    /// IK end bone name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub bone: String,
    /// Number of bones in the IK chain (`@linkCount`).
    #[serde(
        rename = "@linkCount",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_u8",
        serialize_with = "crate::hw1::types::serialize_optional_u8"
    )]
    pub link_count: Option<u8>,
}

/// Trainer category and formation behavior.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TrainerType {
    /// Trainer category number. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub trainer_type: i32,
    /// Put trained units in formation (`@ApplyFormation`).
    #[serde(rename = "@ApplyFormation")]
    pub apply_formation: Option<bool>,
}

/// Target database selected by a train-limit entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainLimitType {
    /// The target is a proto-object (`Unit`).
    Unit,
    /// The target is a proto-squad (`Squad`).
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
    /// Unit or squad name to limit. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub target: String,
    /// Database of the target (`@Type`).
    #[serde(rename = "@Type")]
    pub limit_type: Option<TrainLimitType>,
    /// Maximum number of the target (`@Count`).
    #[serde(
        rename = "@Count",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_u8",
        serialize_with = "crate::hw1::types::serialize_optional_u8"
    )]
    pub count: Option<u8>,
    /// Shared limit bucket index (`@Bucket`).
    #[serde(
        rename = "@Bucket",
        default,
        deserialize_with = "crate::hw1::types::deserialize_optional_u8",
        serialize_with = "crate::hw1::types::serialize_optional_u8"
    )]
    pub bucket: Option<u8>,
}
