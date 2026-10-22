use super::*;

#[test]
fn allow_veterancy_matches_retail_missing_and_present_defaults() {
    for (xml, expected) in [
        ("<Scenario />", false),
        ("<Scenario><AllowVeterancy /></Scenario>", true),
        (
            "<Scenario><AllowVeterancy>true</AllowVeterancy></Scenario>",
            true,
        ),
        (
            "<Scenario><AllowVeterancy>1</AllowVeterancy></Scenario>",
            true,
        ),
        (
            "<Scenario><AllowVeterancy> FALSE </AllowVeterancy></Scenario>",
            false,
        ),
        (
            "<Scenario><AllowVeterancy>false</AllowVeterancy></Scenario>",
            false,
        ),
        (
            "<Scenario><AllowVeterancy>0</AllowVeterancy></Scenario>",
            false,
        ),
    ] {
        let scenario = ScenarioData::from_xml_str(xml).expect("valid scenario XML");
        assert_eq!(scenario.allow_veterancy, expected, "{xml}");
    }
}

#[test]
fn visual_variation_index_keeps_absent_and_explicit_zero_apart() {
    let scenario = ScenarioData::from_xml_str(
        r#"<Scenario><Objects>
            <Object ID="10" VisualVariationIndex="2">marine</Object>
            <Object ID="11">random</Object>
            <Object ID="12" VisualVariationIndex="0">first</Object>
            <Object ID="13" VisualVariationIndex="-2">negative</Object>
            <Object ID="14" VisualVariationIndex="bad">bad_index</Object>
        </Objects></Scenario>"#,
    )
    .expect("valid scenario XML");

    assert_eq!(
        scenario
            .objects()
            .iter()
            .map(|object| (object.id, object.visual_variation_index))
            .collect::<Vec<_>>(),
        vec![
            (10, Some(2)),
            (11, None),
            (12, Some(0)),
            (13, Some(-2)),
            (14, None),
        ]
    );
}

#[test]
fn serialization_keeps_absent_authored_flags_absent() {
    let scenario = ScenarioData::from_xml_str(
        r#"<Scenario><Objects>
            <Object ID="10" VisualVariationIndex="0">first</Object>
            <Object ID="11">random</Object>
        </Objects></Scenario>"#,
    )
    .expect("valid scenario XML");

    let document = scenario.to_document().expect("serializable scenario");
    let root = document.root().expect("scenario root");
    assert!(
        root.children
            .iter()
            .all(|child| child.name != "AllowVeterancy")
    );
    let reparsed = parse_scenario_data(root);
    assert_eq!(
        reparsed
            .objects()
            .iter()
            .map(|object| object.visual_variation_index)
            .collect::<Vec<_>>(),
        vec![Some(0), None]
    );
    assert!(!reparsed.allow_veterancy);

    let enabled = ScenarioData {
        allow_veterancy: true,
        ..ScenarioData::default()
    };
    let document = enabled.to_document().expect("serializable scenario");
    assert!(parse_scenario_data(document.root().expect("scenario root")).allow_veterancy);
}

#[test]
fn xsd_path_follows_the_terrain_base_path() {
    let descriptor = ScenarioDescriptor {
        file: "skirmish/design/blood_gulch/blood_gulch.scn".to_owned(),
        ..ScenarioDescriptor::default()
    };
    assert_eq!(
        descriptor.xsd_path().as_deref(),
        Some("scenario\\skirmish\\design\\blood_gulch\\blood_gulch.xsd")
    );
    assert_eq!(ScenarioDescriptor::default().xsd_path(), None);
}
