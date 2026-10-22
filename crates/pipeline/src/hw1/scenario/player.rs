//! `<Player>` records of a `.scn` file, as `BScenario_loadPlayer` @ `0x1403808a0` reads them.

use serde::{Deserialize, Serialize};

use super::retail_text::{
    bool_or_false, bool_or_true, float_or_zero, integer_or_minus_one, minus_one, optional_float,
    strtol, yes,
};

/// A player definition in the scenario.
///
/// `BScenario_loadPlayer` @ `0x1403808a0` reads the attributes with
/// case-insensitive names. The field defaults are the engine defaults. The
/// engine resolves the names (civ, leader, prototypes, technologies and
/// population types) against the database. This struct keeps the names.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is one independent XML attribute of the serde schema"
)]
pub struct ScenarioPlayer {
    /// Player name (e.g. `"Player1"`, `"Creeps"`).
    #[serde(rename = "@Name", default)]
    pub name: String,
    /// Localised display name (e.g. `"25966,Rebels"`).
    ///
    /// The engine reads only the leading integer. See
    /// [`localised_display_name_id`](Self::localised_display_name_id).
    #[serde(
        rename = "@LocalisedDisplayName",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub localised_display_name: String,
    /// Whether the player gets the starting units of its leader.
    ///
    /// The engine default is `false`.
    #[serde(
        rename = "@UseStartingUnits",
        default,
        deserialize_with = "bool_or_false"
    )]
    pub use_starting_units: bool,
    /// Whether a lobby slot controls the player.
    ///
    /// The engine default is `true`.
    #[serde(
        rename = "@Controllable",
        default = "yes",
        deserialize_with = "bool_or_true"
    )]
    pub controllable: bool,
    /// Whether the lobby settings (civ, leader, team) replace the scenario values.
    ///
    /// The engine default is `false`.
    #[serde(
        rename = "@UsePlayerSettings",
        default,
        deserialize_with = "bool_or_false"
    )]
    pub use_player_settings: bool,
    /// Civilization name (e.g. `"UNSC"`, `"Covenant"`).
    ///
    /// The engine keeps civ index `1` when the attribute is absent or when
    /// the database has no civ with this name.
    #[serde(rename = "@Civ", default)]
    pub civ: String,
    /// Leader name, from the `Leader1` attribute or its alias `Leader`.
    ///
    /// The engine resolves the name with `sub_1401F1830`. It stores the result
    /// also when the database has no leader with this name (index `-1`).
    /// Retail files also have a `Leader2` attribute. The engine ignores it.
    #[serde(
        rename = "@Leader1",
        alias = "@Leader",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub leader: String,
    /// Player colour index. The engine default is `-1`.
    #[serde(
        rename = "@Color",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub color: i32,
    /// Team number. The engine default is `-1`.
    #[serde(
        rename = "@Team",
        default = "minus_one",
        deserialize_with = "integer_or_minus_one"
    )]
    pub team: i32,
    /// Whether the player keeps the starting resources of its leader.
    ///
    /// The engine default is `true`. If the value is `false`, the engine sets
    /// each resource to the scenario value. See [`resource`](Self::resource).
    #[serde(
        rename = "@DefaultResources",
        default = "yes",
        deserialize_with = "bool_or_true"
    )]
    pub default_resources: bool,
    /// Scenario amount of the `Supplies` resource.
    #[serde(
        rename = "@Supplies",
        default,
        deserialize_with = "optional_float",
        skip_serializing_if = "Option::is_none"
    )]
    pub supplies: Option<f32>,
    /// Scenario amount of the `Power` resource.
    #[serde(
        rename = "@Power",
        default,
        deserialize_with = "optional_float",
        skip_serializing_if = "Option::is_none"
    )]
    pub power: Option<f32>,
    /// Scenario amount of the `CampaignFoo` resource.
    #[serde(
        rename = "@CampaignFoo",
        default,
        deserialize_with = "optional_float",
        skip_serializing_if = "Option::is_none"
    )]
    pub campaign_foo: Option<f32>,
    /// Scenario amount of the `Collectable` resource.
    #[serde(
        rename = "@Collectable",
        default,
        deserialize_with = "optional_float",
        skip_serializing_if = "Option::is_none"
    )]
    pub collectable: Option<f32>,
    /// Population entries from the `<Pops>` child.
    #[serde(rename = "Pops", default)]
    pub pops: Option<PopsWrapper>,
    /// Forbidden object prototypes from the `<ForbidObjects>` child.
    #[serde(rename = "ForbidObjects", default)]
    pub forbid_objects: Option<ForbidObjectsWrapper>,
    /// Forbidden squad prototypes from the `<ForbidSquads>` child.
    #[serde(rename = "ForbidSquads", default)]
    pub forbid_squads: Option<ForbidSquadsWrapper>,
    /// Forbidden technologies from the `<ForbidTechs>` child.
    #[serde(rename = "ForbidTechs", default)]
    pub forbid_techs: Option<ForbidTechsWrapper>,
}

impl Default for ScenarioPlayer {
    fn default() -> Self {
        Self {
            name: String::new(),
            localised_display_name: String::new(),
            use_starting_units: false,
            controllable: true,
            use_player_settings: false,
            civ: String::new(),
            leader: String::new(),
            color: -1,
            team: -1,
            default_resources: true,
            supplies: None,
            power: None,
            campaign_foo: None,
            collectable: None,
            pops: None,
            forbid_objects: None,
            forbid_squads: None,
            forbid_techs: None,
        }
    }
}

impl ScenarioPlayer {
    /// Resource names that have a scenario attribute.
    ///
    /// These are the `<Resource>` names of the retail `gamedata.xml`. The
    /// engine reads an attribute for each database resource name. Retail
    /// files also have `Favor`, `Relics` and `Honor` attributes. No database
    /// resource has these names, so the engine ignores them.
    pub const RESOURCE_NAMES: [&'static str; 4] =
        ["Supplies", "Power", "CampaignFoo", "Collectable"];

    /// Scenario amount of the resource `name` (case-insensitive).
    ///
    /// An absent attribute or an unknown resource name gives `0.0`, the value
    /// of the zeroed engine resource array.
    #[must_use]
    pub fn resource(&self, name: &str) -> f32 {
        let value = if name.eq_ignore_ascii_case("Supplies") {
            self.supplies
        } else if name.eq_ignore_ascii_case("Power") {
            self.power
        } else if name.eq_ignore_ascii_case("CampaignFoo") {
            self.campaign_foo
        } else if name.eq_ignore_ascii_case("Collectable") {
            self.collectable
        } else {
            None
        };
        value.unwrap_or(0.0)
    }

    /// String-table ID of the localised display name.
    ///
    /// The engine converts the attribute text with `strtol`, so
    /// `"25966,Rebels"` gives `25966`. An empty attribute gives `None`.
    #[must_use]
    pub fn localised_display_name_id(&self) -> Option<i32> {
        (!self.localised_display_name.is_empty()).then(|| strtol(&self.localised_display_name))
    }

    /// Population entries in document order.
    #[must_use]
    pub fn pops(&self) -> &[ScenarioPop] {
        self.pops.as_ref().map_or(&[], |w| &w.entries)
    }

    /// Forbidden object prototype names in document order.
    #[must_use]
    pub fn forbidden_objects(&self) -> &[String] {
        self.forbid_objects.as_ref().map_or(&[], |w| &w.entries)
    }

    /// Forbidden squad prototype names in document order.
    #[must_use]
    pub fn forbidden_squads(&self) -> &[String] {
        self.forbid_squads.as_ref().map_or(&[], |w| &w.entries)
    }

    /// Forbidden technology names in document order.
    #[must_use]
    pub fn forbidden_techs(&self) -> &[String] {
        self.forbid_techs.as_ref().map_or(&[], |w| &w.entries)
    }
}

/// One `<Pop type="..." max="...">count</Pop>` entry of a player.
///
/// The engine keeps the entry only if the database has a population type
/// with this name. Retail files have entries with an empty `type`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioPop {
    /// Population type name from the `type` attribute (e.g. `"Unit"`).
    #[serde(rename = "@type", default)]
    pub pop_type: String,
    /// Population cap from the `max` attribute. The engine default is `0.0`.
    #[serde(rename = "@max", default, deserialize_with = "float_or_zero")]
    pub max: f32,
    /// Population count from the element text. The engine default is `0.0`.
    #[serde(rename = "$text", default, deserialize_with = "float_or_zero")]
    pub count: f32,
}

/// `<Pops>` wrapper containing `<Pop>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PopsWrapper {
    /// Population entries from the `<Pop>` children.
    #[serde(rename = "Pop", default)]
    pub entries: Vec<ScenarioPop>,
}

/// `<ForbidObjects>` wrapper containing `<Object>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ForbidObjectsWrapper {
    /// Object prototype names from the `<Object>` children.
    #[serde(rename = "Object", default)]
    pub entries: Vec<String>,
}

/// `<ForbidSquads>` wrapper containing `<Squad>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ForbidSquadsWrapper {
    /// Squad prototype names from the `<Squad>` children.
    #[serde(rename = "Squad", default)]
    pub entries: Vec<String>,
}

/// `<ForbidTechs>` wrapper containing `<Tech>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ForbidTechsWrapper {
    /// Technology names from the `<Tech>` children.
    #[serde(rename = "Tech", default)]
    pub entries: Vec<String>,
}
