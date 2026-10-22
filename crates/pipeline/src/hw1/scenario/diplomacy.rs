//! `<Diplomacies>` team relations of a `.scn` file.

use serde::{Deserialize, Serialize};

use super::retail_text::optional_integer;

/// Number of teams in the engine relation table (teams `0` to `4`).
pub const DIPLOMACY_TEAM_COUNT: usize = 5;

/// Team relation table: `table[team][other_team]`.
pub type DiplomacyTable = [[ScenarioRelation; DIPLOMACY_TEAM_COUNT]; DIPLOMACY_TEAM_COUNT];

/// `<Diplomacies>` wrapper containing `<Diplomacy>` children.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DiplomaciesWrapper {
    /// Relation rows from the `<Diplomacy>` children, in document order.
    #[serde(rename = "Diplomacy", default)]
    pub entries: Vec<ScenarioDiplomacy>,
}

impl DiplomaciesWrapper {
    /// Build the relation table like `sub_14037F840`, or `None` without rows.
    ///
    /// `BScenario_load` @ `0x140377be0` gives the table to
    /// `BScenario_setupPlayers` only if it read at least one `<Diplomacy>`
    /// element. The table starts with [`ScenarioRelation::Any`] (value `0`)
    /// in each cell. Each row and each entry with a team outside `0..=4`
    /// is ignored. A later entry replaces an earlier one.
    #[must_use]
    pub fn table(&self) -> Option<DiplomacyTable> {
        if self.entries.is_empty() {
            return None;
        }
        let mut table = [[ScenarioRelation::Any; DIPLOMACY_TEAM_COUNT]; DIPLOMACY_TEAM_COUNT];
        for row in &self.entries {
            let Some(team) = row.team.and_then(team_index) else {
                continue;
            };
            for entry in &row.relations {
                if let Some(other) = entry.id.and_then(team_index) {
                    table[team][other] = ScenarioRelation::from_name(&entry.relation);
                }
            }
        }
        Some(table)
    }
}

fn team_index(team: i32) -> Option<usize> {
    usize::try_from(team)
        .ok()
        .filter(|team| *team < DIPLOMACY_TEAM_COUNT)
}

/// One `<Diplomacy Team="N">` row with its `<Team ID="M">` entries.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioDiplomacy {
    /// Team of this row, or `None` if the attribute is absent or empty.
    #[serde(
        rename = "@Team",
        default,
        deserialize_with = "optional_integer",
        skip_serializing_if = "Option::is_none"
    )]
    pub team: Option<i32>,
    /// Relations of this team to other teams, from the `<Team>` children.
    #[serde(rename = "Team", default)]
    pub relations: Vec<ScenarioDiplomacyEntry>,
}

/// One `<Team ID="M">Relation</Team>` entry.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScenarioDiplomacyEntry {
    /// Other team, or `None` if the attribute is absent or empty.
    #[serde(
        rename = "@ID",
        default,
        deserialize_with = "optional_integer",
        skip_serializing_if = "Option::is_none"
    )]
    pub id: Option<i32>,
    /// Relation name (e.g. `"Ally"`), from the element text.
    #[serde(rename = "$text", default)]
    pub relation: String,
}

/// Team relation value, as `sub_1401F8680` converts a relation name.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum ScenarioRelation {
    /// `Any`, or a name that the engine does not know. The engine value is `0`.
    #[default]
    Any,
    /// `Self`. The engine value is `1`.
    SelfTeam,
    /// `Ally`. The engine value is `2`.
    Ally,
    /// `Enemy`. The engine value is `3`.
    Enemy,
    /// `Neutral`. The engine value is `4`.
    Neutral,
}

impl ScenarioRelation {
    /// Convert a relation name. The compare ignores case and does not trim the name.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        if name.eq_ignore_ascii_case("Self") {
            Self::SelfTeam
        } else if name.eq_ignore_ascii_case("Ally") {
            Self::Ally
        } else if name.eq_ignore_ascii_case("Enemy") {
            Self::Enemy
        } else if name.eq_ignore_ascii_case("Neutral") {
            Self::Neutral
        } else {
            Self::Any
        }
    }

    /// Integer value of the relation in the engine relation table.
    #[must_use]
    pub const fn engine_value(self) -> i32 {
        match self {
            Self::Any => 0,
            Self::SelfTeam => 1,
            Self::Ally => 2,
            Self::Enemy => 3,
            Self::Neutral => 4,
        }
    }
}
