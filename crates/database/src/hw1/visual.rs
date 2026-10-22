//! Parser for `.vis.xmb` — visual definitions.
//!
//! A visual file defines the models, animations, attachments, and effects for
//! a game entity. The root element is `<visual>` with `<model>` children.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::expect_root;

/// A complete visual definition from a `.vis.xmb` file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Visual {
    /// Default model name (from `defaultmodel` attribute on root).
    #[serde(rename = "@defaultmodel")]
    pub default_model: Option<String>,
    /// Named models (e.g. `"Default"`, `"Turret"`, `"Wheel"`).
    #[serde(rename = "model", default)]
    pub models: Vec<Model>,
    /// Root-level model-selection logic.
    #[serde(rename = "logic")]
    pub logic: Option<Logic>,
}

/// A named model within a visual.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Model {
    /// Model name (e.g. `"Default"`, `"Turret"`).
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Component (contains asset refs and attachments).
    #[serde(rename = "component")]
    pub component: Option<Component>,
    /// Animations.
    #[serde(rename = "anim", default)]
    pub anims: Vec<Anim>,
}

/// Component data: model files and attachment points.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Component {
    /// Direct asset references in the component.
    #[serde(rename = "asset", default)]
    pub assets: Vec<Asset>,
    /// Attachment points (model refs, particle effects, etc.).
    #[serde(rename = "attach", default)]
    pub attachments: Vec<Attachment>,
    /// Impact/board/launch points.
    #[serde(rename = "point", default)]
    pub points: Vec<Point>,
    /// Logic-switched asset variants (tech upgrades).
    #[serde(rename = "logic")]
    pub logic: Option<Logic>,
}

/// An asset reference (model or animation file).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Asset {
    /// Asset type: `"Model"` or `"Anim"`.
    #[serde(rename = "@type", default)]
    pub asset_type: String,
    /// File path (relative, no `art\` prefix, no extension).
    #[serde(rename = "file")]
    pub file: Option<String>,
    /// Damage model file path.
    #[serde(rename = "damagefile")]
    pub damage_file: Option<String>,
    /// Weight for random selection.
    #[serde(rename = "weight")]
    pub weight: Option<i32>,
    /// Per-texture-channel UV offsets.
    #[serde(rename = "uvOffset", default)]
    pub uv_offsets: Vec<UvOffset>,
    /// Animation event tags attached to this asset.
    #[serde(rename = "tag", default)]
    pub tags: Vec<VisualTag>,
    /// Optional opacity progression.
    #[serde(rename = "progression")]
    pub progression: Option<FloatProgression>,
}

/// A model texture-coordinate offset.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UvOffset {
    /// Texture-coordinate channel (`@channel`).
    #[serde(rename = "@channel")]
    pub channel: Option<u32>,
    /// U offset (`@uOfs`).
    #[serde(rename = "@uOfs")]
    pub u_offset: Option<f32>,
    /// V offset (`@vOfs`).
    #[serde(rename = "@vOfs")]
    pub v_offset: Option<f32>,
}

/// An attachment point.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Attachment {
    /// Attachment type: `"ModelRef"`, `"ParticleFile"`, `"TerrainEffect"`.
    #[serde(rename = "@type", default)]
    pub attach_type: String,
    /// Attachment name / reference.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Target bone.
    #[serde(rename = "@tobone")]
    pub to_bone: Option<String>,
    /// Source bone.
    #[serde(rename = "@frombone")]
    pub from_bone: Option<String>,
    /// Whether to sync animations.
    #[serde(rename = "@syncanims")]
    pub sync_anims: Option<bool>,
    /// Discard the source bone's orientation when attaching.
    #[serde(rename = "@disregardorient")]
    pub disregard_orient: Option<bool>,
}

/// A point on a component (impact, board, launch, pickup).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Point {
    /// Point type: `"Impact"`, `"Board"`, `"Launch"`, `"Pickup"`.
    #[serde(rename = "@pointType", default)]
    pub point_type: String,
    /// Bone name.
    #[serde(rename = "@bone")]
    pub bone: Option<String>,
    /// Point data (material type, e.g. `"Metal"`).
    #[serde(rename = "@pointData", alias = "@pointdata")]
    pub point_data: Option<String>,
}

/// Logic switch for tech-based model variants.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Logic {
    /// Logic type: `"Tech"`.
    #[serde(rename = "@type", default)]
    pub logic_type: String,
    /// Logic data entries (one per tech level).
    #[serde(rename = "logicdata", default)]
    pub entries: Vec<LogicEntry>,
}

/// A single logic entry (maps a tech value to an asset).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LogicEntry {
    /// Tech value that activates this variant (empty = default).
    #[serde(rename = "@value", default)]
    pub value: String,
    /// Model reference name.
    #[serde(rename = "@modelref")]
    pub model_ref: Option<String>,
    /// Weight.
    #[serde(rename = "@weight")]
    pub weight: Option<i32>,
    /// The asset selected for this variant.
    #[serde(rename = "asset")]
    pub asset: Option<Asset>,
    /// Nested selection logic for this result model.
    #[serde(rename = "logic")]
    pub logic: Option<Logic>,
}

/// An animation definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Anim {
    /// Animation type: `"Idle"`, `"Walk"`, `"Death"`, etc.
    #[serde(rename = "@type", default)]
    pub anim_type: String,
    /// Exit action: `"Loop"`, `"Freeze"`, `"Transition"`.
    #[serde(rename = "@exitAction")]
    pub exit_action: Option<String>,
    /// Tween time.
    #[serde(rename = "@tweenTime")]
    pub tween_time: Option<i32>,
    /// Tween-to animation name.
    #[serde(rename = "@tweenToAnimation")]
    pub tween_to_animation: Option<String>,
    /// Asset references (animation files).
    #[serde(rename = "asset", default)]
    pub assets: Vec<Asset>,
    /// Attachments active during this animation.
    #[serde(rename = "attach", default)]
    pub attachments: Vec<Attachment>,
    /// Meshes enabled only while this animation is active.
    #[serde(rename = "meshEnable", default)]
    pub enabled_meshes: Vec<String>,
}

/// An animation event tag. Attribute meanings depend on `tag_type`, matching
/// the tagged union used by `BProtoVisualTag`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VisualTag {
    /// Tag type, for example `Sound` or `Particle` (`@type`).
    #[serde(rename = "@type", default)]
    pub tag_type: String,
    /// Event time in the animation (`@position`).
    #[serde(rename = "@position")]
    pub position: Option<f32>,
    /// Referenced asset or cue name (`@name`).
    #[serde(rename = "@name")]
    pub name: Option<String>,
    /// Bone that the tag attaches to (`@tobone`).
    #[serde(rename = "@tobone")]
    pub to_bone: Option<String>,
    /// Lifespan of the spawned effect (`@lifespan`).
    #[serde(rename = "@lifespan")]
    pub lifespan: Option<f32>,
    /// First force value (`@force`).
    #[serde(rename = "@force")]
    pub force: Option<f32>,
    /// Second force value (`@force2`).
    #[serde(rename = "@force2")]
    pub force2: Option<f32>,
    /// Fire only when the object is visible (`@checkvisible`).
    #[serde(rename = "@checkvisible", alias = "@checkVisible")]
    pub check_visible: Option<bool>,
    /// Fire only when the object is out of fog of war (`@checkFOW`).
    #[serde(rename = "@checkFOW")]
    pub check_fow: Option<bool>,
    /// Fire only when the object is selected (`@checkSelected`).
    #[serde(rename = "@checkSelected")]
    pub check_selected: Option<bool>,
    /// End value (`@end`).
    #[serde(rename = "@end")]
    pub end: Option<f32>,
    /// Start value (`@start`).
    #[serde(rename = "@start")]
    pub start: Option<f32>,
    /// Lock the effect to the ground (`@lockToGround`).
    #[serde(rename = "@lockToGround")]
    pub lock_to_ground: Option<bool>,
    /// Attach the effect to the object (`@attach`).
    #[serde(rename = "@attach")]
    pub attach: Option<bool>,
    /// Mini-language used by terrain-alpha, decal, and UV-offset tags.
    #[serde(rename = "@userData")]
    pub user_data: Option<String>,
    /// Rumble type for the left motor (`@leftRumbleType`).
    #[serde(rename = "@leftRumbleType")]
    pub left_rumble_type: Option<String>,
    /// Rumble type for the right motor (`@rightRumbleType`).
    #[serde(rename = "@rightRumbleType")]
    pub right_rumble_type: Option<String>,
    /// Loop the tag effect (`@loop`).
    #[serde(rename = "@loop")]
    pub looped: Option<bool>,
    /// Discard the bone orientation when attaching (`@disregardorient`).
    #[serde(rename = "@disregardorient")]
    pub disregard_orient: Option<bool>,
}

/// An opacity curve applied to a visual asset.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FloatProgression {
    /// Curve stages (`<Stages>`).
    #[serde(rename = "Stages")]
    pub stages: Option<FloatProgressionStages>,
    /// Repeat the curve (`<Loop>`).
    #[serde(rename = "Loop")]
    pub looped: Option<bool>,
    /// Number of curve cycles (`<Cycles>`).
    #[serde(rename = "Cycles")]
    pub cycles: Option<f32>,
}

/// Wrapper for the `<Stages>` list of a float progression.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FloatProgressionStages {
    /// The `<Stage>` entries, in order.
    #[serde(rename = "Stage", default)]
    pub stages: Vec<FloatProgressionStage>,
}

/// One stage of a float progression curve.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FloatProgressionStage {
    /// Value at this stage (`<Value>`).
    #[serde(rename = "Value")]
    pub value: Option<f32>,
    /// Position of the stage on the curve (`<Alpha>`).
    #[serde(rename = "Alpha")]
    pub alpha: Option<f32>,
    /// Random variance added to the value (`<ValueVariance>`).
    #[serde(rename = "ValueVariance")]
    pub value_variance: Option<f32>,
}

/// Parse a visual definition from a `.vis.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or the visual cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Visual> {
    let root = expect_root(doc, "visual")?;
    let vis: Visual = bdt_serde::from_node(root)?;
    Ok(vis)
}

/// Serialize a [`Visual`] back into an XMB [`Document`](xmb::Document).
///
/// # Errors
///
/// Returns an error if the visual cannot be serialized into an XMB node.
pub fn to_document(vis: &Visual) -> crate::Result<xmb::Document> {
    let node = bdt_serde::to_node("visual", vis)?;
    Ok(xmb::Document::with_root(node))
}
