//! Parser for `squads.xml.xmb` — squad definitions.
//!
//! Squads are groups of units that the player trains and controls together.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use super::Vector3;
use crate::node_ext::expect_root;

/// A squad definition from `squads.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Squad {
    /// Squad name (unique key), e.g. `"unsc_veh_warthog_01"`.
    #[serde(rename = "@name", default)]
    pub name: String,
    /// Database ID.
    #[serde(rename = "@dbid")]
    pub dbid: Option<i32>,
    /// Portrait icon path.
    #[serde(rename = "PortraitIcon")]
    pub portrait_icon: Option<String>,
    /// Minimap icon shape.
    #[serde(rename = "MinimapIcon")]
    pub minimap_icon: Option<String>,
    /// Display name string ID.
    #[serde(rename = "DisplayNameID")]
    pub display_name_id: Option<i32>,
    /// Rollover text string ID.
    #[serde(rename = "RolloverTextID")]
    pub rollover_text_id: Option<i32>,
    /// Role text string ID.
    #[serde(rename = "RoleTextID")]
    pub role_text_id: Option<i32>,
    /// Prerequisite text string ID.
    #[serde(rename = "PrereqTextID")]
    pub prereq_text_id: Option<i32>,
    /// Build time in seconds.
    #[serde(rename = "BuildPoints")]
    pub build_points: Option<f32>,
    /// Resource costs.
    #[serde(rename = "Cost", default)]
    pub costs: Vec<Cost>,
    /// Units in this squad (wrapper element).
    #[serde(rename = "Units")]
    pub units: Option<UnitsWrapper>,
    /// HP bar style and optional size/offset overrides.
    #[serde(rename = "HPBar")]
    pub hp_bar: Option<HpBar>,
    /// Birth configuration.
    #[serde(rename = "Birth")]
    pub birth: Option<Birth>,
    /// Flags.
    #[serde(rename = "Flag", default)]
    pub flags: Vec<String>,
    /// Leash distance for AI.
    #[serde(rename = "LeashDistance")]
    pub leash_distance: Option<f32>,
    /// Aggro distance for AI.
    #[serde(rename = "AggroDistance")]
    pub aggro_distance: Option<f32>,
    /// Sub-select sort priority.
    #[serde(rename = "SubSelectSort")]
    pub sub_select_sort: Option<i32>,
    /// Formation type attribute.
    #[serde(rename = "@formationType")]
    pub formation_type: Option<String>,
    /// Update flag (incremental data merge).
    #[serde(rename = "@update")]
    pub update: Option<bool>,
    /// Ability recovery bar style and placement.
    #[serde(rename = "AbilityRecoveryBar")]
    pub ability_recovery_bar: Option<StatusBar>,
    /// Bobble head configuration.
    #[serde(rename = "BobbleHead")]
    pub bobble_head: Option<String>,
    /// Alternate portrait/icon path.
    #[serde(rename = "AltIcon")]
    pub alt_icon: Option<String>,
    /// Default combat stance.
    #[serde(rename = "Stance")]
    pub stance: Option<String>,
    /// Animation used while the squad is trained.
    #[serde(rename = "TrainAnim")]
    pub train_anim: Option<String>,
    /// Building-strength display style.
    #[serde(rename = "BuildingStrengthDisplay")]
    pub building_strength_display: Option<String>,
    /// Whether the squad can attack while moving.
    #[serde(
        rename = "CanAttackWhileMoving",
        default,
        deserialize_with = "super::types::deserialize_optional_bool"
    )]
    pub can_attack_while_moving: Option<bool>,
    /// Cryo points.
    #[serde(rename = "CryoPoints")]
    pub cryo_points: Option<f32>,
    /// Daze resistance value.
    #[serde(rename = "DazeResist")]
    pub daze_resist: Option<f32>,
    /// Leash deadzone distance.
    #[serde(rename = "LeashDeadzone")]
    pub leash_deadzone: Option<f32>,
    /// Leash recall delay in milliseconds.
    #[serde(rename = "LeashRecallDelay")]
    pub leash_recall_delay: Option<u32>,
    /// Minimap icon scale.
    #[serde(rename = "MinimapScale")]
    pub minimap_scale: Option<f32>,
    /// Selection configuration.
    #[serde(rename = "Selection")]
    pub selection: Option<Selection>,
    /// Sound cue bindings.
    #[serde(rename = "Sound", default)]
    pub sounds: Vec<SquadSound>,
    /// Stats name string ID.
    #[serde(rename = "StatsNameID")]
    pub stats_name_id: Option<i32>,
    /// Turn radius (with optional min attribute).
    #[serde(rename = "TurnRadius")]
    pub turn_radius: Option<TurnRadius>,
    /// Veterancy bar style and placement.
    #[serde(rename = "VeterancyBar")]
    pub veterancy_bar: Option<StatusBar>,
    /// General progression level.
    #[serde(rename = "Level")]
    pub level: Option<i32>,
    /// Required technology level.
    #[serde(rename = "TechLevel")]
    pub tech_level: Option<i32>,
    /// Effect used while an ability is recovering.
    #[serde(rename = "RecoveringEffect")]
    pub recovering_effect: Option<String>,
}

/// Wrapper for the `<Units>` element containing `<Unit>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UnitsWrapper {
    /// The `<Unit>` entries.
    #[serde(rename = "Unit", default)]
    pub entries: Vec<UnitEntry>,
}

/// A resource cost entry.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Cost {
    /// Resource type name (`@resourcetype`).
    #[serde(rename = "@resourcetype", default)]
    pub resource_type: String,
    /// Cost quantity. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub amount: f32,
}

/// A unit entry within a squad.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UnitEntry {
    /// Proto-object name this unit references.
    #[serde(rename = "$text", default)]
    pub proto_object: String,
    /// Number of this unit in the squad.
    #[serde(rename = "@count", default = "default_one")]
    pub count: i32,
    /// Role: `"normal"`, etc.
    #[serde(rename = "@role")]
    pub role: Option<String>,
}

/// Turn radius element: `<TurnRadius min="..." max="...">value</TurnRadius>`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TurnRadius {
    /// Minimum turn radius (`@min`).
    #[serde(rename = "@min")]
    pub min: Option<f32>,
    /// Maximum turn radius (`@max`).
    #[serde(rename = "@max")]
    pub max: Option<f32>,
    /// Turn radius value. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub value: f32,
}

/// HP-bar style plus optional geometry overrides.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HpBar {
    /// HP-bar style name. Comes from the element text.
    #[serde(rename = "$text")]
    pub style: Option<String>,
    /// Bar width override (`@sizeX`).
    #[serde(rename = "@sizeX")]
    pub size_x: Option<f32>,
    /// Bar height override (`@sizeY`).
    #[serde(rename = "@sizeY")]
    pub size_y: Option<f32>,
    /// Bar position offset (`@offset`).
    #[serde(rename = "@offset")]
    pub offset: Option<Vector3>,
}

/// A veterancy or ability-recovery bar and its screen-space placement.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StatusBar {
    /// Bar style name. Comes from the element text.
    #[serde(rename = "$text")]
    pub style: Option<String>,
    /// Bar style for centered placement (`@Centered`).
    #[serde(rename = "@Centered")]
    pub centered_style: Option<String>,
    /// Bar width (`@sizeX`).
    #[serde(rename = "@sizeX")]
    pub size_x: Option<f32>,
    /// Bar height (`@sizeY`).
    #[serde(rename = "@sizeY")]
    pub size_y: Option<f32>,
    /// Horizontal screen offset (`@offsetX`).
    #[serde(rename = "@offsetX")]
    pub offset_x: Option<f32>,
    /// Vertical screen offset (`@offsetY`).
    #[serde(rename = "@offsetY")]
    pub offset_y: Option<f32>,
}

/// Birth/spawn animation configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Birth {
    /// Birth type name. Comes from the element text.
    #[serde(rename = "$text")]
    pub birth_type: Option<String>,
    /// Spawn point name (`@spawnPoint`).
    #[serde(rename = "@spawnPoint")]
    pub spawn_point: Option<String>,
    /// End point name (`@endPoint`).
    #[serde(rename = "@endPoint")]
    pub end_point: Option<String>,
    /// Birth animation 0 (`@anim0`).
    #[serde(rename = "@anim0")]
    pub animation_0: Option<String>,
    /// Birth animation 1 (`@anim1`).
    #[serde(rename = "@anim1")]
    pub animation_1: Option<String>,
    /// Birth animation 2 (`@anim2`).
    #[serde(rename = "@anim2")]
    pub animation_2: Option<String>,
    /// Birth animation 3 (`@anim3`).
    #[serde(rename = "@anim3")]
    pub animation_3: Option<String>,
    /// Animation of the trainer building (`@trainerAnim`).
    #[serde(rename = "@trainerAnim")]
    pub trainer_animation: Option<String>,
}

/// Selection decal geometry and orientation behavior.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Selection {
    /// Decal half-width (`<RadiusX>`).
    #[serde(rename = "RadiusX")]
    pub radius_x: Option<f32>,
    /// Decal half-depth (`<RadiusZ>`).
    #[serde(rename = "RadiusZ")]
    pub radius_z: Option<f32>,
    /// Vertical offset of the decal (`<YOffset>`).
    #[serde(rename = "YOffset")]
    pub y_offset: Option<f32>,
    /// Forward offset of the decal (`<ZOffset>`).
    #[serde(rename = "ZOffset")]
    pub z_offset: Option<f32>,
    /// Make the decal follow the terrain (`<ConformToTerrain>`).
    #[serde(rename = "ConformToTerrain")]
    pub conform_to_terrain: Option<bool>,
    /// Let the decal rotate with the squad (`<AllowOrientation>`).
    #[serde(rename = "AllowOrientation")]
    pub allow_orientation: Option<bool>,
}

/// A squad sound cue with its trigger and optional filters.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SquadSound {
    /// Sound cue name. Comes from the element text.
    #[serde(rename = "$text", default)]
    pub cue: String,
    /// Sound event that plays the cue (`@Type`).
    #[serde(rename = "@Type")]
    pub sound_type: Option<String>,
    /// Squad filter (`@Squad`).
    #[serde(rename = "@Squad")]
    pub squad: Option<String>,
    /// World filter (`@World`).
    #[serde(rename = "@World")]
    pub world: Option<String>,
    /// Play the cue only for the casting unit (`@CastingUnitOnly`).
    #[serde(rename = "@CastingUnitOnly")]
    pub casting_unit_only: Option<bool>,
}

fn default_one() -> i32 {
    1
}

/// Parse all squads from a `squads.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document root is invalid or a squad cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<Squad>> {
    let root = expect_root(doc, "Squads")?;
    let squads: Vec<Squad> = root
        .children
        .iter()
        .filter(|c| c.name == "Squad")
        .map(bdt_serde::from_node)
        .collect::<Result<_, _>>()?;
    Ok(squads)
}
