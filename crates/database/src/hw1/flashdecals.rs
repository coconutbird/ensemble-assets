//! Parser for the Flash decal lists `hud_decal.xml.xmb` and `hud_decal_cov.xml.xmb`.
//!
//! The `<FlashResources>` of `art\ui\game\uigame.xml` give one decal list to each
//! civilization. `BUIGame_loadFlashResources` @ `0x1404c07c0` reads the `<Decals Type>` children.
//! The engine loads only the list of the civilization of the local player
//! (`BUIManager_initFlashDecals` @ `0x1404c73b0`). [`DECAL_LIST_SOURCES`] gives the shipped
//! entries.
//!
//! `BFlashMovieDefinition_load` @ `0x14022eea0` reads the file. A decal list uses only two kinds
//! of children of the `<FlashMovieDefinition>` root:
//!
//! - `<Properties>`: `<Data type name>` entries, for example the Flash movie of each decal set.
//! - `<Icon>`: `BFlashMovieIcon_load` @ `0x1402307b0` reads the attributes.
//!
//! `BFlashUIDecals_bindDecalIndices` @ `0x14021e710` binds each `uniticon`, `clumpdecal` and
//! `staticdecal` icon to the protosquad or the proto object that its `owner` names. The squad
//! name map wins, and implicit squads have the names of their objects. `BSquad_drawSelectionDecal`
//! @ `0x1403abea0` then uses the first bound icon of the squad (see
//! [`FlashDecalList::owner_icon`]).

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use super::Vector3;
use super::types::{deserialize_optional_float_text, deserialize_optional_vector3_text};
use crate::node_ext::expect_root;

/// One decal list of `<FlashResources>` in `art\ui\game\uigame.xml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecalListSource {
    /// The civilization name of the `Type` attribute.
    pub civ: &'static str,
    /// The game path of the `<Data>` file, without `.xmb`.
    pub path: &'static str,
}

/// The `<Decals>` entries of the shipped `uigame.xml`, in document order.
pub const DECAL_LIST_SOURCES: [DecalListSource; 2] = [
    DecalListSource {
        civ: "UNSC",
        path: "art\\ui\\flash\\hud\\hud_decal\\hud_decal.xml",
    },
    DecalListSource {
        civ: "Covenant",
        path: "art\\ui\\flash\\hud\\hud_decal\\hud_decal_cov.xml",
    },
];

/// The `type` of a unit icon. The engine compares the type with case (`strcmp`).
pub const UNIT_ICON: &str = "uniticon";
/// The `type` of a clump decal.
pub const CLUMP_DECAL: &str = "clumpdecal";
/// The `type` of a static decal.
pub const STATIC_DECAL: &str = "staticdecal";
/// The `type` of a miscellaneous icon (`owner` `circleselect` or `placeholder`).
pub const MISC_ICON: &str = "miscicon";

/// The decal list of one civilization.
#[derive(Debug, Clone, Default)]
pub struct CivDecalList {
    /// The civilization name of the `Type` attribute in `uigame.xml`.
    pub civ: String,
    /// The game path of the file, without `.xmb`.
    pub path: String,
    /// The contents of the file.
    pub list: FlashDecalList,
}

/// The contents of a decal list file (root `<FlashMovieDefinition>`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FlashDecalList {
    /// The `<Properties>` element.
    #[serde(rename = "Properties", skip_serializing_if = "Option::is_none")]
    pub properties: Option<FlashProperties>,
    /// The `<Icon>` elements, in document order.
    #[serde(rename = "Icon", default)]
    pub icons: Vec<FlashIcon>,
    /// Root text. The shipped `.xmb` files have the text `a`; the engine does not read it.
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl FlashDecalList {
    /// Iterate over the icons that the engine binds to a proto, in document order.
    ///
    /// These are the `uniticon`, `clumpdecal` and `staticdecal` icons with an `owner`. Each item
    /// is the owner name and the icon.
    pub fn owner_icons(&self) -> impl Iterator<Item = (&str, &FlashIcon)> {
        self.icons.iter().filter_map(|icon| {
            let kind = icon.kind.as_deref()?;
            if [UNIT_ICON, CLUMP_DECAL, STATIC_DECAL].contains(&kind) {
                icon.owner.as_deref().map(|owner| (owner, icon))
            } else {
                None
            }
        })
    }

    /// Find the icon that `BSquad_drawSelectionDecal` uses for the protosquad `name`.
    ///
    /// The engine puts the bound icons into a list in document order and uses the first one of
    /// the squad (`BFlashUIDecals_findIconIndex` @ `0x1404c7f10`). The owner compare ignores
    /// case and does not trim, as `BNameMap_find` @ `0x14008c7e0` does. An implicit squad has
    /// the name of its proto object.
    ///
    /// The engine stores the proto ID of each icon, not the name. An owner that names only a
    /// proto object (no protosquad and no implicit squad) stores the object ID, and a squad with
    /// the same ID then also finds that icon. No owner in the shipped lists does this.
    #[must_use]
    pub fn owner_icon(&self, name: &str) -> Option<&FlashIcon> {
        self.owner_icons()
            .find(|(owner, _)| owner.eq_ignore_ascii_case(name))
            .map(|(_, icon)| icon)
    }
}

/// The `<Properties>` element of a decal list.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FlashProperties {
    /// The `<Data>` entries, in document order.
    #[serde(rename = "Data", default)]
    pub data: Vec<FlashData>,
}

/// One `<Data type name>` entry, for example a `flashfile` or a `Texture`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FlashData {
    /// The kind of the entry, for example `flashfile` or `Texture`.
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// The name that icons use in their `name` attribute.
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The file path.
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// One `<Icon>` of a decal list.
///
/// `BFlashMovieIcon_load` @ `0x1402307b0` stores the sizes, the offset and the intensities as
/// half floats. An absent or empty number gives the default of the field.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct FlashIcon {
    /// The icon kind: [`UNIT_ICON`], [`CLUMP_DECAL`], [`STATIC_DECAL`] or [`MISC_ICON`].
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// The `<Data>` name of the Flash movie of the icon, for example `Infantry`.
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The `<Data>` name of the second Flash movie of a clump or static decal.
    #[serde(rename = "@name2", skip_serializing_if = "Option::is_none")]
    pub name2: Option<String>,
    /// The texture name.
    #[serde(rename = "@texture", skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
    /// Texture coordinate U. Default 0.
    #[serde(
        rename = "@U",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub u: Option<f32>,
    /// Texture coordinate V. Default 0.
    #[serde(
        rename = "@V",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub v: Option<f32>,
    /// Decal size along the right axis. Default 0.
    #[serde(
        rename = "@SizeX",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub size_x: Option<f32>,
    /// The height of the selection decal (the `YOffset` slot of the decal). Default 0.
    #[serde(
        rename = "@SizeY",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub size_y: Option<f32>,
    /// Decal size along the forward axis. Default 0.
    #[serde(
        rename = "@SizeZ",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub size_z: Option<f32>,
    /// Fourth size value. Default 0.
    #[serde(
        rename = "@SizeW",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub size_w: Option<f32>,
    /// The attribute `Intensity1` of the shipped files.
    ///
    /// The engine does not read it: it reads the misspelled name `Intenstiy1` (see
    /// [`FlashIcon::intenstiy1`]).
    #[serde(
        rename = "@Intensity1",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub intensity1: Option<f32>,
    /// The first intensity, from the misspelled attribute `Intenstiy1`. Default 1.
    ///
    /// `BWorld_drawPlacementDecal` @ `0x1405ce810` uses it as the intensity of the building
    /// placement decal. The selection decal does not use the icon intensities.
    #[serde(
        rename = "@Intenstiy1",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub intenstiy1: Option<f32>,
    /// The second intensity. Default 1.
    #[serde(
        rename = "@Intensity2",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub intensity2: Option<f32>,
    /// The third intensity. Default 1.
    #[serde(
        rename = "@Intensity3",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub intensity3: Option<f32>,
    /// The fourth intensity. Default 1.
    #[serde(
        rename = "@Intensity4",
        default,
        deserialize_with = "deserialize_optional_float_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub intensity4: Option<f32>,
    /// The offset of the decal center along the right, up and forward axes of the squad.
    /// Default zero.
    #[serde(
        rename = "@Offset",
        default,
        deserialize_with = "deserialize_optional_vector3_text",
        skip_serializing_if = "Option::is_none"
    )]
    pub offset: Option<Vector3>,
    /// The icon color (`BColor_parseText`).
    #[serde(rename = "@Color", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// The draw priority.
    #[serde(rename = "@Priority", skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    /// If true, the icon takes the player color. Default true.
    #[serde(rename = "@allowPlayerColor", skip_serializing_if = "Option::is_none")]
    pub allow_player_color: Option<bool>,
    /// The protosquad or proto object name that the icon belongs to.
    #[serde(rename = "@owner", skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// The element text. The engine reads it as an integer; the shipped icons have `-1`.
    #[serde(rename = "$text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl FlashIcon {
    /// Give `SizeX`, `SizeY` and `SizeZ`, with 0 for an absent value.
    #[must_use]
    pub fn size(&self) -> [f32; 3] {
        [self.size_x, self.size_y, self.size_z].map(|value| value.unwrap_or(0.0))
    }

    /// Give `Offset`, with the zero vector for an absent value.
    #[must_use]
    pub fn offset(&self) -> Vector3 {
        self.offset.unwrap_or_default()
    }

    /// Give the four intensities that the engine reads (`Intenstiy1`, `Intensity2`,
    /// `Intensity3` and `Intensity4`), with 1 for an absent value.
    #[must_use]
    pub fn intensities(&self) -> [f32; 4] {
        [
            self.intenstiy1,
            self.intensity2,
            self.intensity3,
            self.intensity4,
        ]
        .map(|value| value.unwrap_or(1.0))
    }
}

/// Parse a decal list document.
///
/// # Errors
///
/// Returns an error if the root is not `<FlashMovieDefinition>` or an icon cannot be
/// deserialized.
pub fn parse(doc: &xmb::Document) -> crate::Result<FlashDecalList> {
    let root = expect_root(doc, "FlashMovieDefinition")?;
    Ok(bdt_serde::from_node(root)?)
}
