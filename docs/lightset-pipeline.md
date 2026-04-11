# Lightset Pipeline Implementation Plan

## Overview

HWDE stores per-scenario lighting parameters in two files referenced by the `.scn`:

| File | Format | Contents |
|------|--------|----------|
| `<name>.gls.xmb` | XMB (compiled XML) | Sun, fog, hemisphere, AO, HDR, specular, local lights |
| `<name>.fls` | Plain-text XML | 2nd-order SH coefficients (9 per R/G/B channel) |

These are stored inside the scenario ERA (e.g. `blood_gulch.era`) at paths like
`scenario\skirmish\design\blood_gulch\blood_gulch.gls.xmb`.

The SCN already references them via `<Lightset>` / `<Lightsets>`, and
`manifest.rs` collects the paths into `lightset_refs` — but `parse_lightset()`
is currently `unimplemented!()`.

---

## 1. New module: `crates/pipeline/src/hw1/lightset.rs`

Add a new `lightset` module to the HW1 pipeline. This parses both GLS and FLS
files and produces a single `LightSetData` struct.

### 1a. GLS fields to parse (`<lightSet>` root)

All fields are optional with sensible defaults.

**Sun / directional light:**
- `sunInclination: f32` — polar angle (degrees)
- `sunRotation: f32` — azimuthal angle (degrees)
- `setTerrainColor: [u8; 3]` — RGB 0-255, sun color for terrain
- `sunTerrainIntensity: f32`
- `sunTerrainShadowDarkness: f32`
- `sunShadows: bool`

**Hemisphere fill light:**
- `hemiTerrainTopColor: [u8; 3]`
- `hemiTerrainBottomColor: [u8; 3]`
- `hemiTerrainIntensity: f32`

**SH fill:**
- `SHFillIntensity: f32` — scales the FLS coefficients

**Fog:**
- `zFogColor: [u8; 3]`, `zFogIntensity: f32`, `zFogStart: f32`, `zFogDensity: f32`
- `planarFogColor: [u8; 3]`, `planarFogIntensity: f32`, `planarFogStart: f32`, `planarFogDensity: f32`

**Terrain material:**
- `terrainSpecularPower: f32`
- `terrainBumpStrength: f32`
- `TerrainAODiffuseIntensity: f32`

**HDR / tone mapping (future use):**
- `middleGrey`, `brightMaskThresh`, `bloomIntensity`, `bloomSigma`
- `adaptationRate`, `logAveMin`, `logAveMax`, `whitePointMin`, `whitePointMax`

**Local lights** (`<Light>` children — future use):
- Type, position, radius, color, intensity, shadow settings

### 1b. FLS fields to parse (`<SHLightParams>` root)

- `<R>`: 9 comma-separated f32 — SH coefficients for red channel
- `<G>`: 9 comma-separated f32 — SH coefficients for green channel
- `<B>`: 9 comma-separated f32 — SH coefficients for blue channel

### 1c. Output struct

```rust
/// Parsed lightset data from GLS + FLS files.
pub struct LightSetData {
    // -- Sun --
    pub sun_inclination_deg: f32,
    pub sun_rotation_deg: f32,
    pub sun_terrain_color: [f32; 3],   // normalized 0..1
    pub sun_terrain_intensity: f32,
    pub sun_terrain_shadow_darkness: f32,
    pub sun_shadows: bool,

    // -- Hemisphere --
    pub hemi_terrain_top_color: [f32; 3],
    pub hemi_terrain_bottom_color: [f32; 3],
    pub hemi_terrain_intensity: f32,

    // -- SH fill --
    pub sh_fill_intensity: f32,
    pub sh_coeffs_r: [f32; 9],
    pub sh_coeffs_g: [f32; 9],
    pub sh_coeffs_b: [f32; 9],

    // -- Fog --
    pub z_fog_color: [f32; 3],
    pub z_fog_intensity: f32,
    pub z_fog_start: f32,
    pub z_fog_density: f32,
    pub planar_fog_color: [f32; 3],
    pub planar_fog_intensity: f32,
    pub planar_fog_start: f32,
    pub planar_fog_density: f32,

    // -- Terrain material --
    pub terrain_specular_power: f32,
    pub terrain_bump_strength: f32,
    pub terrain_ao_diffuse_intensity: f32,
}
```

### 1d. Conversion helpers

`LightSetData` should provide:

- `sun_direction() -> [f32; 3]` — converts inclination/rotation (degrees) to
  a unit direction vector toward the sun.
- `sun_color_linear() -> [f32; 3]` — `sun_terrain_color * sun_terrain_intensity`.
- `sh_fill_ar/ag/ab/br/bg/bb/c() -> [f32; 4]` — packs the 9 SH coefficients
  per channel into the 7×vec4 layout expected by `LightingParams`, scaled by
  `sh_fill_intensity`.

---

## 2. Wire into `World`

In `crates/pipeline/src/hw1/world.rs`:

- Add `pub lightset: Option<LightSetData>` to the `World` struct.
- In `swap_scenario()`, after the SCN is parsed:
  1. Read the GLS XMB from the asset source (the path is in `manifest.lightset_refs[0]`).
  2. Decode it with `xmb::Reader` → XML string → deserialize.
  3. Read the corresponding FLS file (same base name, `.fls` extension).
  4. Call `lightset::parse(gls_xml, fls_data) -> LightSetData`.
  5. Store it in `world.lightset`.

---

## 3. Expose through `pipeline` re-exports

In `crates/pipeline/src/hw1/mod.rs`, add:
```rust
pub mod lightset;
pub use lightset::LightSetData;
```

In `crates/data/src/lib.rs` (openensemble), re-export:
```rust
pub use pipeline::hw1::lightset::LightSetData;
```

---

## 4. Consume in openensemble (`terrain_viewer`)

In `src/bin/terrain_viewer/viewer/rendering.rs`, replace the hardcoded
`LightingParams::default()` with values from `world.lightset`:

```rust
let lighting_params = if let Some(ls) = &world.lightset {
    ls.to_lighting_params(camera_pos, shadow_vp_cols, shadow_enabled)
} else {
    LightingParams { world_camera_pos: ..., ..Default::default() }
};
```

This `to_lighting_params()` method (on `LightSetData` or as a free function
in `render`) maps every GLS/FLS field to the corresponding `LightingParams`
field.

---

## File Inventory

| Repo | File | Action |
|------|------|--------|
| ensemble-assets | `crates/pipeline/src/hw1/lightset.rs` | **New** — GLS/FLS parser + `LightSetData` |
| ensemble-assets | `crates/pipeline/src/hw1/mod.rs` | Add `pub mod lightset` + re-export |
| ensemble-assets | `crates/pipeline/src/hw1/manifest.rs` | Replace `parse_lightset()` stub |
| ensemble-assets | `crates/pipeline/src/hw1/world.rs` | Add `lightset` field, load in `swap_scenario` |
| openensemble | `crates/data/src/lib.rs` | Re-export `LightSetData` |
| openensemble | `crates/render/src/terrain/uniforms.rs` | Add `from_lightset()` or conversion fn |
| openensemble | `src/bin/terrain_viewer/viewer/mod.rs` | Pass `world.lightset` to renderer |
| openensemble | `src/bin/terrain_viewer/viewer/rendering.rs` | Use real values instead of defaults |
