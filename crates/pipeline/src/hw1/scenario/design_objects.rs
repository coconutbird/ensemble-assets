//! Decoding of the SCN `DesignObjects` spheres and lines.
//!
//! `BDesignObjectManager_load` @ `0x140200530` reads every child of
//! `<DesignObjects>`. A child named `Spheres` holds spheres and a child named
//! `Lines` holds lines (case-insensitive names). Each child of such a wrapper
//! is one object, whatever its element name. The engine keeps the spheres and
//! the lines in two arrays in document order.

use std::fmt;

use xmb::{Document, Node};

/// The design spheres and lines of one SCN file, in document order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DesignObjects {
    /// Spheres from all `DesignObjects/Spheres` wrappers.
    pub spheres: Vec<DesignSphere>,
    /// Lines from all `DesignObjects/Lines` wrappers.
    pub lines: Vec<DesignLine>,
}

/// One authored design sphere (`BDesignSphere_load` @ `0x1401fedd0`).
#[derive(Debug, Clone, PartialEq)]
pub struct DesignSphere {
    /// The `ID` attribute.
    pub id: i32,
    /// The `Position` attribute in SCN axes. The default is the origin.
    pub position: [f32; 3],
    /// The `Radius` attribute. The default is 0.
    pub radius: f32,
    /// The `Data` child.
    pub data: DesignData,
}

/// The `Data` child of a design object (`BDesignObjectData_load` @ `0x1401fdf50`).
///
/// A missing or empty element keeps the default of 0 or the empty string. A
/// later `Data` child replaces the values of an earlier one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DesignData {
    /// The `Type` text. Triggers select spheres by this text
    /// (`CanGetDesignSpheres`).
    pub type_name: String,
    /// The `Attract` value.
    pub attract: f32,
    /// The `Repel` value.
    pub repel: f32,
    /// The `ChokePoint` value. The engine gives each player AI the spheres with
    /// a value above 0.
    pub choke_point: f32,
    /// The `Value` value.
    pub value: f32,
    /// The `Value1` to `Value12` values.
    pub values: [f32; 12],
}

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

/// Failure while decoding one SCN design object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesignObjectError {
    /// A design object has no `ID` attribute.
    MissingId {
        /// The kind of the object: `sphere` or `line`.
        kind: &'static str,
    },
    /// The `ID` attribute of a design object is not a signed integer.
    InvalidId {
        /// The kind of the object: `sphere` or `line`.
        kind: &'static str,
        /// Text of the `ID` attribute.
        value: String,
    },
    /// A position or path point does not contain three finite floats.
    InvalidVector {
        /// ID of the object with the invalid vector.
        object_id: i32,
        /// Source of the vector: `Position` or `Points`.
        field: &'static str,
        /// Text of the invalid vector.
        value: String,
    },
    /// A number field does not contain one finite float.
    InvalidNumber {
        /// ID of the object with the invalid number.
        object_id: i32,
        /// Name of the attribute or `Data` child.
        field: String,
        /// Text of the invalid number.
        value: String,
    },
}

impl fmt::Display for DesignObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingId { kind } => write!(f, "a design {kind} is missing its ID attribute"),
            Self::InvalidId { kind, value } => {
                write!(f, "design-{kind} ID '{value}' is not a valid integer")
            }
            Self::InvalidVector {
                object_id,
                field,
                value,
            } => write!(
                f,
                "design object {object_id} has invalid {field} vector '{value}'"
            ),
            Self::InvalidNumber {
                object_id,
                field,
                value,
            } => write!(
                f,
                "design object {object_id} has invalid {field} number '{value}'"
            ),
        }
    }
}

impl std::error::Error for DesignObjectError {}

const DATA_VALUE_NAMES: [&str; 12] = [
    "Value1", "Value2", "Value3", "Value4", "Value5", "Value6", "Value7", "Value8", "Value9",
    "Value10", "Value11", "Value12",
];

/// Decode the design spheres and lines of a `<Scenario>` document.
///
/// Returns empty lists if the document has no `DesignObjects` element.
///
/// # Errors
///
/// Returns the [`DesignObjectError`] of the first malformed object.
pub fn parse_design_objects(document: &Document) -> Result<DesignObjects, DesignObjectError> {
    let mut objects = DesignObjects::default();
    let Some(design_objects) = document.root().and_then(|root| {
        root.children
            .iter()
            .find(|child| child.name == "DesignObjects")
    }) else {
        return Ok(objects);
    };
    for wrapper in &design_objects.children {
        if wrapper.name.eq_ignore_ascii_case("Spheres") {
            for node in &wrapper.children {
                objects.spheres.push(parse_design_sphere(node)?);
            }
        } else if wrapper.name.eq_ignore_ascii_case("Lines") {
            for node in &wrapper.children {
                objects.lines.push(parse_design_line(node)?);
            }
        }
    }
    Ok(objects)
}

fn parse_design_sphere(node: &Node) -> Result<DesignSphere, DesignObjectError> {
    let id = parse_id(node, "sphere")?;
    let position = node
        .get_attribute("Position")
        .map(|position| parse_point(id, "Position", &position.value_string()))
        .transpose()?
        .unwrap_or_default();
    let radius = node
        .get_attribute("Radius")
        .map(|radius| parse_number(id, "Radius", &radius.value_string()))
        .transpose()?
        .unwrap_or_default();
    let mut data = DesignData::default();
    for data_node in node
        .children
        .iter()
        .filter(|child| child.name.eq_ignore_ascii_case("Data"))
    {
        parse_design_data(id, data_node, &mut data)?;
    }
    Ok(DesignSphere {
        id,
        position,
        radius,
        data,
    })
}

fn parse_design_data(
    object_id: i32,
    node: &Node,
    data: &mut DesignData,
) -> Result<(), DesignObjectError> {
    for child in &node.children {
        let name = child.name.as_str();
        if name.eq_ignore_ascii_case("Type") {
            data.type_name = child.text_string();
            continue;
        }
        let slot = if name.eq_ignore_ascii_case("Attract") {
            &mut data.attract
        } else if name.eq_ignore_ascii_case("Repel") {
            &mut data.repel
        } else if name.eq_ignore_ascii_case("ChokePoint") {
            &mut data.choke_point
        } else if name.eq_ignore_ascii_case("Value") {
            &mut data.value
        } else if let Some(index) = DATA_VALUE_NAMES
            .iter()
            .position(|value_name| name.eq_ignore_ascii_case(value_name))
        {
            &mut data.values[index]
        } else {
            continue;
        };
        let text = child.text_string();
        if !text.trim().is_empty() {
            *slot = parse_number(object_id, name, &text)?;
        }
    }
    Ok(())
}

fn parse_design_line(node: &Node) -> Result<DesignLine, DesignObjectError> {
    let id = parse_id(node, "line")?;
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

fn parse_id(node: &Node, kind: &'static str) -> Result<i32, DesignObjectError> {
    let id_attribute = node
        .get_attribute("ID")
        .ok_or(DesignObjectError::MissingId { kind })?;
    let id_text = id_attribute.value_string();
    id_text
        .trim()
        .parse()
        .map_err(|_| DesignObjectError::InvalidId {
            kind,
            value: id_text,
        })
}

fn parse_number(object_id: i32, field: &str, value: &str) -> Result<f32, DesignObjectError> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|number| number.is_finite())
        .ok_or_else(|| DesignObjectError::InvalidNumber {
            object_id,
            field: field.to_owned(),
            value: value.to_owned(),
        })
}

fn parse_point(
    object_id: i32,
    field: &'static str,
    value: &str,
) -> Result<[f32; 3], DesignObjectError> {
    let invalid = || DesignObjectError::InvalidVector {
        object_id,
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

#[cfg(test)]
mod tests;
