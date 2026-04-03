//! HW1 scenario descriptor and scene data parsing.
//!
//! Scenarios in HW1 are described in `scenariodescriptions.xml.xmb` which
//! lists all available maps. Each scenario has an associated ERA file
//! containing map-specific assets (terrain, lightmaps, trigger scripts).
//!
//! The `.scn` file itself is an XMB document containing the full scene:
//! placed objects, player definitions, start positions, terrain references,
//! cinematics, objectives, triggers, and more.
//!
//! The engine loads scenario descriptors via `BScenarioList::load` which
//! reads `data\scenariodescriptions.xml.xmb` from the asset source.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::source::AssetSource;

// ── Scene data types ───────────────────────────────────────────────────

/// A placed object in the scenario.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioObject {
    /// Proto-object name (the text content of the `<Object>` node).
    #[serde(rename = "$text", default)]
    pub proto_name: String,
    /// Unique object ID within the scenario.
    #[serde(rename = "@ID", default)]
    pub id: i32,
    /// Whether this is a squad placement.
    #[serde(rename = "@IsSquad", default)]
    pub is_squad: bool,
    /// Owning player index (1-based, 0 = Gaia).
    #[serde(rename = "@Player", default)]
    pub player: i32,
    /// Editor name (e.g. `"cov_bldg_shadeTurret_01_11446"`).
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
    /// Visual variation index.
    #[serde(rename = "@VisualVariationIndex", default)]
    pub visual_variation_index: i32,
    /// Tint value.
    #[serde(rename = "@TintValue", default)]
    pub tint_value: f32,
}

impl ScenarioObject {
    /// Parse the position string into `[f32; 3]`.
    pub fn position_vec3(&self) -> [f32; 3] {
        parse_vec3(&self.position)
    }
    /// Parse the forward string into `[f32; 3]`.
    pub fn forward_vec3(&self) -> [f32; 3] {
        parse_vec3(&self.forward)
    }
    /// Parse the right string into `[f32; 3]`.
    pub fn right_vec3(&self) -> [f32; 3] {
        parse_vec3(&self.right)
    }
}

/// A player definition in the scenario.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioPlayer {
    /// Player name (e.g. `"Player1"`, `"Covenant"`).
    #[serde(rename = "@Name", default)]
    pub name: String,
    /// Localised display name (e.g. `"25532,Player1"`).
    #[serde(rename = "@LocalisedDisplayName", default)]
    pub localised_display_name: String,
    /// Civilization (e.g. `"UNSC"`, `"Covenant"`).
    #[serde(rename = "@Civ", default)]
    pub civ: String,
    /// Leader name (e.g. `"Major Vanilla"`).
    #[serde(rename = "@Leader1", default)]
    pub leader1: String,
    /// Team number.
    #[serde(rename = "@Team", default)]
    pub team: i32,
    /// Player colour index.
    #[serde(rename = "@Color", default)]
    pub color: i32,
    /// Whether the player is controllable.
    #[serde(rename = "@Controllable", default)]
    pub controllable: bool,
    /// Starting supplies.
    #[serde(rename = "@Supplies", default)]
    pub supplies: f32,
    /// Starting power (reactors).
    #[serde(rename = "@Power", default)]
    pub power: f32,
}

/// A start position in the scenario.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioPosition {
    /// Player index for this position (-1 = unassigned).
    #[serde(rename = "@Player", default)]
    pub player: i32,
    /// Position number.
    #[serde(rename = "@Number", default)]
    pub number: i32,
    /// World position as comma-separated `"x,y,z"` string.
    #[serde(rename = "@Position", default)]
    pub position: String,
    /// Forward direction as comma-separated `"x,y,z"` string.
    #[serde(rename = "@Forward", default)]
    pub forward: String,
    /// Default camera flag.
    #[serde(rename = "@DefaultCamera", default)]
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
}

impl ScenarioPosition {
    /// Parse the position string into `[f32; 3]`.
    pub fn position_vec3(&self) -> [f32; 3] {
        parse_vec3(&self.position)
    }
    /// Parse the forward string into `[f32; 3]`.
    pub fn forward_vec3(&self) -> [f32; 3] {
        parse_vec3(&self.forward)
    }
}

/// A cinematic reference from the SCN file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CinematicRef {
    /// Cinematic ID.
    #[serde(rename = "@ID", default)]
    pub id: i32,
    /// Cinematic file path.
    #[serde(rename = "$text", default)]
    pub path: String,
}

/// A talking head reference from the SCN file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TalkingHeadRef {
    /// Talking head ID.
    #[serde(rename = "@ID", default)]
    pub id: i32,
    /// Talking head name.
    #[serde(rename = "$text", default)]
    pub name: String,
}

/// An objective reference from the SCN file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectiveRef {
    /// Objective ID.
    #[serde(rename = "@id", default)]
    pub id: i32,
    /// Objective name/description.
    #[serde(rename = "$text", default)]
    pub name: String,
}

/// Terrain element — has text content and a `LoadVisRep` attribute.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TerrainInfo {
    /// Terrain reference name.
    #[serde(rename = "$text", default)]
    pub name: String,
    /// Whether to load the terrain visual representation.
    #[serde(rename = "@LoadVisRep", default)]
    pub load_vis_rep: bool,
}

/// `<Lightsets>` wrapper containing `<Lightset>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LightsetsWrapper {
    #[serde(rename = "Lightset", default)]
    pub entries: Vec<String>,
}

/// `<PlayerPlacement>` element with a `Type` attribute.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlayerPlacementInfo {
    #[serde(rename = "@Type", default)]
    pub placement_type: String,
}

/// `<Minimap>` wrapper containing `<MinimapTexture>` child.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MinimapInfo {
    #[serde(rename = "MinimapTexture", default)]
    pub texture: Option<String>,
}

/// `<Objects>` wrapper containing `<Object>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectsWrapper {
    #[serde(rename = "Object", default)]
    pub entries: Vec<ScenarioObject>,
}

/// `<Players>` wrapper containing `<Player>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlayersWrapper {
    #[serde(rename = "Player", default)]
    pub entries: Vec<ScenarioPlayer>,
}

/// `<Positions>` wrapper containing `<Position>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PositionsWrapper {
    #[serde(rename = "Position", default)]
    pub entries: Vec<ScenarioPosition>,
}

/// `<Cinematics>` wrapper containing `<Cinematic>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CinematicsWrapper {
    #[serde(rename = "Cinematic", default)]
    pub entries: Vec<CinematicRef>,
}

/// `<TalkingHeads>` wrapper containing `<TalkingHead>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TalkingHeadsWrapper {
    #[serde(rename = "TalkingHead", default)]
    pub entries: Vec<TalkingHeadRef>,
}

/// `<Objectives>` wrapper containing `<Objective>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectivesWrapper {
    #[serde(rename = "Objective", default)]
    pub entries: Vec<ObjectiveRef>,
}

/// `<SoundBanks>` wrapper — child element names vary, so we collect text.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SoundBanksWrapper {
    #[serde(rename = "SoundBank", default)]
    pub entries: Vec<String>,
}

/// Sim bounds from the SCN file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SimBoundsInfo {
    #[serde(rename = "MinX", default)]
    pub min_x: Option<f32>,
    #[serde(rename = "MinZ", default)]
    pub min_z: Option<f32>,
    #[serde(rename = "MaxX", default)]
    pub max_x: Option<f32>,
    #[serde(rename = "MaxZ", default)]
    pub max_z: Option<f32>,
}

impl SimBoundsInfo {
    /// Convert to `[min_x, min_z, max_x, max_z]`, returning `None` if all zero.
    pub fn to_array(&self) -> Option<[f32; 4]> {
        let arr = [
            self.min_x.unwrap_or(0.0),
            self.min_z.unwrap_or(0.0),
            self.max_x.unwrap_or(0.0),
            self.max_z.unwrap_or(0.0),
        ];
        if arr == [0.0, 0.0, 0.0, 0.0] {
            None
        } else {
            Some(arr)
        }
    }
}

/// The fully parsed `.scn` scenario file.
///
/// Deserialized directly from the `<Scenario>` XMB root via `bdt-serde`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioData {
    /// Terrain reference (has text + `@LoadVisRep` attribute).
    #[serde(rename = "Terrain")]
    pub terrain_info: Option<TerrainInfo>,
    /// Sky dome reference.
    #[serde(rename = "Sky")]
    pub sky: Option<String>,
    /// Sky environment texture path.
    #[serde(rename = "TerrainEnv")]
    pub terrain_env: Option<String>,
    /// Global lightset reference.
    #[serde(rename = "Lightset")]
    pub lightset: Option<String>,
    /// Additional lightset entries.
    #[serde(rename = "Lightsets")]
    pub lightsets: Option<LightsetsWrapper>,
    /// Pathing reference.
    #[serde(rename = "Pathing")]
    pub pathing: Option<String>,
    /// Player placement type.
    #[serde(rename = "PlayerPlacement")]
    pub player_placement: Option<PlayerPlacementInfo>,
    /// Minimap configuration.
    #[serde(rename = "Minimap")]
    pub minimap: Option<MinimapInfo>,
    /// All placed objects.
    #[serde(rename = "Objects")]
    pub objects: Option<ObjectsWrapper>,
    /// Player definitions.
    #[serde(rename = "Players")]
    pub players: Option<PlayersWrapper>,
    /// Start positions.
    #[serde(rename = "Positions")]
    pub positions: Option<PositionsWrapper>,
    /// Cinematic references.
    #[serde(rename = "Cinematics")]
    pub cinematics: Option<CinematicsWrapper>,
    /// Talking head references.
    #[serde(rename = "TalkingHeads")]
    pub talking_heads: Option<TalkingHeadsWrapper>,
    /// Objective names.
    #[serde(rename = "Objectives")]
    pub objectives_wrapper: Option<ObjectivesWrapper>,
    /// Sound bank names.
    #[serde(rename = "SoundBanks")]
    pub sound_banks: Option<SoundBanksWrapper>,
    /// Sim bounds.
    #[serde(rename = "SimBounds")]
    pub sim_bounds_info: Option<SimBoundsInfo>,
}

impl ScenarioData {
    /// Terrain reference name.
    pub fn terrain(&self) -> &str {
        self.terrain_info.as_ref().map_or("", |t| t.name.as_str())
    }
    /// Whether to load the terrain visual representation.
    pub fn terrain_load_vis_rep(&self) -> bool {
        self.terrain_info.as_ref().is_some_and(|t| t.load_vis_rep)
    }
    /// Sky dome reference.
    pub fn sky(&self) -> &str {
        self.sky.as_deref().unwrap_or("")
    }
    /// Terrain environment texture path.
    pub fn terrain_env(&self) -> &str {
        self.terrain_env.as_deref().unwrap_or("")
    }
    /// Global lightset reference.
    pub fn lightset(&self) -> &str {
        self.lightset.as_deref().unwrap_or("")
    }
    /// Additional lightset entries.
    pub fn lightsets(&self) -> &[String] {
        self.lightsets.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Pathing reference.
    pub fn pathing(&self) -> &str {
        self.pathing.as_deref().unwrap_or("")
    }
    /// Minimap texture path.
    pub fn minimap_texture(&self) -> &str {
        self.minimap
            .as_ref()
            .and_then(|m| m.texture.as_deref())
            .unwrap_or("")
    }
    /// Player placement type.
    pub fn player_placement_type(&self) -> &str {
        self.player_placement
            .as_ref()
            .map_or("", |p| p.placement_type.as_str())
    }
    /// All placed objects.
    pub fn objects(&self) -> &[ScenarioObject] {
        self.objects.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Player definitions.
    pub fn players(&self) -> &[ScenarioPlayer] {
        self.players.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Start positions.
    pub fn positions(&self) -> &[ScenarioPosition] {
        self.positions.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Cinematic references.
    pub fn cinematics(&self) -> &[CinematicRef] {
        self.cinematics.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Talking head references.
    pub fn talking_heads(&self) -> &[TalkingHeadRef] {
        self.talking_heads.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Objective references.
    pub fn objectives(&self) -> &[ObjectiveRef] {
        self.objectives_wrapper.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Sound bank names.
    pub fn sound_banks(&self) -> &[String] {
        self.sound_banks.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Sim bounds as `[min_x, min_z, max_x, max_z]`, or `None` if absent/all-zero.
    pub fn sim_bounds(&self) -> Option<[f32; 4]> {
        self.sim_bounds_info.as_ref().and_then(|s| s.to_array())
    }
}

/// A single scenario descriptor from `scenariodescriptions.xml`.
///
/// All fields are stored as attributes on the `<ScenarioInfo>` element.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioDescriptor {
    /// Map file path (e.g. `"CampaignUNSC\\design\\campaignTutorial\\campaignTutorial.scn"`).
    #[serde(rename = "@File", default)]
    pub file: String,

    /// Scenario type: `"Skirmish"`, `"Campaign"`, `"Multiplayer"`.
    #[serde(rename = "@Type", default)]
    pub scenario_type: String,

    /// Maximum number of players.
    #[serde(rename = "@MaxPlayers", default)]
    pub max_players: u32,

    /// Display name string ID.
    #[serde(rename = "@NameStringID")]
    pub name_string_id: Option<u32>,

    /// Info/rollover string ID.
    #[serde(rename = "@InfoStringID")]
    pub info_string_id: Option<u32>,

    /// Map image path.
    #[serde(rename = "@MapName")]
    pub map_name: Option<String>,

    /// Loading screen identifier.
    #[serde(rename = "@LoadingScreen")]
    pub loading_screen: Option<String>,
}

impl ScenarioDescriptor {
    /// Derive a short name from the `File` path (last path component, no extension).
    pub fn name(&self) -> &str {
        self.file
            .rsplit(['\\', '/'])
            .next()
            .and_then(|s| s.strip_suffix(".scn"))
            .unwrap_or(&self.file)
    }

    /// Path to this scenario's `.scn` file inside the asset source.
    ///
    /// The engine prepends `scenario\` to the file path from the descriptor.
    pub fn scn_path(&self) -> String {
        let normalized = self.file.replace('/', "\\");
        format!("scenario\\{normalized}")
    }

    /// Load and parse the `.scn` XMB file for this scenario.
    ///
    /// Returns the fully parsed scene data (objects, players, positions, etc.).
    pub fn read_scenario(
        &self,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Option<ScenarioData> {
        let path = self.scn_path();
        let doc = src.read_xmb(&path)?;
        let root = doc.root()?;
        Some(parse_scenario_data(root))
    }

    /// Return the base path for this scenario's terrain files.
    ///
    /// The engine derives terrain paths by stripping the `.scn` extension
    /// from the scenario file path and prepending `scenario\`.
    ///
    /// For example, `skirmish\design\blood_gulch\blood_gulch.scn` becomes
    /// `scenario\skirmish\design\blood_gulch\blood_gulch`.
    fn terrain_base(&self) -> Option<String> {
        let base = self.file.strip_suffix(".scn")?;
        let base = base.replace('/', "\\");
        Some(format!("scenario\\{base}"))
    }

    /// Path to this scenario's XTD (terrain displacement/heightmap) file.
    ///
    /// Follows the engine's `BTerrainIOLoader::loadXTDInternal` logic:
    /// `scenario\{scn_path_without_ext}.xtd`
    pub fn xtd_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.xtd"))
    }

    /// Path to this scenario's XTT (terrain textures/foliage/roads) file.
    ///
    /// Follows the engine's `BTerrainIOLoader::loadXTTInternal` logic:
    /// `scenario\{scn_path_without_ext}.xtt`
    pub fn xtt_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.xtt"))
    }
}

/// All scenario descriptors loaded from the game data.
#[derive(Debug, Clone, Default)]
pub struct ScenarioList {
    /// All scenarios keyed by derived name.
    pub scenarios: HashMap<String, ScenarioDescriptor>,
}

impl ScenarioList {
    /// The game path for the scenario descriptions file.
    pub const GAME_PATH: &'static str = "data\\scenariodescriptions.xml";

    /// Load scenario descriptors from the asset source.
    ///
    /// Parses `data\scenariodescriptions.xml.xmb` and returns all scenario
    /// entries. Returns an empty list if the file is not found.
    pub fn load(assets: &mut AssetSource<impl assets::FileProvider>) -> Self {
        let mut list = Self::default();

        let Some(doc) = assets.read_xmb(Self::GAME_PATH) else {
            return list;
        };

        let Some(root) = doc.root() else {
            return list;
        };

        for child in root.children.iter().filter(|c| c.name == "ScenarioInfo") {
            if let Ok(desc) = bdt_serde::from_node::<ScenarioDescriptor>(child) {
                let name = desc.name().to_string();
                if !name.is_empty() {
                    list.scenarios.insert(name, desc);
                }
            }
        }

        list
    }

    /// Serialize all scenario descriptors back into an XMB [`Document`](xmb::Document).
    ///
    /// Produces `<ScenarioDescriptions><ScenarioInfo .../>...</ScenarioDescriptions>`.
    pub fn to_document(&self) -> Result<xmb::Document, bdt_serde::Error> {
        let mut root = bdt::Node::new("ScenarioDescriptions");
        // Sort by name for deterministic output
        let mut entries: Vec<_> = self.scenarios.values().collect();
        entries.sort_by(|a, b| a.file.cmp(&b.file));
        for desc in entries {
            root.add_child(bdt_serde::to_node("ScenarioInfo", desc)?);
        }
        Ok(xmb::Document::with_root(root))
    }
}

// ── SCN parsing ─────────────────────────────────────────────────────────

/// Parse a `<Scenario>` XMB root node into [`ScenarioData`] via `bdt-serde`.
fn parse_scenario_data(root: &bdt::Node) -> ScenarioData {
    bdt_serde::from_node(root).unwrap_or_default()
}

/// Parse a comma-separated `"x,y,z"` string into `[f32; 3]`.
fn parse_vec3(s: &str) -> [f32; 3] {
    let mut parts = s.split(',');
    let x = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let y = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let z = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    [x, y, z]
}
