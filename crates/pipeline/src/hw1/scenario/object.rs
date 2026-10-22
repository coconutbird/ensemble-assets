//! `<Object>` records of `.scn`, `.sc2` and `.sc3` files.

use serde::{Deserialize, Serialize};

use super::retail_text::{float_or_one, one, optional_bool, optional_integer, scan_vec3};

/// A placed object in the scenario.
///
/// `BScenario_loadObject` @ `0x14037d6a0` reads one `<Object>` element. The
/// field defaults are the engine defaults.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScenarioObject {
    /// Proto-object or proto-squad name (the text content of the `<Object>` node).
    ///
    /// The engine does not trim the name.
    #[serde(rename = "$text", default)]
    pub proto_name: String,
    /// Unique object ID within the scenario.
    #[serde(rename = "@ID", default)]
    pub id: i32,
    /// Whether this is a squad placement, or `None` if the attribute is absent.
    ///
    /// Text that is not an engine boolean (`true`, `1`, `false`, `0`, any
    /// case) also gives `None`. The engine then looks up the name as a squad
    /// first and as an object second.
    #[serde(
        rename = "@IsSquad",
        default,
        deserialize_with = "optional_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_squad: Option<bool>,
    /// Owning scenario player index (1-based, 0 = Gaia).
    #[serde(rename = "@Player", default)]
    pub player: i32,
    /// Editor name (e.g. `"cov_bldg_shadeTurret_01_11446"`). The engine does not read it.
    #[serde(rename = "@EditorName", default)]
    pub editor_name: String,
    /// World position as comma-separated `"x,y,z"` string.
    #[serde(rename = "@Position", default)]
    pub position: String,
    /// Forward direction as comma-separated `"x,y,z"` string.
    #[serde(rename = "@Forward", default)]
    pub forward: String,
    /// Right direction as comma-separated `"x,y,z"` string.
    #[serde(rename = "@Right", default)]
    pub right: String,
    /// Object group ID (-1 = none).
    #[serde(rename = "@Group", default)]
    pub group: i32,
    /// Authored visual variation index, or `None` if the attribute is absent.
    ///
    /// Retail uses an absent index to select the variation from the visual
    /// logic. The engine converts present text with `strtol`, so text that is
    /// not a number gives `0`. Only empty text counts as absent.
    #[serde(
        rename = "@VisualVariationIndex",
        default,
        deserialize_with = "optional_integer"
    )]
    pub visual_variation_index: Option<i32>,
    /// Tint value. The engine default is `1.0`.
    #[serde(
        rename = "@TintValue",
        default = "one",
        deserialize_with = "float_or_one"
    )]
    pub tint_value: f32,
    /// Flag names from the `<Flag>` children, in document order.
    ///
    /// The engine uses only `NoTieToGround` and `IncludeInSimRep`. Retail
    /// files also have `OrientUnitWithGround` and `IgnoreToAO`, which the
    /// object load ignores.
    #[serde(rename = "Flag", default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<String>,
}

impl Default for ScenarioObject {
    fn default() -> Self {
        Self {
            proto_name: String::new(),
            id: 0,
            is_squad: None,
            player: 0,
            editor_name: String::new(),
            position: String::new(),
            forward: String::new(),
            right: String::new(),
            group: 0,
            visual_variation_index: None,
            tint_value: 1.0,
            flags: Vec::new(),
        }
    }
}

impl ScenarioObject {
    /// World position, or `None` if the `Position` attribute is not a valid vector.
    ///
    /// `BScenario_loadObject` skips the object when this value is `None`.
    #[must_use]
    pub fn position_vec3(&self) -> Option<[f32; 3]> {
        scan_vec3(&self.position)
    }

    /// Normalized forward direction.
    ///
    /// An absent or invalid attribute gives `+Z`, the engine default.
    #[must_use]
    pub fn forward_vec3(&self) -> [f32; 3] {
        scan_vec3(&self.forward).map_or([0.0, 0.0, 1.0], normalize)
    }

    /// Normalized right direction.
    ///
    /// An absent or invalid attribute gives `+X`, the engine default.
    #[must_use]
    pub fn right_vec3(&self) -> [f32; 3] {
        scan_vec3(&self.right).map_or([1.0, 0.0, 0.0], normalize)
    }

    /// Whether the object has the `NoTieToGround` flag (case-insensitive).
    #[must_use]
    pub fn no_tie_to_ground(&self) -> bool {
        self.has_flag("NoTieToGround")
    }

    /// Whether the object has the `IncludeInSimRep` flag (case-insensitive).
    #[must_use]
    pub fn include_in_sim_rep(&self) -> bool {
        self.has_flag("IncludeInSimRep")
    }

    fn has_flag(&self, name: &str) -> bool {
        self.flags
            .iter()
            .any(|flag| flag.eq_ignore_ascii_case(name))
    }
}

/// Normalize a vector like `XMVector3Normalize` in `BScenario_loadObject`.
///
/// A zero vector stays zero. A vector with an infinite squared length gives
/// NaN components.
fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = vector;
    let length_squared = x * x + y * y + z * z;
    if length_squared.is_infinite() {
        return [f32::NAN; 3];
    }
    let length = length_squared.sqrt();
    if length == 0.0 {
        return [0.0; 3];
    }
    vector.map(|component| component / length)
}

/// `<Objects>` wrapper containing `<Object>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectsWrapper {
    /// Scenario objects from the `<Object>` children.
    #[serde(rename = "Object", default)]
    pub entries: Vec<ScenarioObject>,
}

/// The placed objects of a `.sc2` or `.sc3` file.
///
/// `BScenario_load` @ `0x140377be0` reads only the `<Objects>` children of
/// these files. It creates the `.sc2` objects (step `AA`) and then the `.sc3`
/// objects (step `BB`) after all `.scn` elements.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioObjectFile {
    /// All placed objects.
    #[serde(rename = "Objects")]
    pub objects: Option<ObjectsWrapper>,
}

impl ScenarioObjectFile {
    /// All placed objects in document order.
    #[must_use]
    pub fn objects(&self) -> &[ScenarioObject] {
        self.objects.as_ref().map_or(&[], |w| &w.entries)
    }

    /// Parse a `<Scenario>` XMB root node of a `.sc2` or `.sc3` file.
    ///
    /// # Errors
    ///
    /// Returns an error if an `<Objects>` child does not match the object schema.
    pub fn from_node(root: &bdt::Node) -> Result<Self, bdt_serde::Error> {
        bdt_serde::from_node(root)
    }

    /// Serialize these objects back to an XMB document.
    ///
    /// # Errors
    ///
    /// Returns an error if the objects cannot be serialized into an XMB node.
    pub fn to_document(&self) -> Result<xmb::Document, bdt_serde::Error> {
        let node = bdt_serde::to_node("Scenario", self)?;
        Ok(xmb::Document::with_root(node))
    }
}
