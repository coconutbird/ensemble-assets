//! HW1 scenario descriptors and `.scn` scene data parsing.
//!
//! A "scenario" is a single playable map. Each scenario has:
//!
//! - A **descriptor** ([`ScenarioDescriptor`]) from `scenarioDescriptions.xml`
//!   that declares metadata (map name, player count, image, terrain file paths).
//! - Optionally, a **scenario ERA** (`{scenario}.era`) loaded on top of the
//!   base ERA stack for map-specific overrides.
//! - Scene data ([`ScenarioData`]) parsed from the `.scn` XML file, containing
//!   placed objects, players, cinematics, objectives, and trigger variables.
//! - Two optional object files, `.sc2` and `.sc3` ([`ScenarioObjectFile`]).
//!   `BScenario_load` @ `0x140377be0` creates their objects after all `.scn`
//!   elements, first `.sc2` and then `.sc3`.
//!
//! The field defaults of the scene data types are the engine defaults. The
//! engine reads each value with the readers in `retail_text`.
//!
//! [`crate::hw1::loader::find_scenario_era`] handles fuzzy matching so callers can pass a map name
//! (e.g. `"blood_gulch"`), an ERA filename, or a full SCN path.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::source::AssetSource;

mod design_objects;
mod diplomacy;
mod load_steps;
mod object;
mod objectives;
mod player;
mod position;
mod retail_text;

pub use design_objects::{
    DesignData, DesignLine, DesignObjectError, DesignObjects, DesignSphere, parse_design_objects,
};
pub use diplomacy::{
    DIPLOMACY_TEAM_COUNT, DiplomaciesWrapper, DiplomacyTable, ScenarioDiplomacy,
    ScenarioDiplomacyEntry, ScenarioRelation,
};
pub use load_steps::{ExternalTriggerScript, ScenarioLoadStep, scenario_load_steps};
pub use object::{ObjectsWrapper, ScenarioObject, ScenarioObjectFile};
pub use objectives::{ObjectiveDefinition, ObjectiveError, parse_objectives};
pub use player::{
    ForbidObjectsWrapper, ForbidSquadsWrapper, ForbidTechsWrapper, PopsWrapper, ScenarioPlayer,
    ScenarioPop,
};
pub use position::{PlayerPlacement, PlayerPlacementInfo, ScenarioPosition};

use retail_text::optional_float;

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
    /// Lightset names from the `<Lightset>` children.
    #[serde(rename = "Lightset", default)]
    pub entries: Vec<String>,
}

/// `<Minimap>` wrapper containing `<MinimapTexture>` child.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MinimapInfo {
    /// Minimap texture path from `<MinimapTexture>`.
    #[serde(rename = "MinimapTexture", default)]
    pub texture: Option<String>,
}

/// `<Players>` wrapper containing `<Player>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlayersWrapper {
    /// Players from the `<Player>` children.
    #[serde(rename = "Player", default)]
    pub entries: Vec<ScenarioPlayer>,
}

/// `<Positions>` wrapper containing `<Position>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PositionsWrapper {
    /// Player start positions from the `<Position>` children.
    #[serde(rename = "Position", default)]
    pub entries: Vec<ScenarioPosition>,
}

/// `<Cinematics>` wrapper containing `<Cinematic>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CinematicsWrapper {
    /// Cinematics from the `<Cinematic>` children.
    #[serde(rename = "Cinematic", default)]
    pub entries: Vec<CinematicRef>,
}

/// `<TalkingHeads>` wrapper containing `<TalkingHead>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TalkingHeadsWrapper {
    /// Talking heads from the `<TalkingHead>` children.
    #[serde(rename = "TalkingHead", default)]
    pub entries: Vec<TalkingHeadRef>,
}

/// `<Objectives>` wrapper containing `<Objective>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ObjectivesWrapper {
    /// Objectives from the `<Objective>` children.
    #[serde(rename = "Objective", default)]
    pub entries: Vec<ObjectiveRef>,
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
    /// Player placement rule.
    #[serde(rename = "PlayerPlacement")]
    pub player_placement: Option<PlayerPlacementInfo>,
    /// Team relations.
    #[serde(rename = "Diplomacies")]
    pub diplomacies: Option<DiplomaciesWrapper>,
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
    /// Sound bank names from the top-level `<SoundBank>` elements.
    ///
    /// `BScenario_load` collects every top-level `SoundBank` element in
    /// document order before it reads the other elements.
    #[serde(rename = "SoundBank", default, skip_serializing_if = "Vec::is_empty")]
    pub sound_banks: Vec<String>,
    /// Minimum X of the simulation bounds, from `<SimBoundsMinX>`.
    #[serde(
        rename = "SimBoundsMinX",
        default,
        deserialize_with = "optional_float",
        skip_serializing_if = "Option::is_none"
    )]
    pub sim_bounds_min_x: Option<f32>,
    /// Minimum Z of the simulation bounds, from `<SimBoundsMinZ>`.
    #[serde(
        rename = "SimBoundsMinZ",
        default,
        deserialize_with = "optional_float",
        skip_serializing_if = "Option::is_none"
    )]
    pub sim_bounds_min_z: Option<f32>,
    /// Maximum X of the simulation bounds, from `<SimBoundsMaxX>`.
    #[serde(
        rename = "SimBoundsMaxX",
        default,
        deserialize_with = "optional_float",
        skip_serializing_if = "Option::is_none"
    )]
    pub sim_bounds_max_x: Option<f32>,
    /// Maximum Z of the simulation bounds, from `<SimBoundsMaxZ>`.
    #[serde(
        rename = "SimBoundsMaxZ",
        default,
        deserialize_with = "optional_float",
        skip_serializing_if = "Option::is_none"
    )]
    pub sim_bounds_max_z: Option<f32>,
    /// External trigger scripts from the `<ExternalTriggerScript>` elements.
    ///
    /// Use [`scenario_load_steps`] for their order relative to the
    /// `TriggerSystem` elements.
    #[serde(
        rename = "ExternalTriggerScript",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub external_trigger_scripts: Vec<ExternalTriggerScript>,
    /// Whether the scenario enables veterancy.
    ///
    /// Retail starts with veterancy disabled. An `AllowVeterancy` element
    /// enables it, unless its trimmed text is `0` or `false` (any case).
    #[serde(
        rename = "AllowVeterancy",
        default,
        deserialize_with = "retail_flag_text",
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub allow_veterancy: bool,
}

impl ScenarioData {
    /// Maximum number of `<Player>` elements that `BScenario_load` reads.
    pub const MAX_PLAYERS: usize = 8;

    /// Terrain reference name.
    #[must_use]
    pub fn terrain(&self) -> &str {
        self.terrain_info.as_ref().map_or("", |t| t.name.as_str())
    }
    /// Whether to load the terrain visual representation.
    #[must_use]
    pub fn terrain_load_vis_rep(&self) -> bool {
        self.terrain_info.as_ref().is_some_and(|t| t.load_vis_rep)
    }
    /// Sky dome reference.
    #[must_use]
    pub fn sky(&self) -> &str {
        self.sky.as_deref().unwrap_or("")
    }
    /// Terrain environment texture path.
    #[must_use]
    pub fn terrain_env(&self) -> &str {
        self.terrain_env.as_deref().unwrap_or("")
    }
    /// Global lightset reference.
    #[must_use]
    pub fn lightset(&self) -> &str {
        self.lightset.as_deref().unwrap_or("")
    }
    /// Additional lightset entries.
    #[must_use]
    pub fn lightsets(&self) -> &[String] {
        self.lightsets.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Pathing reference.
    #[must_use]
    pub fn pathing(&self) -> &str {
        self.pathing.as_deref().unwrap_or("")
    }
    /// Minimap texture path.
    #[must_use]
    pub fn minimap_texture(&self) -> &str {
        self.minimap
            .as_ref()
            .and_then(|m| m.texture.as_deref())
            .unwrap_or("")
    }
    /// Player placement rule. An absent element gives [`PlayerPlacement::Fixed`].
    #[must_use]
    pub fn player_placement(&self) -> PlayerPlacement {
        self.player_placement
            .as_ref()
            .map_or(PlayerPlacement::Fixed, PlayerPlacementInfo::placement)
    }
    /// Team relation table, or `None` if the file has no `<Diplomacy>` element.
    ///
    /// See [`DiplomaciesWrapper::table`].
    #[must_use]
    pub fn diplomacy_table(&self) -> Option<DiplomacyTable> {
        self.diplomacies
            .as_ref()
            .and_then(DiplomaciesWrapper::table)
    }
    /// All placed objects.
    #[must_use]
    pub fn objects(&self) -> &[ScenarioObject] {
        self.objects.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Player definitions.
    ///
    /// `BScenario_load` reads at most [`MAX_PLAYERS`](Self::MAX_PLAYERS)
    /// `<Player>` elements. This list keeps all of them.
    #[must_use]
    pub fn players(&self) -> &[ScenarioPlayer] {
        self.players.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Start positions.
    #[must_use]
    pub fn positions(&self) -> &[ScenarioPosition] {
        self.positions.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Cinematic references.
    #[must_use]
    pub fn cinematics(&self) -> &[CinematicRef] {
        self.cinematics.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Talking head references.
    #[must_use]
    pub fn talking_heads(&self) -> &[TalkingHeadRef] {
        self.talking_heads.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Objective references.
    #[must_use]
    pub fn objectives(&self) -> &[ObjectiveRef] {
        self.objectives_wrapper.as_ref().map_or(&[], |w| &w.entries)
    }
    /// Sound bank names.
    #[must_use]
    pub fn sound_banks(&self) -> &[String] {
        &self.sound_banks
    }
    /// Simulation bounds as `[min_x, min_z, max_x, max_z]`.
    ///
    /// An absent or empty element gives the engine default: `0.0` for the
    /// minimum values and `f32::MAX` for the maximum values.
    /// `BWorld_setSimBounds` @ `0x1405d2210` then clamps the bounds to the
    /// terrain.
    #[must_use]
    pub fn sim_bounds(&self) -> [f32; 4] {
        [
            self.sim_bounds_min_x.unwrap_or(0.0),
            self.sim_bounds_min_z.unwrap_or(0.0),
            self.sim_bounds_max_x.unwrap_or(f32::MAX),
            self.sim_bounds_max_z.unwrap_or(f32::MAX),
        ]
    }
    /// External trigger scripts in document order.
    #[must_use]
    pub fn external_trigger_scripts(&self) -> &[ExternalTriggerScript] {
        &self.external_trigger_scripts
    }

    /// Serialize this scenario data back to an XMB document.
    ///
    /// # Errors
    ///
    /// Returns an error if the scenario data cannot be serialized into an XMB node.
    pub fn to_document(&self) -> Result<xmb::Document, bdt_serde::Error> {
        let node = bdt_serde::to_node("Scenario", self)?;
        Ok(xmb::Document::with_root(node))
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
    #[must_use]
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
    #[must_use]
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
    pub(crate) fn terrain_base(&self) -> Option<String> {
        let base = self.file.strip_suffix(".scn")?;
        let base = base.replace('/', "\\");
        Some(format!("scenario\\{base}"))
    }

    /// Path to this scenario's `.sc2` object file.
    ///
    /// `BScenario_load` appends `.sc2` to the scenario path without `.scn`:
    /// `scenario\{scn_path_without_ext}.sc2`.
    #[must_use]
    pub fn sc2_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.sc2"))
    }

    /// Path to this scenario's `.sc3` object file.
    ///
    /// `scenario\{scn_path_without_ext}.sc3`
    #[must_use]
    pub fn sc3_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.sc3"))
    }

    /// Load and parse this scenario's `.sc2` object file.
    ///
    /// Returns `Ok(None)` if the source has no readable `.sc2` document. The
    /// engine then loads the scenario without it.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Scenario`] if the document does not match the
    /// object schema.
    pub fn read_sc2(
        &self,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> crate::Result<Option<ScenarioObjectFile>> {
        read_object_file(self.sc2_path(), src)
    }

    /// Load and parse this scenario's `.sc3` object file.
    ///
    /// Returns `Ok(None)` if the source has no readable `.sc3` document. The
    /// engine then loads the scenario without it.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Scenario`] if the document does not match the
    /// object schema.
    pub fn read_sc3(
        &self,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> crate::Result<Option<ScenarioObjectFile>> {
        read_object_file(self.sc3_path(), src)
    }

    /// Path to this scenario's XTD (terrain displacement/heightmap) file.
    ///
    /// Follows the engine's `BTerrainIOLoader::loadXTDInternal` logic:
    /// `scenario\{scn_path_without_ext}.xtd`
    #[must_use]
    pub fn xtd_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.xtd"))
    }

    /// Path to this scenario's XTT (terrain textures/foliage/roads) file.
    ///
    /// Follows the engine's `BTerrainIOLoader::loadXTTInternal` logic:
    /// `scenario\{scn_path_without_ext}.xtt`
    #[must_use]
    pub fn xtt_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.xtt"))
    }

    /// Path to this scenario's XSD (simulation terrain) file.
    ///
    /// `scenario\{scn_path_without_ext}.xsd`
    #[must_use]
    pub fn xsd_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.xsd"))
    }

    /// Load and decode this scenario's XSD simulation terrain.
    ///
    /// The source resolves the exact [`xsd_path`](Self::xsd_path), with no
    /// extension fallback. Returns `Ok(None)` if the descriptor has no `.scn`
    /// path or if the source has no XSD file.
    ///
    /// # Errors
    ///
    /// Returns an [`xsd::Error`] if the XSD file is malformed.
    pub fn read_simulation_terrain(
        &self,
        src: &mut AssetSource<impl assets::FileProvider>,
    ) -> Result<Option<xsd::Terrain>, xsd::Error> {
        let Some(path) = self.xsd_path() else {
            return Ok(None);
        };
        src.resolve_exact(&path)
            .map(|bytes| xsd::Terrain::read(&bytes))
            .transpose()
    }

    /// Path to this scenario's GLS (lightset) file.
    ///
    /// `scenario\{scn_path_without_ext}.gls`
    #[must_use]
    pub fn gls_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.gls"))
    }

    /// Path to this scenario's FLS (SH fill light) file.
    ///
    /// `scenario\{scn_path_without_ext}.fls`
    #[must_use]
    pub fn fls_path(&self) -> Option<String> {
        self.terrain_base().map(|b| format!("{b}.fls"))
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
    ///
    /// # Errors
    ///
    /// Returns an error if a scenario descriptor cannot be serialized into an XMB node.
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

/// Parse a `<Scenario>` XMB root node into [`ScenarioData`] via `bdt-serde`.
#[must_use]
pub fn parse_scenario_data(root: &bdt::Node) -> ScenarioData {
    bdt_serde::from_node(root).unwrap_or_default()
}

impl ScenarioData {
    /// Parse scenario data from an XML string.
    ///
    /// Convenience for tests and tools. Parses the XML into an XMB
    /// document, then deserialises the root `<Scenario>` node.
    #[must_use]
    pub fn from_xml_str(xml: &str) -> Option<Self> {
        let doc = xmb::Document::from_xml(xml).ok()?;
        let root = doc.root()?;
        Some(parse_scenario_data(root))
    }
}

/// Read a `.sc2` or `.sc3` document, or `None` if the path does not resolve.
fn read_object_file(
    path: Option<String>,
    src: &mut AssetSource<impl assets::FileProvider>,
) -> crate::Result<Option<ScenarioObjectFile>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let Some(document) = src.read_xmb(&path) else {
        return Ok(None);
    };
    let Some(root) = document.root() else {
        return Ok(None);
    };
    ScenarioObjectFile::from_node(root)
        .map(Some)
        .map_err(|source| crate::Error::Scenario { path, source })
}

/// Decode a retail boolean element that is present in the document.
///
/// Empty text gives `true`. Trimmed text `0` or `false` (any case) gives
/// `false`. All other text gives `true`.
fn retail_flag_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let text = String::deserialize(deserializer)?;
    let value = text.trim();
    Ok(value.is_empty() || (!value.eq_ignore_ascii_case("false") && value != "0"))
}

#[cfg(test)]
mod tests;
