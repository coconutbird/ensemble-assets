//! GLS/FLS lightset parser for HW1 scenarios.
//!
//! Each scenario references a **lightset** — a pair of files that define
//! the scene's lighting environment:
//!
//! - **`.gls.xmb`** — compiled XML with sun, fog, hemisphere, terrain
//!   material, and SH fill parameters.
//! - **`.fls`** — plain-text XML with 2nd-order spherical harmonic
//!   coefficients (9 per R/G/B channel).
//!
//! Use [`load`] to parse both files from an [`AssetSource`] given a
//! scenario descriptor, or [`parse_gls`] / [`parse_fls`] individually.

use serde::Deserialize;

use crate::source::AssetSource;

use super::scenario::ScenarioDescriptor;

/// Parsed lightset data from GLS + FLS files.
#[derive(Debug, Clone)]
pub struct LightSetData {
    // -- Sun / directional light --
    pub sun_inclination_deg: f32,
    pub sun_rotation_deg: f32,
    /// Sun colour for terrain, normalised 0..1.
    pub sun_terrain_color: [f32; 3],
    pub sun_terrain_intensity: f32,
    pub sun_terrain_shadow_darkness: f32,
    pub sun_shadows: bool,

    // -- Hemisphere fill light --
    pub hemi_terrain_top_color: [f32; 3],
    pub hemi_terrain_bottom_color: [f32; 3],
    pub hemi_terrain_intensity: f32,

    // -- SH fill --
    pub sh_fill_intensity: f32,
    pub sh_coeffs_r: [f32; 9],
    pub sh_coeffs_g: [f32; 9],
    pub sh_coeffs_b: [f32; 9],

    // -- Z-fog --
    pub z_fog_color: [f32; 3],
    pub z_fog_intensity: f32,
    pub z_fog_start: f32,
    pub z_fog_density: f32,

    // -- Planar fog --
    pub planar_fog_color: [f32; 3],
    pub planar_fog_intensity: f32,
    pub planar_fog_start: f32,
    pub planar_fog_density: f32,

    // -- Terrain material --
    pub terrain_specular_power: f32,
    pub terrain_bump_strength: f32,
    pub terrain_ao_diffuse_intensity: f32,

    // -- HDR / tone mapping --
    pub middle_grey: f32,
    pub bright_mask_thresh: f32,
    pub bloom_intensity: f32,
    pub bloom_sigma: f32,
    pub adaptation_rate: f32,
    pub log_ave_min: f32,
    pub log_ave_max: f32,
    pub white_point_min: f32,
    pub white_point_max: f32,
}

impl Default for LightSetData {
    fn default() -> Self {
        Self {
            sun_inclination_deg: 45.0,
            sun_rotation_deg: 0.0,
            sun_terrain_color: [1.0; 3],
            sun_terrain_intensity: 1.0,
            sun_terrain_shadow_darkness: 0.5,
            sun_shadows: true,
            hemi_terrain_top_color: [0.5, 0.5, 0.6],
            hemi_terrain_bottom_color: [0.2, 0.2, 0.15],
            hemi_terrain_intensity: 1.0,
            sh_fill_intensity: 1.0,
            sh_coeffs_r: [0.0; 9],
            sh_coeffs_g: [0.0; 9],
            sh_coeffs_b: [0.0; 9],
            z_fog_color: [0.5, 0.5, 0.5],
            z_fog_intensity: 0.0,
            z_fog_start: 0.0,
            z_fog_density: 0.0,
            planar_fog_color: [0.5, 0.5, 0.5],
            planar_fog_intensity: 0.0,
            planar_fog_start: 0.0,
            planar_fog_density: 0.0,
            terrain_specular_power: 20.0,
            terrain_bump_strength: 1.0,
            terrain_ao_diffuse_intensity: 1.0,
            middle_grey: 0.22,
            bright_mask_thresh: 0.8,
            bloom_intensity: 0.0,
            bloom_sigma: 1.0,
            adaptation_rate: 1.0,
            log_ave_min: -4.0,
            log_ave_max: 4.0,
            white_point_min: 1.5,
            white_point_max: 4.0,
        }
    }
}

impl LightSetData {
    /// Unit direction vector **toward** the sun, derived from inclination
    /// (polar angle from zenith) and rotation (azimuthal angle).
    pub fn sun_direction(&self) -> [f32; 3] {
        let inc = self.sun_inclination_deg.to_radians();
        let rot = self.sun_rotation_deg.to_radians();
        let sin_inc = inc.sin();
        [sin_inc * rot.sin(), inc.cos(), sin_inc * rot.cos()]
    }

    /// Sun colour scaled by terrain intensity (linear-space).
    pub fn sun_color_linear(&self) -> [f32; 3] {
        let i = self.sun_terrain_intensity;
        [
            self.sun_terrain_color[0] * i,
            self.sun_terrain_color[1] * i,
            self.sun_terrain_color[2] * i,
        ]
    }
}

// ---------------------------------------------------------------------------
// Raw serde structs — 1:1 mapping to the GLS / FLS XML via bdt_serde
// ---------------------------------------------------------------------------

/// Raw GLS deserialization target. All fields are XML attributes on the
/// `<lightSet>` root element. Colors arrive as comma-separated `"R,G,B"`
/// strings (0-255 range) and are normalised in the conversion to
/// [`LightSetData`].
#[derive(Debug, Clone, Default, Deserialize)]
struct GlsRaw {
    // -- Sun --
    #[serde(rename = "sunInclination")]
    sun_inclination: Option<f32>,
    #[serde(rename = "sunRotation")]
    sun_rotation: Option<f32>,
    #[serde(rename = "setTerrainColor")]
    set_terrain_color: Option<String>,
    #[serde(rename = "sunTerrainIntensity")]
    sun_terrain_intensity: Option<f32>,
    #[serde(rename = "sunTerrainShadowDarkness")]
    sun_terrain_shadow_darkness: Option<f32>,
    #[serde(rename = "sunShadows")]
    sun_shadows: Option<bool>,

    // -- Hemisphere --
    #[serde(rename = "hemiTerrainTopColor")]
    hemi_terrain_top_color: Option<String>,
    #[serde(rename = "hemiTerrainBottomColor")]
    hemi_terrain_bottom_color: Option<String>,
    #[serde(rename = "hemiTerrainIntensity")]
    hemi_terrain_intensity: Option<f32>,

    // -- SH fill --
    #[serde(rename = "SHFillIntensity")]
    sh_fill_intensity: Option<f32>,

    // -- Z-fog --
    #[serde(rename = "zFogColor")]
    z_fog_color: Option<String>,
    #[serde(rename = "zFogIntensity")]
    z_fog_intensity: Option<f32>,
    #[serde(rename = "zFogStart")]
    z_fog_start: Option<f32>,
    #[serde(rename = "zFogDensity")]
    z_fog_density: Option<f32>,

    // -- Planar fog --
    #[serde(rename = "planarFogColor")]
    planar_fog_color: Option<String>,
    #[serde(rename = "planarFogIntensity")]
    planar_fog_intensity: Option<f32>,
    #[serde(rename = "planarFogStart")]
    planar_fog_start: Option<f32>,
    #[serde(rename = "planarFogDensity")]
    planar_fog_density: Option<f32>,

    // -- Terrain material --
    #[serde(rename = "terrainSpecularPower")]
    terrain_specular_power: Option<f32>,
    #[serde(rename = "terrainBumpStrength")]
    terrain_bump_strength: Option<f32>,
    #[serde(rename = "TerrainAODiffuseIntensity")]
    terrain_ao_diffuse_intensity: Option<f32>,

    // -- HDR / tone mapping --
    #[serde(rename = "middleGrey")]
    middle_grey: Option<f32>,
    #[serde(rename = "brightMaskThresh")]
    bright_mask_thresh: Option<f32>,
    #[serde(rename = "bloomIntensity")]
    bloom_intensity: Option<f32>,
    #[serde(rename = "bloomSigma")]
    bloom_sigma: Option<f32>,
    #[serde(rename = "adaptationRate")]
    adaptation_rate: Option<f32>,
    #[serde(rename = "logAveMin")]
    log_ave_min: Option<f32>,
    #[serde(rename = "logAveMax")]
    log_ave_max: Option<f32>,
    #[serde(rename = "whitePointMin")]
    white_point_min: Option<f32>,
    #[serde(rename = "whitePointMax")]
    white_point_max: Option<f32>,
}

/// Raw FLS SH coefficient channel — text content is 9 comma-separated f32s.
#[derive(Debug, Clone, Default, Deserialize)]
struct FlsChannel {
    #[serde(rename = "$text", default)]
    text: String,
}

/// Raw FLS deserialization target. The root element contains `<R>`, `<G>`,
/// `<B>` children whose text content holds 9 comma-separated SH coefficients.
#[derive(Debug, Clone, Default, Deserialize)]
struct FlsRaw {
    #[serde(rename = "R")]
    r: Option<FlsChannel>,
    #[serde(rename = "G")]
    g: Option<FlsChannel>,
    #[serde(rename = "B")]
    b: Option<FlsChannel>,
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

/// Parse an RGB colour string `"R,G,B"` (0-255) into normalised `[f32; 3]`.
fn parse_color(s: &str) -> [f32; 3] {
    let mut parts = s.split(',');
    let r: f32 = parts
        .next()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0.0);
    let g: f32 = parts
        .next()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0.0);
    let b: f32 = parts
        .next()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0.0);
    [r / 255.0, g / 255.0, b / 255.0]
}

/// Parse 9 comma-separated f32 values from a string.
fn parse_sh_coeffs(csv: &str) -> [f32; 9] {
    let mut coeffs = [0.0f32; 9];
    for (i, tok) in csv.split(',').enumerate() {
        if i >= 9 {
            break;
        }
        if let Ok(v) = tok.trim().parse::<f32>() {
            coeffs[i] = v;
        }
    }
    coeffs
}

impl From<GlsRaw> for LightSetData {
    fn from(raw: GlsRaw) -> Self {
        let d = LightSetData::default();
        LightSetData {
            sun_inclination_deg: raw.sun_inclination.unwrap_or(d.sun_inclination_deg),
            sun_rotation_deg: raw.sun_rotation.unwrap_or(d.sun_rotation_deg),
            sun_terrain_color: raw
                .set_terrain_color
                .as_deref()
                .map(parse_color)
                .unwrap_or(d.sun_terrain_color),
            sun_terrain_intensity: raw.sun_terrain_intensity.unwrap_or(d.sun_terrain_intensity),
            sun_terrain_shadow_darkness: raw
                .sun_terrain_shadow_darkness
                .unwrap_or(d.sun_terrain_shadow_darkness),
            sun_shadows: raw.sun_shadows.unwrap_or(d.sun_shadows),
            hemi_terrain_top_color: raw
                .hemi_terrain_top_color
                .as_deref()
                .map(parse_color)
                .unwrap_or(d.hemi_terrain_top_color),
            hemi_terrain_bottom_color: raw
                .hemi_terrain_bottom_color
                .as_deref()
                .map(parse_color)
                .unwrap_or(d.hemi_terrain_bottom_color),
            hemi_terrain_intensity: raw
                .hemi_terrain_intensity
                .unwrap_or(d.hemi_terrain_intensity),
            sh_fill_intensity: raw.sh_fill_intensity.unwrap_or(d.sh_fill_intensity),
            z_fog_color: raw
                .z_fog_color
                .as_deref()
                .map(parse_color)
                .unwrap_or(d.z_fog_color),
            z_fog_intensity: raw.z_fog_intensity.unwrap_or(d.z_fog_intensity),
            z_fog_start: raw.z_fog_start.unwrap_or(d.z_fog_start),
            z_fog_density: raw.z_fog_density.unwrap_or(d.z_fog_density),
            planar_fog_color: raw
                .planar_fog_color
                .as_deref()
                .map(parse_color)
                .unwrap_or(d.planar_fog_color),
            planar_fog_intensity: raw.planar_fog_intensity.unwrap_or(d.planar_fog_intensity),
            planar_fog_start: raw.planar_fog_start.unwrap_or(d.planar_fog_start),
            planar_fog_density: raw.planar_fog_density.unwrap_or(d.planar_fog_density),
            terrain_specular_power: raw
                .terrain_specular_power
                .unwrap_or(d.terrain_specular_power),
            terrain_bump_strength: raw.terrain_bump_strength.unwrap_or(d.terrain_bump_strength),
            terrain_ao_diffuse_intensity: raw
                .terrain_ao_diffuse_intensity
                .unwrap_or(d.terrain_ao_diffuse_intensity),
            middle_grey: raw.middle_grey.unwrap_or(d.middle_grey),
            bright_mask_thresh: raw.bright_mask_thresh.unwrap_or(d.bright_mask_thresh),
            bloom_intensity: raw.bloom_intensity.unwrap_or(d.bloom_intensity),
            bloom_sigma: raw.bloom_sigma.unwrap_or(d.bloom_sigma),
            adaptation_rate: raw.adaptation_rate.unwrap_or(d.adaptation_rate),
            log_ave_min: raw.log_ave_min.unwrap_or(d.log_ave_min),
            log_ave_max: raw.log_ave_max.unwrap_or(d.log_ave_max),
            white_point_min: raw.white_point_min.unwrap_or(d.white_point_min),
            white_point_max: raw.white_point_max.unwrap_or(d.white_point_max),
            // SH coefficients come from FLS, not GLS
            ..d
        }
    }
}

// ---------------------------------------------------------------------------
// Public parsing API
// ---------------------------------------------------------------------------

/// Parse a GLS XMB document into a [`LightSetData`] via `bdt_serde`.
///
/// Unrecognised fields are silently ignored; missing fields use defaults.
pub fn parse_gls(doc: &xmb::Document) -> LightSetData {
    let Some(root) = doc.root() else {
        return LightSetData::default();
    };
    let raw: GlsRaw = bdt_serde::from_node(root).unwrap_or_default();
    raw.into()
}

/// Parse FLS SH coefficients and merge them into an existing [`LightSetData`].
///
/// Handles both binary XMB and plain-text XML FLS files.
pub fn parse_fls(data: &[u8], ls: &mut LightSetData) {
    // Try binary XMB first (compiled FLS).
    if let Ok(doc) = xmb::Reader::read(data)
        && let Some(root) = doc.root()
        && let Ok(fls) = bdt_serde::from_node::<FlsRaw>(root)
    {
        if let Some(ch) = &fls.r {
            ls.sh_coeffs_r = parse_sh_coeffs(&ch.text);
        }
        if let Some(ch) = &fls.g {
            ls.sh_coeffs_g = parse_sh_coeffs(&ch.text);
        }
        if let Some(ch) = &fls.b {
            ls.sh_coeffs_b = parse_sh_coeffs(&ch.text);
        }
        return;
    }

    // Fall back to plain-text XML via the xml crate's tree builder.
    let text = match std::str::from_utf8(data) {
        Ok(t) => t,
        Err(_) => return,
    };

    fn extract_tag_content<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        let start = xml.find(&open)? + open.len();
        let end = xml[start..].find(&close)? + start;
        Some(xml[start..end].trim())
    }

    if let Some(r) = extract_tag_content(text, "R") {
        ls.sh_coeffs_r = parse_sh_coeffs(r);
    }
    if let Some(g) = extract_tag_content(text, "G") {
        ls.sh_coeffs_g = parse_sh_coeffs(g);
    }
    if let Some(b) = extract_tag_content(text, "B") {
        ls.sh_coeffs_b = parse_sh_coeffs(b);
    }
}

// ---------------------------------------------------------------------------
// High-level loader
// ---------------------------------------------------------------------------

/// Load the lightset for a scenario from the asset source.
///
/// Reads the GLS (`.gls.xmb`) and FLS (`.fls`) files referenced by the
/// scenario descriptor's lightset path, and returns the combined
/// [`LightSetData`].
///
/// Returns `None` if neither file can be found.
pub fn load(
    descriptor: &ScenarioDescriptor,
    lightset_ref: &str,
    src: &mut AssetSource<impl assets::FileProvider>,
) -> Option<LightSetData> {
    if lightset_ref.is_empty() {
        return None;
    }

    // The lightset ref from the SCN is a bare name like "blood_gulch".
    // The GLS file lives at the same path as the SCN but with .gls.xmb extension.
    let base = descriptor.terrain_base()?;
    let gls_path = format!("{base}.gls.xmb");
    let fls_path = format!("{base}.fls");

    let mut ls = if let Some(doc) = src.read_xmb(gls_path.trim_end_matches(".xmb")) {
        parse_gls(&doc)
    } else {
        // No GLS — start from defaults
        LightSetData::default()
    };

    // Merge FLS SH coefficients if the file exists.
    if let Some(fls_data) = src.resolve_exact(&fls_path) {
        parse_fls(&fls_data, &mut ls);
    }

    Some(ls)
}
