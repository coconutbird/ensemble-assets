//! HW1 scenario descriptor parsing.
//!
//! Scenarios in HW1 are described in `scenariodescriptions.xml.xmb` which
//! lists all available maps. Each scenario has an associated ERA file
//! containing map-specific assets (terrain, lightmaps, trigger scripts).
//!
//! The engine loads scenario descriptors via `BScenarioList::load` which
//! reads `data\scenariodescriptions.xml.xmb` from the asset source.

use std::collections::HashMap;

use serde::Deserialize;

use crate::source::AssetSource;

/// A single scenario descriptor from `scenariodescriptions.xml`.
///
/// All fields are stored as attributes on the `<ScenarioInfo>` element.
#[derive(Debug, Clone, Default, Deserialize)]
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
}

/// All scenario descriptors loaded from the game data.
#[derive(Debug, Clone, Default)]
pub struct ScenarioList {
    /// All scenarios keyed by derived name.
    pub scenarios: HashMap<String, ScenarioDescriptor>,
}

impl ScenarioList {
    /// Load scenario descriptors from the asset source.
    ///
    /// Parses `data\scenariodescriptions.xml.xmb` and returns all scenario
    /// entries. Returns an empty list if the file is not found.
    pub fn load(assets: &mut AssetSource<impl assets::FileProvider>) -> Self {
        let mut list = Self::default();

        let Some(doc) = assets.read_xmb("data\\scenariodescriptions.xml") else {
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
}
