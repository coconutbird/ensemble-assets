//! Terrain texture resolution and decoding.
//!
//! An XTT file names the terrain splat textures and the terrain decals. Each name refers to a
//! set of DDX maps below `art\terrain\`, one map for each suffix (for example `_df` for the
//! diffuse map). This module owns that path rule. It also loads and decodes the diffuse,
//! normal and specular maps of the splat textures, and it supplies a fallback image when a map
//! is missing or does not decode.
//!
//! The functions read the archives in sequence because [`AssetSource`] needs mutable access.
//! They decode the DDX data in parallel and keep the order of the input textures.

use ddx::DdxTexture;
use rayon::prelude::*;
use xtt::ActiveTextureInfo;

use crate::source::AssetSource;

/// Side length of the placeholder diffuse image, in pixels.
const PLACEHOLDER_SIZE: u32 = 64;

/// Returns the archive path of one terrain texture map.
///
/// `name` is a splat texture name or a decal name from an XTT file, for example
/// `arcadia\grass_01`. The function changes `/` to `\` and removes leading separators.
/// `suffix` is the map suffix without the underscore, for example `df`.
///
/// # Examples
///
/// ```
/// use pipeline::hw1::terrain::terrain_texture_path;
///
/// assert_eq!(
///     terrain_texture_path("arcadia/grass_01", "df"),
///     "art\\terrain\\arcadia\\grass_01_df.ddx"
/// );
/// ```
#[must_use]
pub fn terrain_texture_path(name: &str, suffix: &str) -> String {
    let name = name.replace('/', "\\");
    let name = name.trim_start_matches('\\');
    format!("art\\terrain\\{name}_{suffix}.ddx")
}

/// A decoded RGBA8 terrain image.
#[derive(Clone)]
pub struct TerrainImage {
    /// Texture name from the XTT file (for example, `arcadia\grass_01`).
    pub name: String,
    /// Width, in pixels.
    pub width: u32,
    /// Height, in pixels.
    pub height: u32,
    /// RGBA8 pixel data, row by row.
    pub pixels: Vec<u8>,
}

/// A decoded terrain diffuse texture and its XTT tiling scale.
#[derive(Clone)]
pub struct TerrainTexture {
    /// The decoded diffuse image.
    pub image: TerrainImage,
    /// U scale from XTT.
    pub u_scale: i32,
    /// V scale from XTT.
    pub v_scale: i32,
}

/// The decoded maps of all terrain splat textures, in XTT order.
///
/// The three lists have the same length. Index `i` of each list belongs to XTT active
/// texture `i`.
#[derive(Clone, Default)]
pub struct TerrainTextures {
    /// Diffuse maps (`_df`).
    pub diffuse: Vec<TerrainTexture>,
    /// Normal maps (`_nm`).
    pub normal: Vec<TerrainImage>,
    /// Colored specular maps (`_sp`).
    pub specular: Vec<TerrainImage>,
}

/// A terrain material map with a fallback image of uniform color.
#[derive(Clone, Copy)]
enum MaterialMap {
    /// The `_nm` map. The fallback is a flat normal.
    Normal,
    /// The `_sp` map. The fallback is black, which disables specular for the layer.
    Specular,
}

impl MaterialMap {
    /// Returns the archive suffix of the map.
    const fn suffix(self) -> &'static str {
        match self {
            Self::Normal => "nm",
            Self::Specular => "sp",
        }
    }

    /// Returns the RGBA8 value of each fallback texel.
    const fn fallback_texel(self) -> [u8; 4] {
        match self {
            Self::Normal => [128, 128, 255, 255],
            Self::Specular => [0, 0, 0, 255],
        }
    }

    /// Writes the log message for a decoded map.
    fn log_loaded(self, image: &TerrainImage) {
        let kind = match self {
            Self::Normal => "normal",
            Self::Specular => "specular",
        };
        log::info!(
            "Loaded {kind} map: {} ({}x{})",
            image.name,
            image.width,
            image.height
        );
    }

    /// Writes the log message for a map that uses the fallback.
    fn log_fallback(self, index: usize, name: &str) {
        match self {
            Self::Normal => {
                log::warn!("Failed to load normal map [{index}]: {name} - using a flat normal");
            }
            Self::Specular => log::warn!(
                "Failed to load specular map [{index}]: {name} - disabling specular for this layer"
            ),
        }
    }

    /// Returns the name of the map in a dimension error message.
    const fn dimension_label(self) -> &'static str {
        match self {
            Self::Normal => "normal-map",
            Self::Specular => "specular-map",
        }
    }
}

/// Decodes DDX data into an RGBA8 image. Returns `None` if the data is absent or invalid.
fn decode_image(name: &str, data: Option<&[u8]>) -> Option<TerrainImage> {
    let ddx = DdxTexture::from_bytes(data?).ok()?;
    let image = ddx.decode_to_rgba().ok()?;
    Some(TerrainImage {
        name: name.to_owned(),
        width: image.width,
        height: image.height,
        pixels: image.pixels,
    })
}

/// Decodes one diffuse map. Uses a magenta placeholder if the map does not decode.
fn decode_diffuse(
    index: usize,
    data: Option<&[u8]>,
    texture: &ActiveTextureInfo,
) -> TerrainTexture {
    if let Some(image) = decode_image(&texture.filename, data) {
        log::info!(
            "Loaded terrain texture [{index}]: {} ({}x{})",
            image.name,
            image.width,
            image.height
        );
        return TerrainTexture {
            image,
            u_scale: texture.u_scale,
            v_scale: texture.v_scale,
        };
    }

    log::warn!(
        "Failed to load texture [{index}]: {} - using placeholder",
        texture.filename
    );
    placeholder_texture(&texture.filename)
}

/// Decodes one normal or specular map.
///
/// If the map does not decode, the function returns a uniform fallback image with the
/// dimensions of the diffuse map of the same layer.
fn decode_material_map(
    map: MaterialMap,
    index: usize,
    data: Option<&[u8]>,
    name: &str,
    diffuse: &TerrainImage,
) -> TerrainImage {
    if let Some(image) = decode_image(name, data) {
        map.log_loaded(&image);
        return image;
    }

    map.log_fallback(index, name);
    let texel_count = diffuse
        .width
        .checked_mul(diffuse.height)
        .and_then(|count| usize::try_from(count).ok())
        .unwrap_or_else(|| panic!("{} dimensions must fit usize", map.dimension_label()));
    TerrainImage {
        name: name.to_owned(),
        width: diffuse.width,
        height: diffuse.height,
        pixels: map.fallback_texel().repeat(texel_count),
    }
}

/// Returns the magenta diffuse placeholder for a texture that does not load.
fn placeholder_texture(name: &str) -> TerrainTexture {
    let texel_count = usize::try_from(PLACEHOLDER_SIZE * PLACEHOLDER_SIZE)
        .expect("placeholder texel count must fit usize");
    TerrainTexture {
        image: TerrainImage {
            name: format!("placeholder_{name}"),
            width: PLACEHOLDER_SIZE,
            height: PLACEHOLDER_SIZE,
            pixels: [255_u8, 0, 255, 255].repeat(texel_count),
        },
        u_scale: 1,
        v_scale: 1,
    }
}

/// Decodes the normal or specular maps of all layers in parallel.
fn decode_material_maps(
    map: MaterialMap,
    data: &[Option<Vec<u8>>],
    active_textures: &[ActiveTextureInfo],
    diffuse: &[TerrainTexture],
) -> Vec<TerrainImage> {
    data.par_iter()
        .zip(active_textures.par_iter())
        .enumerate()
        .map(|(index, (data, texture))| {
            decode_material_map(
                map,
                index,
                data.as_deref(),
                &texture.filename,
                &diffuse[index].image,
            )
        })
        .collect()
}

/// Loads the diffuse, normal and specular maps of the XTT active textures.
///
/// The result keeps the order of `active_textures`. A diffuse map that does not load becomes a
/// 64x64 magenta placeholder. A normal map that does not load becomes a flat normal, and a
/// specular map that does not load becomes black. These two fallbacks use the dimensions of
/// the diffuse map of the same layer.
pub fn load_terrain_textures(
    source: &mut AssetSource<impl assets::FileProvider>,
    active_textures: &[ActiveTextureInfo],
) -> TerrainTextures {
    if active_textures.is_empty() {
        return TerrainTextures::default();
    }

    log::info!(
        "Loading terrain textures in parallel ({} total)",
        active_textures.len()
    );

    let suffixes = [
        "df",
        MaterialMap::Normal.suffix(),
        MaterialMap::Specular.suffix(),
    ];
    let all_data: Vec<Option<Vec<u8>>> = suffixes
        .iter()
        .flat_map(|suffix| {
            active_textures
                .iter()
                .map(move |texture| terrain_texture_path(&texture.filename, suffix))
        })
        .map(|path| source.resolve_exact(&path))
        .collect();
    let (diffuse_data, remaining_data) = all_data.split_at(active_textures.len());
    let (normal_data, specular_data) = remaining_data.split_at(active_textures.len());

    let diffuse: Vec<TerrainTexture> = diffuse_data
        .par_iter()
        .zip(active_textures.par_iter())
        .enumerate()
        .map(|(index, (data, texture))| decode_diffuse(index, data.as_deref(), texture))
        .collect();
    let normal = decode_material_maps(MaterialMap::Normal, normal_data, active_textures, &diffuse);
    let specular = decode_material_maps(
        MaterialMap::Specular,
        specular_data,
        active_textures,
        &diffuse,
    );

    log::info!(
        "Loaded {} terrain textures, {} normal maps, {} specular maps",
        diffuse.len(),
        normal.len(),
        specular.len()
    );

    TerrainTextures {
        diffuse,
        normal,
        specular,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{StdFileProvider, normalise_path};

    fn active_texture(filename: &str) -> ActiveTextureInfo {
        ActiveTextureInfo {
            filename: filename.to_owned(),
            u_scale: 2,
            v_scale: 4,
            blend_op: 0,
        }
    }

    #[test]
    fn texture_path_matches_the_runtime_lookup_key() {
        // The game data resolves `art/terrain/{name}_{suffix}.ddx`. The lookup key is equal.
        for name in [
            "arcadia\\grass_01",
            "harvest/Snow_02",
            "sw interior\\geiserspring_01_dcl",
        ] {
            let runtime = format!("art/terrain/{}_df.ddx", name.replace('\\', "/"));
            assert_eq!(
                normalise_path(&terrain_texture_path(name, "df")),
                normalise_path(&runtime)
            );
        }
    }

    #[test]
    fn texture_path_removes_leading_separators() {
        assert_eq!(
            terrain_texture_path("/arcadia\\rock_01", "sp"),
            "art\\terrain\\arcadia\\rock_01_sp.ddx"
        );
    }

    #[test]
    fn missing_maps_use_placeholder_flat_normal_and_black_specular() {
        let mut source = AssetSource::with_provider(StdFileProvider);
        let textures = load_terrain_textures(
            &mut source,
            &[active_texture("a\\one"), active_texture("a\\two")],
        );

        assert_eq!(textures.diffuse.len(), 2);
        assert_eq!(textures.normal.len(), 2);
        assert_eq!(textures.specular.len(), 2);
        let texel_count = 64 * 64;
        for (index, name) in ["a\\one", "a\\two"].into_iter().enumerate() {
            let diffuse = &textures.diffuse[index];
            assert_eq!(diffuse.image.name, format!("placeholder_{name}"));
            assert_eq!((diffuse.image.width, diffuse.image.height), (64, 64));
            assert_eq!((diffuse.u_scale, diffuse.v_scale), (1, 1));
            assert_eq!(diffuse.image.pixels, [255, 0, 255, 255].repeat(texel_count));

            let normal = &textures.normal[index];
            assert_eq!(normal.name, name);
            assert_eq!((normal.width, normal.height), (64, 64));
            assert_eq!(normal.pixels, [128, 128, 255, 255].repeat(texel_count));

            let specular = &textures.specular[index];
            assert_eq!(specular.name, name);
            assert_eq!((specular.width, specular.height), (64, 64));
            assert_eq!(specular.pixels, [0, 0, 0, 255].repeat(texel_count));
        }
    }
}
