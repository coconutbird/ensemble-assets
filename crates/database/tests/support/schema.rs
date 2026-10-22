//! Schema round-trip helpers that the database integration tests share.

use std::collections::BTreeSet;

use serde::{Serialize, de::DeserializeOwned};

/// The values and the warnings of one schema round trip.
pub struct NodeRoundTrip<T> {
    /// The value from the source node.
    pub original: T,
    /// The value after serialization and a second deserialization.
    pub round_trip: T,
    /// The unmapped-field warnings from the source node.
    pub warnings: Vec<String>,
    /// The unmapped-field warnings from the serialized node.
    pub round_trip_warnings: Vec<String>,
}

/// Deserialize `root`, serialize the value with the same root name and deserialize it again.
///
/// `label` identifies the source in panic messages.
///
/// # Panics
///
/// Panics if a deserialization or the serialization fails.
pub fn round_trip_node<T>(label: &str, root: &xmb::Node) -> NodeRoundTrip<T>
where
    T: DeserializeOwned + Serialize,
{
    let (original, warnings) = bdt_serde::from_node_warned::<T>(root)
        .unwrap_or_else(|error| panic!("failed to deserialize {label}: {error}"));
    let serialized = bdt_serde::to_node(root.name.as_str(), &original)
        .unwrap_or_else(|error| panic!("failed to serialize {label}: {error}"));
    let (round_trip, round_trip_warnings) = bdt_serde::from_node_warned::<T>(&serialized)
        .unwrap_or_else(|error| panic!("failed to deserialize round trip for {label}: {error}"));
    NodeRoundTrip {
        original,
        round_trip,
        warnings: warnings.iter().map(ToString::to_string).collect(),
        round_trip_warnings: round_trip_warnings
            .iter()
            .map(ToString::to_string)
            .collect(),
    }
}

/// Parse synthetic XML, check its root name and round trip its root node.
///
/// The function returns the original value and the round-trip value, in that order. Each caller
/// selects the value that it examines.
///
/// # Panics
///
/// Panics if the XML does not parse, if the root is not `root_name`, if a step of
/// [`round_trip_node`] fails, or if a step gives an unmapped-field warning.
pub fn assert_xml_round_trip<T>(label: &str, xml: &str, root_name: &str) -> (T, T)
where
    T: DeserializeOwned + Serialize,
{
    let document = xmb::Document::from_xml(xml)
        .unwrap_or_else(|error| panic!("failed to read {label}: {error}"));
    let root = document
        .root()
        .unwrap_or_else(|| panic!("{label} has no root node"));
    assert_eq!(root.name, root_name, "unexpected root in {label}");
    let NodeRoundTrip {
        original,
        round_trip,
        warnings,
        round_trip_warnings,
    } = round_trip_node(label, root);
    assert_no_warnings(&format!("unmapped schema in {label}"), warnings);
    assert_no_warnings(
        &format!("round-trip schema warnings in {label}"),
        round_trip_warnings,
    );
    (original, round_trip)
}

fn assert_no_warnings(message: &str, warnings: Vec<String>) {
    let warnings = warnings.into_iter().collect::<BTreeSet<_>>();
    assert!(
        warnings.is_empty(),
        "{message}:\n{}",
        warnings.into_iter().collect::<Vec<_>>().join("\n")
    );
}
