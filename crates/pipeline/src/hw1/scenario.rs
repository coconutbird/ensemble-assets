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
#[derive(Debug, Clone, Default)]
pub struct ScenarioObject {
    /// Proto-object name (the text content of the `<Object>` node).
    pub proto_name: String,
    /// Unique object ID within the scenario.
    pub id: i32,
    /// Whether this is a squad placement.
    pub is_squad: bool,
    /// Owning player index (1-based, 0 = Gaia).
    pub player: i32,
    /// Editor name (e.g. `"cov_bldg_shadeTurret_01_11446"`).
    pub editor_name: String,
    /// World position `(x, y, z)`.
    pub position: [f32; 3],
    /// Forward direction vector.
    pub forward: [f32; 3],
    /// Right direction vector.
    pub right: [f32; 3],
    /// Object group ID (-1 = none).
    pub group: i32,
    /// Visual variation index.
    pub visual_variation_index: i32,
    /// Tint value.
    pub tint_value: f32,
}

/// A player definition in the scenario.
#[derive(Debug, Clone, Default)]
pub struct ScenarioPlayer {
    /// Player name (e.g. `"Player1"`, `"Covenant"`).
    pub name: String,
    /// Localised display name (e.g. `"25532,Player1"`).
    pub localised_display_name: String,
    /// Civilization (e.g. `"UNSC"`, `"Covenant"`).
    pub civ: String,
    /// Leader name (e.g. `"Major Vanilla"`).
    pub leader1: String,
    /// Team number.
    pub team: i32,
    /// Player colour index.
    pub color: i32,
    /// Whether the player is controllable.
    pub controllable: bool,
    /// Starting supplies.
    pub supplies: f32,
    /// Starting power (reactors).
    pub power: f32,
}

/// A start position in the scenario.
#[derive(Debug, Clone, Default)]
pub struct ScenarioPosition {
    /// Player index for this position (-1 = unassigned).
    pub player: i32,
    /// Position number.
    pub number: i32,
    /// World position `(x, y, z)`.
    pub position: [f32; 3],
    /// Forward direction.
    pub forward: [f32; 3],
    /// Default camera flag.
    pub default_camera: bool,
    /// Camera yaw in degrees.
    pub camera_yaw: f32,
    /// Camera pitch in degrees.
    pub camera_pitch: f32,
    /// Camera zoom level.
    pub camera_zoom: f32,
}

/// The fully parsed `.scn` scenario file.
#[derive(Debug, Clone, Default)]
pub struct ScenarioData {
    /// Terrain reference name (from `<Terrain>` text).
    pub terrain: String,
    /// Whether to load the terrain visual representation.
    pub terrain_load_vis_rep: bool,
    /// Sky dome reference (from `<Sky>` text).
    pub sky: String,
    /// Sky environment texture path (from `<TerrainEnv>` text).
    pub terrain_env: String,
    /// Lightset reference (from `<Lightset>` text, global lightset).
    pub lightset: String,
    /// Additional lightset entries (from `<Lightsets>` children).
    pub lightsets: Vec<String>,
    /// Pathing reference (from `<Pathing>` text).
    pub pathing: String,
    /// Minimap texture path (from `<Minimap>/<MinimapTexture>` text).
    pub minimap_texture: String,
    /// Player placement type (e.g. `"Fixed"`).
    pub player_placement_type: String,
    /// All placed objects.
    pub objects: Vec<ScenarioObject>,
    /// Player definitions.
    pub players: Vec<ScenarioPlayer>,
    /// Start positions.
    pub positions: Vec<ScenarioPosition>,
    /// Cinematic references (ID → path).
    pub cinematics: Vec<(i32, String)>,
    /// Talking head references (ID → name).
    pub talking_heads: Vec<(i32, String)>,
    /// Objective names (id → name).
    pub objectives: Vec<(i32, String)>,
    /// Sound bank names (from `<SoundBanks>` children).
    pub sound_banks: Vec<String>,
    /// Sim bounds `(min_x, min_z, max_x, max_z)` if present.
    pub sim_bounds: Option<[f32; 4]>,
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

// ── SCN parsing helpers ──────────────────────────────────────────────────

/// Parse a `<Scenario>` XMB root node into [`ScenarioData`].
fn parse_scenario_data(root: &bdt::Node) -> ScenarioData {
    let mut data = ScenarioData::default();

    for child in &root.children {
        match child.name.as_str() {
            "Terrain" => {
                data.terrain = child.text_string().trim().to_string();
                if let Some(attr) = child.get_attribute("LoadVisRep") {
                    data.terrain_load_vis_rep = attr.value_string().eq_ignore_ascii_case("true");
                }
            }
            "Sky" => data.sky = child.text_string().trim().to_string(),
            "TerrainEnv" => data.terrain_env = child.text_string().trim().to_string(),
            "Lightset" => data.lightset = child.text_string().trim().to_string(),
            "Lightsets" => {
                for ls in &child.children {
                    let name = ls.text_string().trim().to_string();
                    if !name.is_empty() {
                        data.lightsets.push(name);
                    }
                }
            }
            "Pathing" => data.pathing = child.text_string().trim().to_string(),
            "PlayerPlacement" => {
                if let Some(a) = child.get_attribute("Type") {
                    data.player_placement_type = a.value_string();
                }
            }
            "Minimap" => {
                for mc in &child.children {
                    if mc.name == "MinimapTexture" {
                        data.minimap_texture = mc.text_string().trim().to_string();
                    }
                }
            }
            "Objects" => {
                for obj_node in child.children.iter().filter(|c| c.name == "Object") {
                    data.objects.push(parse_object(obj_node));
                }
            }
            "Players" => {
                for p_node in child.children.iter().filter(|c| c.name == "Player") {
                    data.players.push(parse_player(p_node));
                }
            }
            "Positions" => {
                for pos_node in child.children.iter().filter(|c| c.name == "Position") {
                    data.positions.push(parse_position(pos_node));
                }
            }
            "Cinematics" => {
                for cin in child.children.iter().filter(|c| c.name == "Cinematic") {
                    let id = attr_i32(cin, "ID");
                    let path = cin.text_string().trim().to_string();
                    data.cinematics.push((id, path));
                }
            }
            "TalkingHeads" => {
                for th in child.children.iter().filter(|c| c.name == "TalkingHead") {
                    let id = attr_i32(th, "ID");
                    let name = th.text_string().trim().to_string();
                    data.talking_heads.push((id, name));
                }
            }
            "Objectives" => {
                for obj_node in child.children.iter().filter(|c| c.name == "Objective") {
                    let id = attr_i32(obj_node, "id");
                    let name = obj_node.text_string().trim().to_string();
                    data.objectives.push((id, name));
                }
            }
            "SoundBanks" => {
                for sb in &child.children {
                    let name = sb.text_string().trim().to_string();
                    if !name.is_empty() {
                        data.sound_banks.push(name);
                    }
                }
            }
            "SimBounds" => {
                let min_x = child_f32(child, "MinX");
                let min_z = child_f32(child, "MinZ");
                let max_x = child_f32(child, "MaxX");
                let max_z = child_f32(child, "MaxZ");
                if min_x != 0.0 || min_z != 0.0 || max_x != 0.0 || max_z != 0.0 {
                    data.sim_bounds = Some([min_x, min_z, max_x, max_z]);
                }
            }
            _ => {} // TriggerSystem, EditorOnlyData, etc. — skip for pipeline
        }
    }

    data
}

fn parse_object(node: &bdt::Node) -> ScenarioObject {
    ScenarioObject {
        proto_name: node.text_string().trim().to_string(),
        id: attr_i32(node, "ID"),
        is_squad: attr_bool(node, "IsSquad"),
        player: attr_i32(node, "Player"),
        editor_name: attr_string(node, "EditorName"),
        position: parse_vec3(&attr_string(node, "Position")),
        forward: parse_vec3(&attr_string(node, "Forward")),
        right: parse_vec3(&attr_string(node, "Right")),
        group: attr_i32(node, "Group"),
        visual_variation_index: attr_i32(node, "VisualVariationIndex"),
        tint_value: attr_f32(node, "TintValue"),
    }
}

fn parse_player(node: &bdt::Node) -> ScenarioPlayer {
    ScenarioPlayer {
        name: attr_string(node, "Name"),
        localised_display_name: attr_string(node, "LocalisedDisplayName"),
        civ: attr_string(node, "Civ"),
        leader1: attr_string(node, "Leader1"),
        team: attr_i32(node, "Team"),
        color: attr_i32(node, "Color"),
        controllable: attr_bool(node, "Controllable"),
        supplies: attr_f32(node, "Supplies"),
        power: attr_f32(node, "Power"),
    }
}

fn parse_position(node: &bdt::Node) -> ScenarioPosition {
    ScenarioPosition {
        player: attr_i32(node, "Player"),
        number: attr_i32(node, "Number"),
        position: parse_vec3(&attr_string(node, "Position")),
        forward: parse_vec3(&attr_string(node, "Forward")),
        default_camera: attr_bool(node, "DefaultCamera"),
        camera_yaw: attr_f32(node, "CameraYaw"),
        camera_pitch: attr_f32(node, "CameraPitch"),
        camera_zoom: attr_f32(node, "CameraZoom"),
    }
}

// ── Attribute helpers ────────────────────────────────────────────────────

fn attr_string(node: &bdt::Node, name: &str) -> String {
    node.get_attribute(name)
        .map(|a| a.value_string())
        .unwrap_or_default()
}

fn attr_i32(node: &bdt::Node, name: &str) -> i32 {
    attr_string(node, name).parse().unwrap_or(0)
}

fn attr_f32(node: &bdt::Node, name: &str) -> f32 {
    attr_string(node, name).parse().unwrap_or(0.0)
}

fn attr_bool(node: &bdt::Node, name: &str) -> bool {
    attr_string(node, name).eq_ignore_ascii_case("true")
}

/// Parse a comma-separated `"x,y,z"` string into `[f32; 3]`.
fn parse_vec3(s: &str) -> [f32; 3] {
    let mut parts = s.split(',');
    let x = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let y = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let z = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    [x, y, z]
}

/// Get the text of a named child element and parse it as `f32`.
fn child_f32(node: &bdt::Node, name: &str) -> f32 {
    node.children
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.text_string().trim().parse().unwrap_or(0.0))
        .unwrap_or(0.0)
}
