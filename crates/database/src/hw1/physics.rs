//! Parser for physics XMB files.
//!
//! Physics data is split across three files per entity:
//! - `*.physics.xmb` — physics config (blueprint ref, vehicle type, center offset)
//! - `*.blueprint.xmb` — physical properties (mass, friction, restitution, shape ref)
//! - `*.shp.xmb` — Havok collision shape (XML-serialized, e.g. `hkBoxShape`)

use alloc::string::String;
use alloc::vec::Vec;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::node_ext::expect_root;

use super::Vector3;

/// Physics configuration from a `.physics.xmb` file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Physics {
    /// Blueprint name references. Clamshell objects can use three.
    #[serde(rename = "blueprint", default)]
    pub blueprints: Vec<String>,
    /// Whether this object can be thrown by projectiles.
    #[serde(
        rename = "ThrownByProjectiles",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub thrown_by_projectiles: Option<bool>,
    /// Whether projectile impacts destroy this object.
    #[serde(rename = "DestroyedByProjectiles")]
    pub destroyed_by_projectiles: Option<bool>,
    /// Terrain effects path.
    #[serde(rename = "TerrainEffects")]
    pub terrain_effects: Option<String>,
    /// Whether physics is activated when the owning entity dies.
    #[serde(rename = "PhysicsOnDeath")]
    pub physics_on_death: Option<bool>,
    /// Vehicle class and optional vehicle tuning data.
    #[serde(rename = "Vehicle", alias = "vehicle")]
    pub vehicle: Option<PhysicsVehicle>,
    /// Three-part upper/lower/pelvis rigid-body definition.
    #[serde(rename = "clamshell")]
    pub clamshell: Option<Clamshell>,
    /// Center-of-mass offset.
    #[serde(rename = "CenterOffset", alias = "centerOffset")]
    pub center_offset: Option<Vector3>,
    /// Minimum impact velocity that triggers impact behavior.
    #[serde(rename = "ImpactMinVelocity")]
    pub impact_min_velocity: Option<f32>,
    /// Presence-based phantom-body flag.
    #[serde(
        rename = "Phantom",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub phantom: Option<bool>,
    /// Havok motion mode.
    #[serde(rename = "MotionType")]
    pub motion_type: Option<MotionType>,
}

impl Physics {
    /// Return the primary blueprint used by the legacy single-body pipeline.
    #[must_use]
    pub fn primary_blueprint(&self) -> Option<&str> {
        self.blueprints.first().map(String::as_str).or_else(|| {
            self.clamshell
                .as_ref()
                .and_then(|clamshell| clamshell.upper.as_ref())
                .map(|part| part.blueprint.as_str())
        })
    }
}

/// A physics vehicle class and its optional tuning block.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PhysicsVehicle {
    /// Vehicle class from the element text. `false` means no vehicle.
    #[serde(rename = "$text", default)]
    pub vehicle_type: PhysicsVehicleType,
    /// Optional tuning block from the `<info>` child.
    #[serde(rename = "info")]
    pub info: Option<PhysicsVehicleInfo>,
}

/// Vehicle classes recognized by `BPhysicsInfo::load`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PhysicsVehicleType {
    /// The XML spelling used for a non-vehicle body.
    #[default]
    None,
    /// UNSC Warthog (`warthog`).
    Warthog,
    /// Covenant Ghost (`ghost`).
    Ghost,
    /// UNSC Scorpion tank (`scorpion`).
    Scorpion,
    /// UNSC Cobra (`cobra`).
    Cobra,
    /// UNSC Wolverine (`wolverine`).
    Wolverine,
    /// UNSC Grizzly tank (`grizzly`).
    Grizzly,
    /// UNSC Elephant (`elephant`).
    Elephant,
    /// UNSC Hawk (`hawk`).
    Hawk,
    /// UNSC Hornet (`hornet`).
    Hornet,
    /// UNSC Vulture (`vulture`).
    Vulture,
    /// Covenant Banshee (`banshee`).
    Banshee,
    /// Covenant Vampire (`vampire`).
    Vampire,
    /// Forerunner Sentinel (`sentinel`).
    Sentinel,
    /// Generic ground vehicle (`ground`).
    Ground,
    /// UNSC Gremlin (`gremlin`).
    Gremlin,
    /// Covenant Chopper (`chopper`).
    Chopper,
    /// UNSC Rhino (`rhino`).
    Rhino,
    /// Reactor body (`reactor`).
    Reactor,
}

impl<'de> Deserialize<'de> for PhysicsVehicleType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.trim() {
            "false" => Ok(Self::None),
            "warthog" => Ok(Self::Warthog),
            "ghost" => Ok(Self::Ghost),
            "scorpion" => Ok(Self::Scorpion),
            "cobra" => Ok(Self::Cobra),
            "wolverine" => Ok(Self::Wolverine),
            "grizzly" => Ok(Self::Grizzly),
            "elephant" => Ok(Self::Elephant),
            "hawk" => Ok(Self::Hawk),
            "hornet" => Ok(Self::Hornet),
            "vulture" => Ok(Self::Vulture),
            "banshee" => Ok(Self::Banshee),
            "vampire" => Ok(Self::Vampire),
            "sentinel" => Ok(Self::Sentinel),
            "ground" => Ok(Self::Ground),
            "gremlin" => Ok(Self::Gremlin),
            "chopper" => Ok(Self::Chopper),
            "rhino" => Ok(Self::Rhino),
            "reactor" => Ok(Self::Reactor),
            value => Err(D::Error::custom(format_args!(
                "unknown physics vehicle type `{value}`"
            ))),
        }
    }
}

impl Serialize for PhysicsVehicleType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::None => "false",
            Self::Warthog => "warthog",
            Self::Ghost => "ghost",
            Self::Scorpion => "scorpion",
            Self::Cobra => "cobra",
            Self::Wolverine => "wolverine",
            Self::Grizzly => "grizzly",
            Self::Elephant => "elephant",
            Self::Hawk => "hawk",
            Self::Hornet => "hornet",
            Self::Vulture => "vulture",
            Self::Banshee => "banshee",
            Self::Vampire => "vampire",
            Self::Sentinel => "sentinel",
            Self::Ground => "ground",
            Self::Gremlin => "gremlin",
            Self::Chopper => "chopper",
            Self::Rhino => "rhino",
            Self::Reactor => "reactor",
        })
    }
}

/// Extra suspension and tread tuning nested below a vehicle element.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PhysicsVehicleInfo {
    /// Suspension settings from `<suspension>`.
    #[serde(rename = "suspension")]
    pub suspension: Option<VehicleSuspension>,
    /// Chassis inertia settings from `<chassisInertia>`.
    #[serde(rename = "chassisInertia")]
    pub chassis_inertia: Option<ChassisInertia>,
    /// Wheel hardpoints from `<hardpoints>`.
    #[serde(rename = "hardpoints")]
    pub hardpoints: Option<VehicleHardpoints>,
    /// Tread settings from `<treadpoints>`.
    #[serde(rename = "treadpoints")]
    pub treadpoints: Option<VehicleTreadpoints>,
}

/// Vehicle suspension tuning from the `<suspension>` element.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VehicleSuspension {
    /// Rest length of each spring (`@springLength`).
    #[serde(rename = "@springLength")]
    pub spring_length: Option<f32>,
    /// Radius of each wheel (`@wheelRadius`).
    #[serde(rename = "@wheelRadius")]
    pub wheel_radius: Option<f32>,
    /// Spring stiffness (`@springStrength`).
    #[serde(rename = "@springStrength")]
    pub spring_strength: Option<f32>,
    /// Damping when the spring compresses (`@dampingCompression`).
    #[serde(rename = "@dampingCompression")]
    pub damping_compression: Option<f32>,
    /// Damping when the spring extends (`@dampingRelaxation`).
    #[serde(rename = "@dampingRelaxation")]
    pub damping_relaxation: Option<f32>,
    /// Maximum ground normal angle (`@normalClippingAngle`).
    #[serde(rename = "@normalClippingAngle")]
    pub normal_clipping_angle: Option<f32>,
}

/// Chassis inertia multipliers from the `<chassisInertia>` element.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChassisInertia {
    /// Inertia about the yaw axis.
    #[serde(rename = "@yaw")]
    pub yaw: Option<f32>,
    /// Inertia about the pitch axis.
    #[serde(rename = "@pitch")]
    pub pitch: Option<f32>,
    /// Inertia about the roll axis.
    #[serde(rename = "@roll")]
    pub roll: Option<f32>,
}

/// Vehicle hardpoint vectors. The engine ignores each child's element name;
/// `hardpoint` is the canonical spelling emitted by this parser.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VehicleHardpoints {
    /// Hardpoint positions, one per child element.
    #[serde(rename = "hardpoint", alias = "point", default)]
    pub points: Vec<Vector3>,
}

/// Tread tuning from the `<treadpoints>` element.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VehicleTreadpoints {
    /// Tread height (`@height`).
    #[serde(rename = "@height")]
    pub height: Option<f32>,
    /// Tread length (`@length`).
    #[serde(rename = "@length")]
    pub length: Option<f32>,
    /// Scale from vehicle speed to tread animation speed (`@velocityScalar`).
    #[serde(rename = "@velocityScalar")]
    pub velocity_scalar: Option<f32>,
    /// When `true`, the treads attach to the wheels (`@wheelAttachments`).
    #[serde(rename = "@wheelAttachments")]
    pub wheel_attachments: Option<bool>,
    /// Tread points, one per child element.
    #[serde(rename = "treadpoint", alias = "point", default)]
    pub points: Vec<Treadpoint>,
}

/// One tread point from a `<treadpoint>` element.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Treadpoint {
    /// Vertical offset of the point (`@zOffset`).
    #[serde(rename = "@zOffset")]
    pub z_offset: Option<f32>,
    /// Position of the point from the element text.
    #[serde(rename = "$text", default)]
    pub position: Vector3,
}

/// Multi-body clamshell layout from the `<clamshell>` element.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Clamshell {
    /// Upper body part from `<upper>`.
    #[serde(rename = "upper")]
    pub upper: Option<ClamshellPart>,
    /// Lower body part from `<lower>`.
    #[serde(rename = "lower")]
    pub lower: Option<ClamshellPart>,
    /// Pelvis body part from `<pelvis>`.
    #[serde(rename = "pelvis")]
    pub pelvis: Option<ClamshellPart>,
}

/// One clamshell body part with its blueprint.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ClamshellPart {
    /// Height offset of the part (`@heightOffset`).
    #[serde(rename = "@heightOffset")]
    pub height_offset: Option<f32>,
    /// Physics blueprint name from the element text.
    #[serde(rename = "$text", default)]
    pub blueprint: String,
}

/// Havok motion modes recognized by the game.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MotionType {
    /// Dynamic body that moves under simulation.
    #[default]
    Normal,
    /// Fixed body that does not move.
    Fixed,
}

impl<'de> Deserialize<'de> for MotionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.trim() {
            "normal" => Ok(Self::Normal),
            "fixed" => Ok(Self::Fixed),
            value => Err(D::Error::custom(format_args!(
                "unknown physics motion type `{value}`"
            ))),
        }
    }
}

impl Serialize for MotionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::Normal => "normal",
            Self::Fixed => "fixed",
        })
    }
}

/// Physical properties from a `.blueprint.xmb` file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Blueprint {
    /// Mass in kg.
    #[serde(rename = "mass")]
    pub mass: Option<f32>,
    /// Friction coefficient.
    #[serde(rename = "friction")]
    pub friction: Option<f32>,
    /// Restitution (bounciness).
    #[serde(rename = "restitution")]
    pub restitution: Option<f32>,
    /// Center-of-mass offset.
    #[serde(rename = "centerOfMassOffset")]
    pub center_of_mass_offset: Option<Vector3>,
    /// Linear damping.
    #[serde(rename = "linearDamping")]
    pub linear_damping: Option<f32>,
    /// Angular damping.
    #[serde(rename = "angularDamping")]
    pub angular_damping: Option<f32>,
    /// Shape reference name.
    #[serde(rename = "shape")]
    pub shape: Option<String>,
    /// Optional collision half-extents override.
    #[serde(rename = "halfExtents")]
    pub half_extents: Option<Vector3>,
}

/// A Havok collision shape from a `.shp.xmb` file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Shape {
    /// Havok version string (e.g. `"V_20200_B_20031014"`).
    #[serde(rename = "@version")]
    pub hke_version: Option<String>,
    /// Shape objects.
    #[serde(rename = "hkobject", default)]
    pub objects: Vec<HavokObject>,
}

/// A single Havok object (shape primitive).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HavokObject {
    /// Object name (e.g. `"body"`).
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Object type (e.g. `"hkBoxShape"`, `"hkConvexVerticesShape"`).
    #[serde(rename = "@type", default)]
    pub object_type: String,
    /// Parameters as key-value pairs.
    #[serde(rename = "hkparam", default)]
    pub params: Vec<HavokParam>,
}

/// A Havok parameter.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HavokParam {
    /// Parameter name (e.g. `"halfExtents"`, `"radius"`).
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Parameter type (e.g. `"hkTypeVector4"`, `"hkTypeReal"`).
    #[serde(rename = "@type", default)]
    pub param_type: String,
    /// Raw value as string.
    #[serde(rename = "$text", default)]
    pub value: String,
}

/// Parse a `.physics.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or the physics data cannot be deserialized.
pub fn parse_physics(doc: &xmb::Document) -> crate::Result<Physics> {
    let root = expect_root(doc, "physics")?;
    let physics: Physics = bdt_serde::from_node(root)?;
    Ok(physics)
}

/// Parse a `.blueprint.xmb` document.
///
/// # Errors
///
/// Returns an error if the document has no root or the blueprint cannot be deserialized.
pub fn parse_blueprint(doc: &xmb::Document) -> crate::Result<Blueprint> {
    let root = crate::node_ext::root_node(doc)?;
    let bp: Blueprint = bdt_serde::from_node(root)?;
    Ok(bp)
}

/// Parse a `.shp.xmb` document (Havok XML shapes).
///
/// # Errors
///
/// Returns an error if the document root is invalid or the shape cannot be deserialized.
pub fn parse_shape(doc: &xmb::Document) -> crate::Result<Shape> {
    let root = expect_root(doc, "hke")?;
    let shape: Shape = bdt_serde::from_node(root)?;
    Ok(shape)
}

/// Serialize a [`Physics`] back into an XMB [`Document`](xmb::Document).
///
/// # Errors
///
/// Returns an error if the physics data cannot be serialized into an XMB node.
pub fn physics_to_document(phys: &Physics) -> crate::Result<xmb::Document> {
    let node = bdt_serde::to_node("physics", phys)?;
    Ok(xmb::Document::with_root(node))
}

/// Serialize a [`Blueprint`] back into an XMB [`Document`](xmb::Document).
///
/// # Errors
///
/// Returns an error if the blueprint cannot be serialized into an XMB node.
pub fn blueprint_to_document(bp: &Blueprint) -> crate::Result<xmb::Document> {
    let node = bdt_serde::to_node("blueprint", bp)?;
    Ok(xmb::Document::with_root(node))
}

/// Serialize a [`Shape`] back into an XMB [`Document`](xmb::Document).
///
/// # Errors
///
/// Returns an error if the shape cannot be serialized into an XMB node.
pub fn shape_to_document(shape: &Shape) -> crate::Result<xmb::Document> {
    let node = bdt_serde::to_node("hke", shape)?;
    Ok(xmb::Document::with_root(node))
}
