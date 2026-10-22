//! PFX particle effect loading.
//!
//! This module resolves a PFX path in the asset stack and reads the XMB document. The
//! [`database::hw1::pfx`] parser decodes the document.

use database::hw1::pfx::{ParticleEffect, PfxError};

use crate::source::AssetSource;

/// Load and parse a PFX effect from the asset stack.
///
/// The function changes `path` to its canonical form with [`canonical_effect_path`]. It then
/// tries the `.pfx.xmb` and `.xmb` extensions.
///
/// # Errors
///
/// Returns an error if the effect is missing, the XMB data is not valid, or the parser rejects
/// the document.
pub fn load(
    source: &mut AssetSource<impl assets::FileProvider>,
    path: &str,
) -> Result<ParticleEffect, PfxLoadError> {
    let canonical = canonical_effect_path(path);
    let bytes = source
        .resolve_with_fallback(&canonical, &[".pfx.xmb", ".xmb"])
        .ok_or_else(|| PfxLoadError::NotFound(canonical.clone()))?;
    let document = xmb::Reader::read(&bytes).map_err(|error| PfxLoadError::Parse {
        path: canonical,
        reason: error.to_string(),
    })?;
    database::hw1::pfx::parse(&document).map_err(PfxLoadError::Decode)
}

/// Change a PFX reference into its canonical game path.
///
/// The canonical path uses backslashes, has the `art\` prefix and has no `.pfx` or `.xmb`
/// extension. The function keeps the case of the path.
#[must_use]
pub fn canonical_effect_path(path: &str) -> String {
    let normalized = path
        .trim()
        .trim_start_matches(['\\', '/'])
        .replace('/', "\\");
    let normalized = [".pfx.xmb", ".PFX.XMB", ".pfx", ".PFX", ".xmb", ".XMB"]
        .into_iter()
        .find_map(|suffix| normalized.strip_suffix(suffix))
        .unwrap_or(&normalized);
    if normalized
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("art\\"))
    {
        normalized.to_owned()
    } else {
        format!("art\\{normalized}")
    }
}

/// PFX loading error.
#[derive(Debug)]
pub enum PfxLoadError {
    /// Effect file was absent from the active asset stack.
    NotFound(String),
    /// XMB decoding failed.
    Parse {
        /// Canonical game path.
        path: String,
        /// XMB diagnostic.
        reason: String,
    },
    /// The PFX parser rejected the document.
    Decode(PfxError),
}

impl core::fmt::Display for PfxLoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "particle effect not found: {path}"),
            Self::Parse { path, reason } => {
                write!(f, "failed to parse particle effect '{path}': {reason}")
            }
            Self::Decode(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for PfxLoadError {}

#[cfg(test)]
mod tests {
    use super::canonical_effect_path;

    #[test]
    fn pfx_path_is_canonicalized_without_double_extensions() {
        assert_eq!(
            canonical_effect_path("effects/fire/test.pfx.xmb"),
            "art\\effects\\fire\\test"
        );
    }
}
