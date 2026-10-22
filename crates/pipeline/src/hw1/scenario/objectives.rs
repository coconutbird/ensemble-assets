//! Decoding of the SCN `Objectives` definitions.

use std::fmt;

use xmb::{Document, Node};

/// One authored objective from the SCN file.
///
/// Each child element of an `<Objective>` is read in document order. A later
/// element replaces the value of an earlier element with the same name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectiveDefinition {
    /// The objective ID from the lowercase `id` attribute.
    pub id: i32,
    /// `true` if a `Flag` element has the text `Required`.
    pub required: bool,
    /// The players from the `Player1` to `Player6` flags, in document order.
    pub players: Vec<u8>,
    /// The value of the last `Score` element.
    ///
    /// Text that is not an `i32` gives `0`.
    pub score: Option<i32>,
    /// The last `TrackerDuration` value that is a valid `u32`, in milliseconds.
    pub tracker_duration_ms: Option<u32>,
    /// The last `MinTrackerIncrement` value that is a valid `u32`.
    pub min_tracker_increment: Option<u32>,
    /// The last `FinalCount` value that is a positive `i32`.
    pub final_count: Option<i32>,
}

/// Failure while decoding the identity of one SCN objective.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectiveError {
    /// An objective has no lowercase `id` attribute, or its ID is `-1`.
    MissingId,
    /// The ID of an objective is not a signed integer.
    InvalidId {
        /// Text of the `id` attribute.
        value: String,
    },
}

impl fmt::Display for ObjectiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingId => f.write_str("a scenario objective is missing its id attribute"),
            Self::InvalidId { value } => {
                write!(f, "objective ID '{value}' is not a valid integer")
            }
        }
    }
}

impl std::error::Error for ObjectiveError {}

/// Decode the `Objective` children of the `Objectives` element in document order.
///
/// Returns an empty list if the document has no `Objectives` element.
///
/// # Errors
///
/// Returns the [`ObjectiveError`] of the first objective with a missing or
/// malformed ID.
pub fn parse_objectives(document: &Document) -> Result<Vec<ObjectiveDefinition>, ObjectiveError> {
    let Some(wrapper) = document.root().and_then(|root| {
        root.children
            .iter()
            .find(|child| child.name == "Objectives")
    }) else {
        return Ok(Vec::new());
    };
    wrapper
        .children
        .iter()
        .filter(|node| node.name == "Objective")
        .map(parse_objective)
        .collect()
}

fn parse_objective(node: &Node) -> Result<ObjectiveDefinition, ObjectiveError> {
    let attribute = node.get_attribute("id").ok_or(ObjectiveError::MissingId)?;
    let id_text = attribute.value_string();
    let id = id_text
        .trim()
        .parse()
        .map_err(|_| ObjectiveError::InvalidId { value: id_text })?;
    if id == -1 {
        return Err(ObjectiveError::MissingId);
    }

    let mut objective = ObjectiveDefinition {
        id,
        ..ObjectiveDefinition::default()
    };
    for field in &node.children {
        let text = field.text_string();
        let value = text.trim();
        match field.name.as_str() {
            "Flag" => apply_flag(&mut objective, value),
            "Score" => objective.score = Some(value.parse().unwrap_or(0)),
            "TrackerDuration" => {
                if let Ok(duration) = value.parse() {
                    objective.tracker_duration_ms = Some(duration);
                }
            }
            "MinTrackerIncrement" => {
                if let Ok(increment) = value.parse() {
                    objective.min_tracker_increment = Some(increment);
                }
            }
            "FinalCount" => {
                if let Ok(count) = value.parse()
                    && count > 0
                {
                    objective.final_count = Some(count);
                }
            }
            _ => {}
        }
    }
    Ok(objective)
}

fn apply_flag(objective: &mut ObjectiveDefinition, flag: &str) {
    let player = match flag {
        "Required" => {
            objective.required = true;
            return;
        }
        "Player1" => 1,
        "Player2" => 2,
        "Player3" => 3,
        "Player4" => 4,
        "Player5" => 5,
        "Player6" => 6,
        _ => return,
    };
    objective.players.push(player);
}

#[cfg(test)]
mod tests;
