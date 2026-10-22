//! Decoding of the SCN `DesignObjects/Lines` path geometry.

use std::fmt;

use xmb::{Document, Node};

/// One authored design line from the SCN file.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignLine {
    /// The integer ID that triggers use to find the line.
    pub id: i32,
    /// The line points in SCN axes.
    ///
    /// The `Position` attribute gives the first point, if it is present. The
    /// `|`-separated entries of each `Points` child follow in document order.
    pub points: Vec<[f32; 3]>,
}

/// Failure while decoding one SCN design line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesignLineError {
    /// A line has no `ID` attribute.
    MissingId,
    /// The `ID` attribute of a line is not a signed integer.
    InvalidId {
        /// Text of the `ID` attribute.
        value: String,
    },
    /// A position or path point does not contain three finite floats.
    InvalidVector {
        /// ID of the line with the invalid vector.
        line_id: i32,
        /// Source of the vector: `Position` or `Points`.
        field: &'static str,
        /// Text of the invalid vector.
        value: String,
    },
}

impl fmt::Display for DesignLineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingId => f.write_str("a design line is missing its ID attribute"),
            Self::InvalidId { value } => {
                write!(f, "design-line ID '{value}' is not a valid integer")
            }
            Self::InvalidVector {
                line_id,
                field,
                value,
            } => write!(
                f,
                "design line {line_id} has invalid {field} vector '{value}'"
            ),
        }
    }
}

impl std::error::Error for DesignLineError {}

/// Decode the design lines of a `<Scenario>` document in document order.
///
/// Returns an empty list if the document has no `DesignObjects/Lines` element.
/// Each child of that element is one line.
///
/// # Errors
///
/// Returns the [`DesignLineError`] of the first malformed line.
pub fn parse_design_lines(document: &Document) -> Result<Vec<DesignLine>, DesignLineError> {
    let Some(wrapper) = document
        .root()
        .and_then(|root| child(root, "DesignObjects"))
        .and_then(|objects| child(objects, "Lines"))
    else {
        return Ok(Vec::new());
    };
    wrapper.children.iter().map(parse_design_line).collect()
}

fn parse_design_line(node: &Node) -> Result<DesignLine, DesignLineError> {
    let id_attribute = node.get_attribute("ID").ok_or(DesignLineError::MissingId)?;
    let id_text = id_attribute.value_string();
    let id = id_text
        .trim()
        .parse()
        .map_err(|_| DesignLineError::InvalidId { value: id_text })?;
    let mut points = Vec::new();
    if let Some(position) = node.get_attribute("Position") {
        points.push(parse_point(id, "Position", &position.value_string())?);
    }
    for points_node in node.children.iter().filter(|node| node.name == "Points") {
        for point in points_node
            .text_string()
            .split('|')
            .map(str::trim)
            .filter(|point| !point.is_empty())
        {
            points.push(parse_point(id, "Points", point)?);
        }
    }
    Ok(DesignLine { id, points })
}

fn parse_point(
    line_id: i32,
    field: &'static str,
    value: &str,
) -> Result<[f32; 3], DesignLineError> {
    let invalid = || DesignLineError::InvalidVector {
        line_id,
        field,
        value: value.to_owned(),
    };
    let mut components = value.split(',').map(str::trim);
    let parse_component = |component: Option<&str>| {
        component
            .and_then(|component| component.parse::<f32>().ok())
            .filter(|component| component.is_finite())
            .ok_or_else(invalid)
    };
    let point = [
        parse_component(components.next())?,
        parse_component(components.next())?,
        parse_component(components.next())?,
    ];
    if components.next().is_some() {
        return Err(invalid());
    }
    Ok(point)
}

fn child<'a>(node: &'a Node, name: &str) -> Option<&'a Node> {
    node.children.iter().find(|child| child.name == name)
}

#[cfg(test)]
mod tests;
