//! Parser for `hpbars.xml.xmb`: the HP bar, veterancy, pie progress, bobble head and building
//! strength definitions of the unit UI.
//!
//! `BDatabase_setupHPBars` @ `0x1401ece90` loads the file. The root must be
//! `<HPBarDefinition>` (the compare ignores case), else the engine logs `BHPBar::loadXML: this
//! is not a hpbar definition file.`. Each child goes to the list of its kind, so the index in
//! a list is the engine ID of the definition:
//!
//! - `<HPBar>`: `BHPBar_loadHPBarDef` @ `0x140040320`.
//! - `<ColorStages>`: `BHPBar_loadColorStages` @ `0x140042230`.
//! - `<VeterancyBar>`: `BHPBar_loadVeterancyBarDef` @ `0x140040fb0`.
//! - `<PieProgress>`: `BHPBar_loadPieProgressDef` @ `0x140040a80`.
//! - `<BobbleHead>`: `BHPBar_loadBobbleHeadDef` @ `0x140041330`.
//! - `<BuildingStrength>`: `BHPBar_loadBuildingStrengthDef` @ `0x140041af0`.
//!
//! The engine reads numbers with `BXMLNode_getTextAsFloat` and
//! `BXMLAttribute_getValueAsFloat`. The shipped file writes many of them as C float literals,
//! for example `0.38f`. A tag that is absent keeps the default of the reset function of the
//! definition. The field documents give these defaults.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use super::RgbColor;
use crate::node_ext::expect_root;

/// All definitions from `hpbars.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HpBarDefinition {
    /// Color stage sets. The engine uses the set named `DEFAULT` (see
    /// [`HpBarDefinition::default_color_stages`]).
    #[serde(rename = "ColorStages", default)]
    pub color_stages: Vec<HpBarColorStages>,
    /// HP bar definitions. The index is the HP bar ID that `BDatabase_getHPBarIDFromName`
    /// @ `0x1401ed9b0` gives.
    #[serde(rename = "HPBar", default)]
    pub hp_bars: Vec<HpBar>,
    /// Veterancy bar definitions.
    #[serde(rename = "VeterancyBar", default)]
    pub veterancy_bars: Vec<VeterancyBar>,
    /// Pie progress definitions (ability recharge and capture progress).
    #[serde(rename = "PieProgress", default)]
    pub pie_progress: Vec<PieProgress>,
    /// Bobble head definitions (hero portraits).
    #[serde(rename = "BobbleHead", default)]
    pub bobble_heads: Vec<BobbleHead>,
    /// Building strength definitions.
    #[serde(rename = "BuildingStrength", default)]
    pub building_strengths: Vec<BuildingStrength>,
}

impl HpBarDefinition {
    /// Find the ID of the HP bar `name`, as `BDatabase_getHPBarIDFromName` @ `0x1401ed9b0` does.
    ///
    /// The compare ignores case. The first match wins.
    #[must_use]
    pub fn hp_bar_id(&self, name: &str) -> Option<usize> {
        self.hp_bars
            .iter()
            .position(|bar| bar.name.eq_ignore_ascii_case(name))
    }

    /// Find the color stage set that the engine uses for all bars.
    ///
    /// `BDatabase_getDefaultHPBarColorStages` @ `0x1401eda50` gives the first set named
    /// `DEFAULT` (the compare ignores case). `BHPBar_initScenario` @ `0x14003e9c0` keeps it.
    #[must_use]
    pub fn default_color_stages(&self) -> Option<&HpBarColorStages> {
        self.color_stages
            .iter()
            .find(|stages| stages.name.eq_ignore_ascii_case("DEFAULT"))
    }
}

/// One `<HPBar>`: the textures and the size of an HP bar, and the texture rectangles of its
/// hitpoint, shield and ammo parts.
///
/// The reset function `BHPBarDef_reset` @ `0x14003e1e0` sets empty textures, a zero size and
/// zero rectangles.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HpBar {
    /// Definition name, for example `"UNSC_Health"`. Objects and squads refer to it in their
    /// `<HPBar>` tag.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Bar texture path without an extension, for example
    /// `ui\flash\HUD\hud_hitpntbar\unsc_unit_hptbar\unsc_unit1`.
    #[serde(rename = "Texture", default)]
    pub texture: String,
    /// Mask texture path without an extension.
    #[serde(rename = "MaskTexture", default)]
    pub mask_texture: String,
    /// Bar width in world units. Default `0`.
    #[serde(rename = "SizeX", default)]
    pub size_x: f32,
    /// Bar height in world units. Default `0`.
    #[serde(rename = "SizeY", default)]
    pub size_y: f32,
    /// Texture rectangle of the hitpoint part.
    #[serde(rename = "HitpointUV")]
    pub hitpoint_uv: Option<HpBarUv>,
    /// Texture rectangle of the shield part.
    #[serde(rename = "ShieldUV")]
    pub shield_uv: Option<HpBarUv>,
    /// Texture rectangle of the ammo part.
    #[serde(rename = "AmmoUV")]
    pub ammo_uv: Option<HpBarUv>,
}

/// A texture rectangle of an HP bar part (`<HitpointUV>`, `<ShieldUV>`, `<AmmoUV>`).
///
/// The shipped file writes empty attributes (`u1=""`) for a part that the bar does not have.
/// The engine keeps `0` for an empty or absent attribute and stores the rectangle as
/// `(u1, u2, v1, v2)`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
pub struct HpBarUv {
    /// Left texture coordinate.
    #[serde(
        rename = "@u1",
        default,
        deserialize_with = "super::types::deserialize_optional_float_text",
        serialize_with = "super::types::serialize_optional_float_text"
    )]
    pub u1: Option<f32>,
    /// Right texture coordinate.
    #[serde(
        rename = "@u2",
        default,
        deserialize_with = "super::types::deserialize_optional_float_text",
        serialize_with = "super::types::serialize_optional_float_text"
    )]
    pub u2: Option<f32>,
    /// Top texture coordinate.
    #[serde(
        rename = "@v1",
        default,
        deserialize_with = "super::types::deserialize_optional_float_text",
        serialize_with = "super::types::serialize_optional_float_text"
    )]
    pub v1: Option<f32>,
    /// Bottom texture coordinate.
    #[serde(
        rename = "@v2",
        default,
        deserialize_with = "super::types::deserialize_optional_float_text",
        serialize_with = "super::types::serialize_optional_float_text"
    )]
    pub v2: Option<f32>,
}

impl HpBarUv {
    /// Give the rectangle as `[u1, v1, u2, v2]` if all four coordinates have a value.
    ///
    /// `None` means that the bar does not have this part.
    #[must_use]
    pub fn rect(&self) -> Option<[f32; 4]> {
        Some([self.u1?, self.v1?, self.u2?, self.v2?])
    }
}

/// One `<ColorStages>` set: the colors of the hitpoint, ammo and shield bars by fill ratio.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HpBarColorStages {
    /// Set name. The engine uses the set `DEFAULT`.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Stages of the hitpoint bar.
    #[serde(rename = "Hitpoint", default)]
    pub hitpoint: HpBarStageList,
    /// Stages of the ammo bar.
    #[serde(rename = "Ammo", default)]
    pub ammo: HpBarStageList,
    /// Stages of the shield bar.
    #[serde(rename = "Shield", default)]
    pub shield: HpBarStageList,
}

/// The stage list of one bar kind, in document order.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HpBarStageList {
    /// The stages. `BHPBar_renderSquadHPBar` @ `0x140044080` uses the first stage whose
    /// `value` is not less than the fill ratio.
    #[serde(rename = "Stage", default)]
    pub stages: Vec<HpBarColorStage>,
}

/// One `<Stage value intensity>r g b</Stage>`.
///
/// `BHPBar_loadColorStage` @ `0x140042070` reads it. All values default to `0`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
pub struct HpBarColorStage {
    /// Highest fill ratio (`0` to `1`) of the stage.
    #[serde(rename = "@value", default)]
    pub value: f32,
    /// Color intensity.
    #[serde(rename = "@intensity", default)]
    pub intensity: f32,
    /// Color. `BColor_parseText` @ `0x1406875d0` reads `r g b` (alpha `255`) or `a r g b`.
    /// This field reads only `r g b`, the form of the shipped file.
    #[serde(rename = "$text", default)]
    pub color: RgbColor,
}

/// One `<VeterancyBar>`: the veterancy star of a squad.
///
/// `BVeterancyBarDef_reset` @ `0x14003e510` sets empty textures, a zero size and a zero offset.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VeterancyBar {
    /// Definition name, for example `"UNSC_Star"`.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Star texture path.
    #[serde(rename = "Texture", default)]
    pub texture: String,
    /// Star width. Default `0`.
    #[serde(rename = "SizeX", default)]
    pub size_x: f32,
    /// Star height. Default `0`.
    #[serde(rename = "SizeY", default)]
    pub size_y: f32,
    /// Horizontal offset. Default `0`.
    #[serde(rename = "OffsetX", default)]
    pub offset_x: f32,
    /// Vertical offset. Default `0`.
    #[serde(rename = "OffsetY", default)]
    pub offset_y: f32,
    /// Background texture path. The engine does not read this tag.
    #[serde(rename = "BackgroundTexture")]
    pub background_texture: Option<String>,
    /// Background width. The engine does not read this tag.
    #[serde(rename = "BackgroundSizeX")]
    pub background_size_x: Option<f32>,
    /// Background height. The engine does not read this tag.
    #[serde(rename = "BackgroundSizeY")]
    pub background_size_y: Option<f32>,
    /// Background horizontal offset. The engine does not read this tag.
    #[serde(rename = "BackgroundOffsetX")]
    pub background_offset_x: Option<f32>,
    /// Background vertical offset. The engine does not read this tag.
    #[serde(rename = "BackgroundOffsetY")]
    pub background_offset_y: Option<f32>,
}

/// One `<PieProgress>`: a circular progress indicator.
///
/// `BPieProgressDef_reset` @ `0x14003e290` sets empty textures, the sizes to `1` and the
/// offsets to `0`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PieProgress {
    /// Definition name, for example `"UNSC_Ability"`.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Background texture path.
    #[serde(rename = "BackgroundTexture", default)]
    pub background_texture: String,
    /// Mask texture path.
    #[serde(rename = "MaskTexture", default)]
    pub mask_texture: String,
    /// Pie width. Default `1`.
    #[serde(rename = "SizeX", default = "one")]
    pub size_x: f32,
    /// Pie height. Default `1`.
    #[serde(rename = "SizeY", default = "one")]
    pub size_y: f32,
    /// Pie horizontal offset. Default `0`.
    #[serde(rename = "OffsetX", default)]
    pub offset_x: f32,
    /// Pie vertical offset. Default `0`.
    #[serde(rename = "OffsetY", default)]
    pub offset_y: f32,
    /// Fill the pie instead of emptying it. Default `false`.
    #[serde(
        rename = "Fill",
        default,
        deserialize_with = "super::types::deserialize_text_bool"
    )]
    pub fill: bool,
    /// Turn clockwise. Default `false`.
    #[serde(
        rename = "Clockwise",
        default,
        deserialize_with = "super::types::deserialize_text_bool"
    )]
    pub clockwise: bool,
    /// Background width. Default `1`.
    #[serde(rename = "BackgroundSizeX", default = "one")]
    pub background_size_x: f32,
    /// Background height. Default `1`.
    #[serde(rename = "BackgroundSizeY", default = "one")]
    pub background_size_y: f32,
    /// Background horizontal offset. Default `0`.
    #[serde(rename = "BackgroundOffsetX", default)]
    pub background_offset_x: f32,
    /// Background vertical offset. Default `0`.
    #[serde(rename = "BackgroundOffsetY", default)]
    pub background_offset_y: f32,
}

impl Default for PieProgress {
    fn default() -> Self {
        Self {
            name: String::new(),
            background_texture: String::new(),
            mask_texture: String::new(),
            size_x: 1.0,
            size_y: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            fill: false,
            clockwise: false,
            background_size_x: 1.0,
            background_size_y: 1.0,
            background_offset_x: 0.0,
            background_offset_y: 0.0,
        }
    }
}

/// One `<BobbleHead>`: a hero portrait with veterancy stars around it.
///
/// `BBobbleHeadDef_reset` @ `0x14003e350` sets empty textures, the sizes, scales, skews and
/// the star radius to `1`, the star angle range to `90` and the offsets to `0`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BobbleHead {
    /// Definition name, for example `"UNSC_Spartan"`. The engine finds `ctf_flag` and
    /// `ctf_flag_spartan` by name.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Background texture path.
    #[serde(rename = "BackgroundTexture", default)]
    pub background_texture: String,
    /// Veterancy star texture path.
    #[serde(rename = "VetTexture", default)]
    pub vet_texture: String,
    /// Portrait texture path.
    #[serde(rename = "PortraitTexture", default)]
    pub portrait_texture: String,
    /// Star positions, in document order.
    #[serde(rename = "VetAnchor", default)]
    pub vet_anchors: HpBarAnchorList,
    /// Star radius. Default `1`.
    #[serde(rename = "VetRadius", default = "one")]
    pub vet_radius: f32,
    /// Star angle range in degrees. Default `90`.
    #[serde(rename = "VetAngleRange", default = "ninety")]
    pub vet_angle_range: f32,
    /// Star horizontal offset. Default `0`.
    #[serde(rename = "VetOffsetX", default)]
    pub vet_offset_x: f32,
    /// Star vertical offset. Default `0`.
    #[serde(rename = "VetOffsetY", default)]
    pub vet_offset_y: f32,
    /// Star horizontal scale. Default `1`.
    #[serde(rename = "VetScaleX", default = "one")]
    pub vet_scale_x: f32,
    /// Star vertical scale. Default `1`.
    #[serde(rename = "VetScaleY", default = "one")]
    pub vet_scale_y: f32,
    /// Star horizontal skew. Default `1`.
    #[serde(rename = "VetSkewX", default = "one")]
    pub vet_skew_x: f32,
    /// Star vertical skew. Default `1`.
    #[serde(rename = "VetSkewY", default = "one")]
    pub vet_skew_y: f32,
    /// Portrait width. Default `1`.
    #[serde(rename = "PortraitSizeX", default = "one")]
    pub portrait_size_x: f32,
    /// Portrait height. Default `1`.
    #[serde(rename = "PortraitSizeY", default = "one")]
    pub portrait_size_y: f32,
    /// Portrait horizontal offset. Default `0`.
    #[serde(rename = "PortraitOffsetX", default)]
    pub portrait_offset_x: f32,
    /// Portrait vertical offset. Default `0`.
    #[serde(rename = "PortraitOffsetY", default)]
    pub portrait_offset_y: f32,
    /// Background width. Default `1`.
    #[serde(rename = "BackgroundSizeX", default = "one")]
    pub background_size_x: f32,
    /// Background height. Default `1`.
    #[serde(rename = "BackgroundSizeY", default = "one")]
    pub background_size_y: f32,
    /// Background horizontal offset. Default `0`.
    #[serde(rename = "BackgroundOffsetX", default)]
    pub background_offset_x: f32,
    /// Background vertical offset. Default `0`.
    #[serde(rename = "BackgroundOffsetY", default)]
    pub background_offset_y: f32,
}

impl Default for BobbleHead {
    fn default() -> Self {
        Self {
            name: String::new(),
            background_texture: String::new(),
            vet_texture: String::new(),
            portrait_texture: String::new(),
            vet_anchors: HpBarAnchorList::default(),
            vet_radius: 1.0,
            vet_angle_range: 90.0,
            vet_offset_x: 0.0,
            vet_offset_y: 0.0,
            vet_scale_x: 1.0,
            vet_scale_y: 1.0,
            vet_skew_x: 1.0,
            vet_skew_y: 1.0,
            portrait_size_x: 1.0,
            portrait_size_y: 1.0,
            portrait_offset_x: 0.0,
            portrait_offset_y: 0.0,
            background_size_x: 1.0,
            background_size_y: 1.0,
            background_offset_x: 0.0,
            background_offset_y: 0.0,
        }
    }
}

/// One `<BuildingStrength>`: the strength indicator of a base.
///
/// `BBuildingStrengthDef_reset` @ `0x14003e590` sets empty textures, the size and the padding
/// to `1`, the offset to `0` and no anchors.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildingStrength {
    /// Definition name, for example `"UNSC_Base"`.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Texture path of an active indicator.
    #[serde(rename = "TextureOn", default)]
    pub texture_on: String,
    /// Texture path of an inactive indicator.
    #[serde(rename = "TextureOff", default)]
    pub texture_off: String,
    /// Indicator width. Default `1`.
    #[serde(rename = "SizeX", default = "one")]
    pub size_x: f32,
    /// Indicator height. Default `1`.
    #[serde(rename = "SizeY", default = "one")]
    pub size_y: f32,
    /// Horizontal offset. Default `0`.
    #[serde(rename = "OffsetX", default)]
    pub offset_x: f32,
    /// Vertical offset. Default `0`.
    #[serde(rename = "OffsetY", default)]
    pub offset_y: f32,
    /// Space between two indicators. Default `1`.
    #[serde(rename = "Padding", default = "one")]
    pub padding: f32,
    /// Indicator positions, in document order. The engine fails the load if an anchor does not
    /// have both offsets.
    #[serde(rename = "Anchors", default)]
    pub anchors: HpBarAnchorList,
}

impl Default for BuildingStrength {
    fn default() -> Self {
        Self {
            name: String::new(),
            texture_on: String::new(),
            texture_off: String::new(),
            size_x: 1.0,
            size_y: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            padding: 1.0,
            anchors: HpBarAnchorList::default(),
        }
    }
}

/// A list of `<Anchor offsetX offsetY>` children (`<VetAnchor>`, `<Anchors>`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HpBarAnchorList {
    /// The anchors, in document order.
    #[serde(rename = "Anchor", default)]
    pub anchors: Vec<HpBarAnchor>,
}

/// One `<Anchor offsetX offsetY>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
pub struct HpBarAnchor {
    /// Horizontal offset.
    #[serde(rename = "@offsetX", default)]
    pub offset_x: f32,
    /// Vertical offset.
    #[serde(rename = "@offsetY", default)]
    pub offset_y: f32,
}

const fn one() -> f32 {
    1.0
}

const fn ninety() -> f32 {
    90.0
}

/// Parse an `hpbars.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the root is not `HPBarDefinition` or a definition cannot be
/// deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<HpBarDefinition> {
    let root = expect_root(doc, "HPBarDefinition")?;
    Ok(bdt_serde::from_node(root)?)
}
