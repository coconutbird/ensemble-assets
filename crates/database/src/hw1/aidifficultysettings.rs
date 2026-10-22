//! Parser for `aidifficultysettings.xml.xmb`: the named settings that the AI reads by
//! difficulty.
//!
//! Each `BAI` loads this file in its constructor (`BAI_construct` calls
//! `BAI_loadDifficultySettings` @ `0x140150640`). The root is `<AIDifficultySettings>`. Each
//! child is one `<AIDifficultySetting>`; another child name stops the load.
//! `BAIDifficultySetting_load` @ `0x14014CFA0` reads the attributes `Name`, `InputLabel`,
//! `OutputLabel` and `Comment`, the `<DefaultValue Name Value>` children and one
//! `<PiecewiseFunc>`. `BPiecewiseFunc_load` @ `0x1402DA220` reads its `<DataPoint Input Output>`
//! children.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::parse_table;

/// One named AI difficulty setting from `aidifficultysettings.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AIDifficultySetting {
    /// Setting name, for example `"MaxMissionFocusTime"`. The engine finds a setting by this
    /// name and ignores case.
    #[serde(rename = "@Name", default)]
    pub name: String,
    /// Label of the function input, for example `"Difficulty"`.
    #[serde(rename = "@InputLabel")]
    pub input_label: Option<String>,
    /// Label of the function output, for example `"Seconds"`.
    #[serde(rename = "@OutputLabel")]
    pub output_label: Option<String>,
    /// Designer comment.
    #[serde(rename = "@Comment")]
    pub comment: Option<String>,
    /// Named fixed values, in document order.
    #[serde(rename = "DefaultValue", default)]
    pub default_values: Vec<AIDifficultyDefaultValue>,
    /// The function of the player difficulty.
    #[serde(rename = "PiecewiseFunc")]
    pub function: Option<PiecewiseFunc>,
}

/// One `<DefaultValue>` of a setting: a fixed value with a name, for example `"SPCEasy"`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AIDifficultyDefaultValue {
    /// Name of the value. The engine finds a value by this name and ignores case.
    #[serde(rename = "@Name", default)]
    pub name: String,
    /// The value.
    #[serde(rename = "@Value", default)]
    pub value: f32,
}

/// A piecewise linear function (`BPiecewiseFunc`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PiecewiseFunc {
    /// Data points in document order.
    #[serde(rename = "DataPoint", default)]
    pub points: Vec<PiecewiseDataPoint>,
}

/// One `<DataPoint>` of a [`PiecewiseFunc`].
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct PiecewiseDataPoint {
    /// Input value.
    #[serde(rename = "@Input", default)]
    pub input: f32,
    /// Output value at the input.
    #[serde(rename = "@Output", default)]
    pub output: f32,
}

/// Parse all settings from an `aidifficultysettings.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the root is not `AIDifficultySettings` or a setting cannot be
/// deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<AIDifficultySetting>> {
    parse_table(doc, "AIDifficultySettings", "AIDifficultySetting")
}
