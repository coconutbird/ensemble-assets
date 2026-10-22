//! Start positions and player placement of a `.scn` file.

use serde::{Deserialize, Serialize};

use super::retail_text::{
    bool_or_true, integer_or_minus_one, minus_one, present_text, scan_vec3, strtol, yes,
};

/// A start position in the scenario.
///
/// `BScenario_loadPosition` @ `0x14037f430` reads one `<Position>` element.
/// The field defaults are the engine defaults.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScenarioPosition {
    /// Scenario player that this position belongs to.
    ///
    /// A value of `1` or more pins the position to that scenario player. The
    /// engine default is `-1` (free position).
    #[serde(
        rename = "@Player",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub player: i32,
    /// Position number. Grouped placement puts each team on one number.
    ///
    /// The engine default is `-1`.
    #[serde(
        rename = "@Number",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub number: i32,
    /// World position as comma-separated `"x,y,z"` string.
    ///
    /// The engine drops a `<Position>` element that has no valid position.
    #[serde(rename = "@Position", default)]
    pub position: String,
    /// Forward direction as comma-separated `"x,y,z"` string, or `None` if absent.
    #[serde(
        rename = "@Forward",
        default,
        deserialize_with = "present_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub forward: Option<String>,
    /// Whether the position uses the default camera.
    ///
    /// The engine default is `true`. The engine reads the camera values only
    /// when this value is `false`.
    #[serde(
        rename = "@DefaultCamera",
        default = "yes",
        deserialize_with = "bool_or_true"
    )]
    pub default_camera: bool,
    /// Camera yaw in degrees.
    #[serde(rename = "@CameraYaw", default)]
    pub camera_yaw: f32,
    /// Camera pitch in degrees.
    #[serde(rename = "@CameraPitch", default)]
    pub camera_pitch: f32,
    /// Camera zoom level.
    #[serde(rename = "@CameraZoom", default)]
    pub camera_zoom: f32,
    /// Scenario object ID of the first unit start marker, or `-1`.
    #[serde(
        rename = "@UnitStartObject1",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub unit_start_object1: i32,
    /// Scenario object ID of the second unit start marker, or `-1`.
    #[serde(
        rename = "@UnitStartObject2",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub unit_start_object2: i32,
    /// Scenario object ID of the third unit start marker, or `-1`.
    #[serde(
        rename = "@UnitStartObject3",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub unit_start_object3: i32,
    /// Scenario object ID of the fourth unit start marker, or `-1`.
    #[serde(
        rename = "@UnitStartObject4",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub unit_start_object4: i32,
    /// Scenario object ID of the rally start marker, or `-1`.
    #[serde(
        rename = "@RallyStartObject",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub rally_start_object: i32,
}

impl Default for ScenarioPosition {
    fn default() -> Self {
        Self {
            player: -1,
            number: -1,
            position: String::new(),
            forward: None,
            default_camera: true,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_zoom: 0.0,
            unit_start_object1: -1,
            unit_start_object2: -1,
            unit_start_object3: -1,
            unit_start_object4: -1,
            rally_start_object: -1,
        }
    }
}

impl ScenarioPosition {
    /// World position, or `None` if the `Position` attribute is not a valid vector.
    ///
    /// `BScenario_loadPosition` drops the record when this value is `None`.
    #[must_use]
    pub fn position_vec3(&self) -> Option<[f32; 3]> {
        scan_vec3(&self.position)
    }

    /// Forward direction.
    ///
    /// An absent attribute gives `+Z`, the engine default. A present attribute
    /// that is not a valid vector gives the zero vector, because
    /// `sub_1406217A0` clears the vector when the read fails. The engine does
    /// not normalize the start position forward.
    #[must_use]
    pub fn forward_vec3(&self) -> [f32; 3] {
        self.forward
            .as_deref()
            .map_or([0.0, 0.0, 1.0], |text| scan_vec3(text).unwrap_or_default())
    }

    /// Scenario object IDs of the four unit start markers (`-1` for none).
    #[must_use]
    pub const fn unit_start_objects(&self) -> [i32; 4] {
        [
            self.unit_start_object1,
            self.unit_start_object2,
            self.unit_start_object3,
            self.unit_start_object4,
        ]
    }
}

/// `<PlayerPlacement>` element with `Type` and `Spacing` attributes.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlayerPlacementInfo {
    /// Placement type from the `Type` attribute (e.g. `"Grouped"`).
    #[serde(rename = "@Type", default)]
    pub placement_type: String,
    /// Spacing from the `Spacing` attribute, as text.
    #[serde(rename = "@Spacing", default, skip_serializing_if = "String::is_empty")]
    pub spacing: String,
}

impl PlayerPlacementInfo {
    /// Placement rule that the engine reads from this element.
    ///
    /// `BScenario_load` @ `0x140377be0` compares `Type` case-insensitively
    /// with `Grouped`, `Consecutive` and `Random`. It reads `Spacing` (with
    /// `strtol`) only for `Consecutive`. All other types (retail uses
    /// `Fixed`) give [`PlayerPlacement::Fixed`].
    #[must_use]
    pub fn placement(&self) -> PlayerPlacement {
        let kind = self.placement_type.as_str();
        if kind.eq_ignore_ascii_case("Grouped") {
            PlayerPlacement::Grouped
        } else if kind.eq_ignore_ascii_case("Consecutive") {
            let spacing = if self.spacing.is_empty() {
                0
            } else {
                strtol(&self.spacing)
            };
            PlayerPlacement::Consecutive { spacing }
        } else if kind.eq_ignore_ascii_case("Random") {
            PlayerPlacement::Random
        } else {
            PlayerPlacement::Fixed
        }
    }
}

/// Player placement rule of a scenario.
///
/// The engine stores the rule as an integer: `0` fixed, `1` grouped, `2`
/// consecutive and `3` random.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum PlayerPlacement {
    /// Players keep the position order. The engine value is `0`.
    #[default]
    Fixed,
    /// Each team gets the positions of one position number. The engine value is `1`.
    Grouped,
    /// Players get consecutive positions with a random offset. The engine value is `2`.
    Consecutive {
        /// Position spacing from the `Spacing` attribute.
        spacing: i32,
    },
    /// Each player gets a random position. The engine value is `3`.
    Random,
}
