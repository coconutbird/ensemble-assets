//! Parser for `gamemodes.xml.xmb`: the skirmish game modes.
//!
//! `BDatabase_setupGameModes` loads one game mode for each `<GameMode>` child of the root (the
//! tag compare ignores case). `BGameMode_load` @ `0x140240e70` reads the attributes `Locked`
//! and `dlc`, and the child elements below. The last element of a name wins. Engine defaults:
//! empty strings, `false` flags and `-1` for the string IDs.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node_ext::expect_root;

/// One game mode from `gamemodes.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GameMode {
    /// Game mode name, for example `"Skirmish"`.
    #[serde(rename = "Name", default)]
    pub name: String,
    /// `true` if the menu shows the mode as locked (attribute `Locked`).
    #[serde(rename = "@Locked")]
    pub locked: Option<bool>,
    /// `true` if the mode comes from downloadable content (attribute `dlc`).
    #[serde(rename = "@dlc")]
    pub dlc: Option<bool>,
    /// Trigger script that runs once for the world, for example
    /// `"SkirmishWorld.triggerscript"`.
    #[serde(rename = "WorldScript")]
    pub world_script: Option<String>,
    /// Trigger script that runs once for each player, for example
    /// `"SkirmishPlayer.triggerscript"`.
    #[serde(rename = "PlayerScript")]
    pub player_script: Option<String>,
    /// Name of a tech. The engine stores its tech ID, or `-1` if no tech has this name.
    #[serde(rename = "Tech")]
    pub tech: Option<String>,
    /// String ID of the display name.
    #[serde(rename = "DisplayNameID")]
    pub display_name_id: Option<i32>,
    /// String ID of the short description.
    #[serde(rename = "DescriptionID")]
    pub description_id: Option<i32>,
    /// String ID of the long description.
    #[serde(rename = "LongDescriptionID")]
    pub long_description_id: Option<i32>,
    /// Text of the `NPC` element. The engine keeps it as a string.
    #[serde(rename = "NPC")]
    pub npc: Option<String>,
}

impl GameMode {
    /// Return the tech name, or `None` if the `Tech` element is absent or empty.
    #[must_use]
    pub fn tech_name(&self) -> Option<&str> {
        self.tech.as_deref().filter(|name| !name.is_empty())
    }
}

/// Parse all game modes from a `gamemodes.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the root is not `GameModes` or a game mode cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<Vec<GameMode>> {
    let root = expect_root(doc, "GameModes")?;
    let modes = root
        .children
        .iter()
        .filter(|child| child.name.eq_ignore_ascii_case("GameMode"))
        .map(bdt_serde::from_node)
        .collect::<Result<_, _>>()?;
    Ok(modes)
}
