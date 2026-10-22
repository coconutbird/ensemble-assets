//! Parser for `*.pfx.xmb` — particle effect definitions.
//!
//! A PFX document has a `ParticleEffect` root with one `ParticleEmitter` child for each emitter.
//! The parser reads the authored values, applies the retail defaults and changes the authored
//! enum strings into typed values. It does not resolve textures or create renderer state.
//!
//! The parser matches element and attribute names without case. It matches enum strings with
//! case.

use alloc::borrow::{Cow, ToOwned};
use alloc::string::String;
use alloc::vec::Vec;

use bdt::{Node, Variant};

mod runtime;
#[cfg(test)]
mod tests;

pub use runtime::{
    ParticleColorDefinition, ParticleColorKey, ParticleColorKind, ParticleColorProgression,
    ParticleEmitterShape, ParticleEmitterShapeKind, ParticleEmitterTiming, ParticleForceDefinition,
    ParticleMagnetDefinition, ParticleMagnetKind, ParticlePaletteEntry, ParticleRuntimeDefinition,
    ParticleScalarKey, ParticleScalarProgression, ParticleScalarProperty, ParticleTrailEmission,
    ParticleTrailUv, ParticleVarying, ParticleVectorProperty,
};

/// A decoded PFX effect.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEffect {
    /// Authored effect name/path.
    pub name: String,
    /// Emitters in authored order.
    pub emitters: Vec<ParticleEmitter>,
}

/// Parse a PFX document into a [`ParticleEffect`].
///
/// # Errors
///
/// Returns an error for the wrong root element, a missing `EmitterData` section or an
/// unsupported authored enum value. The error is the error of the first emitter that fails.
pub fn parse(document: &xmb::Document) -> Result<ParticleEffect, PfxError> {
    let root = document.root().ok_or(PfxError::MissingRoot)?;
    if !root.name.eq_ignore_ascii_case("ParticleEffect") {
        return Err(PfxError::UnexpectedRoot(root.name.clone()));
    }
    let name = attribute(root, "Name").map_or_else(String::new, Cow::into_owned);
    let emitters = root
        .children
        .iter()
        .filter(|node| node.name.eq_ignore_ascii_case("ParticleEmitter"))
        .map(ParticleEmitter::from_node)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ParticleEffect { name, emitters })
}

/// The authored state of one PFX emitter.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitter {
    /// Authored emitter name.
    pub name: String,
    /// Particle Editor preview switch. The retail loader deliberately ignores
    /// this attribute and initializes runtime activity from emitter timing.
    pub editor_active: bool,
    /// Renderable geometry or a nested PFX reference.
    pub kind: ParticleEmitterKind,
    /// Maximum live-particle budget.
    pub max_particles: u32,
    /// Requests back-to-front sorting.
    pub sort_particles: bool,
    /// Static material/texture references.
    pub material: ParticleMaterialDefinition,
    /// Authored timing, shape, progression, force, and magnet state.
    pub runtime: ParticleRuntimeDefinition,
}

impl ParticleEmitter {
    fn from_node(node: &Node) -> Result<Self, PfxError> {
        let name = attribute(node, "Name").map_or_else(String::new, Cow::into_owned);
        let editor_active = attribute(node, "Active")
            .and_then(|value| parse_bool(&value))
            .unwrap_or(true);
        let emitter_data = child(node, "EmitterData").ok_or_else(|| PfxError::MissingSection {
            emitter: name.clone(),
            section: "EmitterData",
        })?;
        let particle_type =
            child_text(emitter_data, "ParticleType").unwrap_or(Cow::Borrowed("eBillBoard"));
        let kind = parse_emitter_kind(node, emitter_data, &particle_type)?;
        let runtime = ParticleRuntimeDefinition::from_node(node, emitter_data, &kind)?;
        let blend_value =
            child_text(emitter_data, "BlendMode").unwrap_or(Cow::Borrowed("eAlphaBlend"));
        let blend = parse_blend(&blend_value)?;
        let texture_data = child(node, "TextureData");
        let diffuse = ["Diffuse", "Diffuse2", "Diffuse3"].map(|name| {
            texture_data
                .and_then(|textures| child(textures, name))
                .map_or_else(ParticleTextureDefinition::default, parse_texture_definition)
        });
        let intensity = texture_data
            .and_then(|textures| child(textures, "Intensity"))
            .map_or_else(ParticleTextureDefinition::default, parse_texture_definition);
        let layer_1_to_2_value = texture_data
            .and_then(|textures| child_text(textures, "DiffuseLayer1To2BlendMode"))
            .unwrap_or(Cow::Borrowed("eBlendMultiply"));
        let layer_1_to_2 = parse_layer_blend(&layer_1_to_2_value)?;
        let layer_2_to_3_value = texture_data
            .and_then(|textures| child_text(textures, "DiffuseLayer2To3BlendMode"))
            .unwrap_or(Cow::Borrowed("eBlendMultiply"));
        let layer_2_to_3 = parse_layer_blend(&layer_2_to_3_value)?;
        let color_data = child(node, "ColorData");
        let corner_colors = [
            "ColorVertex1",
            "ColorVertex2",
            "ColorVertex3",
            "ColorVertex4",
        ]
        .map(|field| packed_color_child(color_data, field).unwrap_or([1.0; 4]));
        let light_volume = matches!(blend, BlendMode::Alpha | BlendMode::PremultipliedAlpha)
            && child_text(emitter_data, "LightBuffer")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(true);
        Ok(Self {
            name,
            editor_active,
            kind,
            max_particles: child_text(emitter_data, "MaxParticles")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1000),
            sort_particles: child_text(emitter_data, "SortParticles")
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            material: ParticleMaterialDefinition {
                diffuse,
                intensity,
                layer_1_to_2,
                layer_2_to_3,
                blend,
                soft_particles: child_text(emitter_data, "SoftParticles")
                    .and_then(|value| parse_bool(&value))
                    .unwrap_or(false),
                light_volume,
                light_volume_intensity: child_text(emitter_data, "LightBufferIntensityScale")
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1.0),
                soft_fade_scale: child_text(emitter_data, "SoftParticleFadeRange")
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1.0_f32)
                    .clamp(0.1, 2.0),
                corner_colors,
            },
            runtime,
        })
    }
}

/// Whether an emitter draws geometry or recursively starts another PFX.
#[derive(Clone, Debug, PartialEq)]
pub enum ParticleEmitterKind {
    /// Particle geometry of the given type.
    Render(ParticleType),
    /// Nested effect path authored by an `ePFX` emitter.
    NestedEffect(String),
}

/// Particle geometry type from the `ParticleType` and `BeamAlignmentType` fields.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ParticleType {
    /// Camera-facing quad (`eBillBoard`).
    #[default]
    Billboard,
    /// Horizontal world-space quad (`eUpfacing`).
    UpFacing,
    /// Camera-facing quad constrained to an authored axis (`eOrientedAxialBillboard`).
    OrientedAxial,
    /// Quad whose surface normal follows particle velocity (`eVelocityAligned`).
    VelocityAligned,
    /// Camera-facing beam (`eBeam` with `eBeamAlignToCamera`).
    Beam,
    /// Beam whose width follows world up (`eBeam` with `eBeamAlignVertical`).
    BeamVertical,
    /// Beam whose width is horizontal (`eBeam` with `eBeamAlignHorizontal`).
    BeamHorizontal,
    /// Camera-facing trail (`eTrail`).
    Trail,
    /// Two perpendicular trail ribbons (`eTrailCross`).
    TrailCross,
    /// Horizontal terrain patch (`eTerrainPatch`).
    TerrainPatch,
}

/// Framebuffer blend family from the `BlendMode` field.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlendMode {
    /// `eAlphaBlend`.
    #[default]
    Alpha,
    /// `eAdditive`.
    Additive,
    /// `ePremultipliedAlpha`.
    PremultipliedAlpha,
    /// `eSubtractive`.
    Subtractive,
    /// `eDistortion`.
    Distortion,
}

/// Diffuse layer blend operation from the `DiffuseLayer*BlendMode` fields.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LayerBlendMode {
    /// `eBlendMultiply`.
    #[default]
    Multiply,
    /// `eBlendAlpha`.
    Alpha,
}

/// Material description with texture paths.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleMaterialDefinition {
    /// Diffuse map definitions in layer order.
    pub diffuse: [ParticleTextureDefinition; 3],
    /// Optional intensity map definition.
    pub intensity: ParticleTextureDefinition,
    /// Diffuse layer-one/two blend operation.
    pub layer_1_to_2: LayerBlendMode,
    /// Diffuse layer-two/three blend operation.
    pub layer_2_to_3: LayerBlendMode,
    /// Framebuffer blend family.
    pub blend: BlendMode,
    /// Whether scene-depth fading is requested.
    pub soft_particles: bool,
    /// Whether the light volume modulates particle RGB.
    pub light_volume: bool,
    /// Authored light-volume multiplier.
    pub light_volume_intensity: f32,
    /// Default depth-delta multiplier copied to emitted instances.
    pub soft_fade_scale: f32,
    /// Quad-corner modulation colors decoded from A8R8G8B8.
    pub corner_colors: [[f32; 4]; 4],
}

/// Weighted texture stages and UV animation for one material layer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParticleTextureDefinition {
    /// Texture stages in authored weighted-selection order.
    pub stages: Vec<ParticleTextureStage>,
    /// Authored UV/frame animation controls.
    pub uv_animation: ParticleUvAnimation,
}

/// One weighted texture-array entry.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleTextureStage {
    /// PFX texture reference.
    pub path: String,
    /// Relative random-selection weight.
    pub weight: f32,
}

/// PFX UV animation controls needed to populate instance UV rectangles.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParticleUvAnimation {
    /// Enables sprite-sheet frame selection.
    pub enabled: bool,
    /// Randomizes initial horizontal scroll.
    pub random_scroll_u: bool,
    /// Randomizes initial vertical scroll.
    pub random_scroll_v: bool,
    /// Number of sprite-sheet frames.
    pub frame_count: u32,
    /// Authored frame width in texture pixels.
    pub frame_width: f32,
    /// Authored frame height in texture pixels.
    pub frame_height: f32,
    /// Playback rate.
    pub frames_per_second: f32,
    /// Horizontal UV scroll per second.
    pub scroll_u: f32,
    /// Vertical UV scroll per second.
    pub scroll_v: f32,
}

/// PFX decoding error.
#[derive(Debug)]
pub enum PfxError {
    /// Document had no root node.
    MissingRoot,
    /// Document root was not `ParticleEffect`.
    UnexpectedRoot(String),
    /// Required emitter section was absent.
    MissingSection {
        /// Emitter name.
        emitter: String,
        /// Required section name.
        section: &'static str,
    },
    /// An authored enum value is not a known value.
    UnsupportedValue {
        /// Field name.
        field: &'static str,
        /// Authored enum string.
        value: String,
    },
}

impl core::fmt::Display for PfxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingRoot => f.write_str("particle effect XMB has no root node"),
            Self::UnexpectedRoot(name) => write!(f, "unexpected particle effect root '{name}'"),
            Self::MissingSection { emitter, section } => {
                write!(f, "particle emitter '{emitter}' is missing {section}")
            }
            Self::UnsupportedValue { field, value } => {
                write!(f, "unsupported particle {field} value '{value}'")
            }
        }
    }
}

impl core::error::Error for PfxError {}

fn parse_emitter_kind(
    emitter: &Node,
    emitter_data: &Node,
    value: &str,
) -> Result<ParticleEmitterKind, PfxError> {
    let particle_type = match value {
        "eBillBoard" => ParticleType::Billboard,
        "eUpfacing" => ParticleType::UpFacing,
        "eOrientedAxialBillboard" => ParticleType::OrientedAxial,
        "eVelocityAligned" => ParticleType::VelocityAligned,
        "eBeam" => parse_beam_type(emitter_data)?,
        "eTrail" => ParticleType::Trail,
        "eTrailCross" => ParticleType::TrailCross,
        "eTerrainPatch" => ParticleType::TerrainPatch,
        "ePFX" => {
            let path =
                descendant_text(emitter, "PFXFilePath").map_or_else(String::new, Cow::into_owned);
            return Ok(ParticleEmitterKind::NestedEffect(path));
        }
        _ => return unsupported("type", value),
    };
    Ok(ParticleEmitterKind::Render(particle_type))
}

fn parse_beam_type(emitter_data: &Node) -> Result<ParticleType, PfxError> {
    let alignment = child_text(emitter_data, "BeamAlignmentType")
        .unwrap_or(Cow::Borrowed("eBeamAlignToCamera"));
    match alignment.as_ref() {
        "eBeamAlignToCamera" => Ok(ParticleType::Beam),
        "eBeamAlignVertical" => Ok(ParticleType::BeamVertical),
        "eBeamAlignHorizontal" => Ok(ParticleType::BeamHorizontal),
        value => unsupported("beam alignment", value),
    }
}

fn parse_blend(value: &str) -> Result<BlendMode, PfxError> {
    match value {
        "eAlphaBlend" => Ok(BlendMode::Alpha),
        "eAdditive" => Ok(BlendMode::Additive),
        "ePremultipliedAlpha" => Ok(BlendMode::PremultipliedAlpha),
        "eSubtractive" => Ok(BlendMode::Subtractive),
        "eDistortion" => Ok(BlendMode::Distortion),
        _ => unsupported("blend mode", value),
    }
}

fn parse_layer_blend(value: &str) -> Result<LayerBlendMode, PfxError> {
    match value {
        "eBlendMultiply" => Ok(LayerBlendMode::Multiply),
        "eBlendAlpha" => Ok(LayerBlendMode::Alpha),
        _ => unsupported("layer blend", value),
    }
}

fn unsupported<T>(field: &'static str, value: &str) -> Result<T, PfxError> {
    Err(PfxError::UnsupportedValue {
        field,
        value: value.to_owned(),
    })
}

fn parse_texture_definition(node: &Node) -> ParticleTextureDefinition {
    let stages = child(node, "Textures")
        .map(|textures| {
            textures
                .children
                .iter()
                .filter(|stage| stage.name.eq_ignore_ascii_case("Stage"))
                .filter_map(|stage| {
                    let path = child_text(stage, "file")?.into_owned();
                    let weight = child_text(stage, "weight")
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(1.0);
                    Some(ParticleTextureStage { path, weight })
                })
                .collect()
        })
        .unwrap_or_default();
    let uv = child(node, "UVAnimation");
    ParticleTextureDefinition {
        stages,
        uv_animation: ParticleUvAnimation {
            enabled: uv
                .and_then(|uv| child_text(uv, "UVAnimationEnabled"))
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            random_scroll_u: uv
                .and_then(|uv| child_text(uv, "UseRandomScrollOffsetU"))
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            random_scroll_v: uv
                .and_then(|uv| child_text(uv, "UseRandomScrollOffsetV"))
                .and_then(|value| parse_bool(&value))
                .unwrap_or(false),
            frame_count: numeric_child(uv, "NumFrames"),
            frame_width: numeric_child(uv, "FrameWidth"),
            frame_height: numeric_child(uv, "FrameHeight"),
            frames_per_second: uv.map_or(0.0, |uv| {
                numeric_child::<f32>(Some(uv), "FramesPerSecond").max(1.0)
            }),
            scroll_u: numeric_child(uv, "ScrollU"),
            scroll_v: numeric_child(uv, "ScrollV"),
        },
    }
}

fn numeric_child<T>(node: Option<&Node>, name: &str) -> T
where
    T: core::str::FromStr + Default,
{
    node.and_then(|node| child_text(node, name))
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
}

fn child<'a>(node: &'a Node, name: &str) -> Option<&'a Node> {
    node.children
        .iter()
        .find(|child| child.name.eq_ignore_ascii_case(name))
}

fn child_text<'a>(node: &'a Node, name: &str) -> Option<Cow<'a, str>> {
    child(node, name).and_then(node_text)
}

fn descendant_text<'a>(node: &'a Node, name: &str) -> Option<Cow<'a, str>> {
    if node.name.eq_ignore_ascii_case(name) {
        return node_text(node);
    }
    node.children
        .iter()
        .find_map(|child| descendant_text(child, name))
}

fn node_text(node: &Node) -> Option<Cow<'_, str>> {
    match &node.text {
        Variant::Null => None,
        Variant::String(value) => Some(Cow::Borrowed(value.as_str())),
        _ => Some(Cow::Owned(node.text_string())),
    }
}

fn attribute<'a>(node: &'a Node, name: &str) -> Option<Cow<'a, str>> {
    let attribute = node
        .attributes
        .iter()
        .find(|attribute| attribute.name.eq_ignore_ascii_case(name))?;
    match &attribute.value {
        Variant::Null => None,
        Variant::String(value) => Some(Cow::Borrowed(value.as_str())),
        _ => Some(Cow::Owned(attribute.value_string())),
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    if value.eq_ignore_ascii_case("true") || value == "1" {
        Some(true)
    } else if value.eq_ignore_ascii_case("false") || value == "0" {
        Some(false)
    } else {
        None
    }
}

fn packed_color_child(node: Option<&Node>, name: &str) -> Option<[f32; 4]> {
    let value = node.and_then(|node| child_text(node, name))?;
    let packed = value
        .parse::<u32>()
        .ok()
        .or_else(|| value.parse::<i32>().ok().map(i32::cast_unsigned))?;
    let channel =
        |shift: u32| f32::from(u8::try_from((packed >> shift) & 0xff_u32).unwrap_or(0)) / 255.0;
    Some([channel(16), channel(8), channel(0), channel(24)])
}
