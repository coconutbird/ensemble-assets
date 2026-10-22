//! Parser for `playercolors.xml.xmb`: the player color tables.
//!
//! `BDatabase_setupPlayerColors` @ `0x1401f9560` loads the file. It does not check the root
//! name. It reads these children of the root (the compares ignore case):
//!
//! - `<skirmish>` and `<spc>`: one color set each (table 0 and table 1). A `<color num>` child
//!   writes entry `num` (0 to 15) of the set. The engine ignores a `<color>` without `num` or
//!   with a `num` above 15. `<friendOrFoeSelf>`, `<friendOrFoeAlly>`, `<friendOrFoeNeutral>`
//!   and `<friendOrFoeEnemy>` give the relation colors that `BWorld_getPlayerColor`
//!   @ `0x1405d4150` uses when the profile option for relation colors is on.
//! - `<civ>`: the default color of each player slot for one civilization.
//!
//! `BPlayerColor_loadEntry` @ `0x1402ffdd0` reads the attributes `objects`, `corpse`, `selection`,
//! `minimap` and `ui` of each color. An absent attribute gives white (`0xFFFFFFFF`).

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use super::RgbColor;
use crate::node_ext::root_node;

/// Number of color entries in a set: the engine accepts `num` from 0 to 15.
pub const PLAYER_COLOR_COUNT: usize = 16;

/// The color tables from `playercolors.xml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlayerColors {
    /// Colors of skirmish games (engine table 0).
    #[serde(rename = "skirmish")]
    pub skirmish: Option<PlayerColorSet>,
    /// Colors of campaign games (engine table 1).
    #[serde(rename = "spc")]
    pub spc: Option<PlayerColorSet>,
    /// Default color of each player slot, by civilization, in document order.
    #[serde(rename = "civ", default)]
    pub civs: Vec<PlayerColorCiv>,
}

/// One color set (`<skirmish>` or `<spc>`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlayerColorSet {
    /// The `<color>` entries, in document order. Use [`PlayerColorSet::color`] to get the entry
    /// that the engine keeps for a number.
    #[serde(rename = "color", default)]
    pub colors: Vec<PlayerColor>,
    /// Color of the local player when relation colors are on.
    #[serde(rename = "friendOrFoeSelf")]
    pub friend_or_foe_self: Option<PlayerColor>,
    /// Color of allies when relation colors are on.
    #[serde(rename = "friendOrFoeAlly")]
    pub friend_or_foe_ally: Option<PlayerColor>,
    /// Color of neutral players when relation colors are on.
    #[serde(rename = "friendOrFoeNeutral")]
    pub friend_or_foe_neutral: Option<PlayerColor>,
    /// Color of enemies when relation colors are on.
    #[serde(rename = "friendOrFoeEnemy")]
    pub friend_or_foe_enemy: Option<PlayerColor>,
}

impl PlayerColorSet {
    /// Give the color that the engine keeps for entry `num`.
    ///
    /// The engine writes each `<color>` to its entry in document order, so the last `<color>`
    /// with this `num` wins. Returns `None` if `num` is above 15 or no `<color>` has this
    /// number.
    #[must_use]
    pub fn color(&self, num: usize) -> Option<&PlayerColor> {
        if num >= PLAYER_COLOR_COUNT {
            return None;
        }
        self.colors
            .iter()
            .rev()
            .find(|color| color.num.and_then(|value| usize::try_from(value).ok()) == Some(num))
    }
}

/// One color entry: the colors of a player in each use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct PlayerColor {
    /// Entry number (0 to 15) of a `<color>`. The relation colors do not have a number.
    #[serde(rename = "@num", skip_serializing_if = "Option::is_none")]
    pub num: Option<u32>,
    /// Color of units and buildings. Default white.
    #[serde(rename = "@objects", default = "white")]
    pub objects: RgbColor,
    /// Color of corpses. Default white.
    #[serde(rename = "@corpse", default = "white")]
    pub corpse: RgbColor,
    /// Color of the selection decal. Default white.
    #[serde(rename = "@selection", default = "white")]
    pub selection: RgbColor,
    /// Color on the minimap. Default white.
    #[serde(rename = "@minimap", default = "white")]
    pub minimap: RgbColor,
    /// Color in the UI (`BWorld_getPlayerColor` kind 1). Default white.
    #[serde(rename = "@ui", default = "white")]
    pub ui: RgbColor,
}

impl Default for PlayerColor {
    fn default() -> Self {
        Self {
            num: None,
            objects: white(),
            corpse: white(),
            selection: white(),
            minimap: white(),
            ui: white(),
        }
    }
}

/// One `<civ>`: the default color entry of each player slot for one civilization.
///
/// `BDatabase_setupPlayerColors` finds the civilization by the element text with
/// `BDatabase_getCivIDFromName` @ `0x1401f8940` and ignores a civ ID above 4. The shipped file
/// has `UNSC`, then `Covenant`. The slot table of a civ has 9 entries.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlayerColorCiv {
    /// Civilization name from the element text, without the surrounding white space.
    #[serde(rename = "$text", default, deserialize_with = "trimmed_text")]
    pub name: String,
    /// The `<player>` entries, in document order.
    #[serde(rename = "player", default)]
    pub players: Vec<PlayerColorSlot>,
}

/// One `<player num color>` of a `<civ>`.
///
/// The engine accepts `num` from 0 to 8 and `color` from 0 to 15.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct PlayerColorSlot {
    /// Player slot.
    #[serde(rename = "@num", skip_serializing_if = "Option::is_none")]
    pub num: Option<u32>,
    /// Color entry number of the slot.
    #[serde(rename = "@color", skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
}

const fn white() -> RgbColor {
    RgbColor {
        red: 255,
        green: 255,
        blue: 255,
    }
}

/// Parse a `playercolors.xml.xmb` document.
///
/// # Errors
///
/// Returns an error if the document has no root or a color cannot be deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<PlayerColors> {
    Ok(bdt_serde::from_node(root_node(doc)?)?)
}

/// Deserialize element text without the white space around it. An XML file with text before
/// the children of an element also gives the indentation between the children.
fn trimmed_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    Ok(String::from(text.trim()))
}
