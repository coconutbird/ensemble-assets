use super::*;

#[test]
fn decodes_flags_counters_and_positive_final_counts() {
    let document = Document::from_xml(
        r#"<Scenario><Objectives>
            <Objective id="8">ojvCounter
                <Flag>Required</Flag><Flag>Player1</Flag><Flag>Player5</Flag><Flag>Player7</Flag>
                <Score>750</Score><TrackerDuration>9000</TrackerDuration>
                <MinTrackerIncrement>2</MinTrackerIncrement><FinalCount>100</FinalCount>
            </Objective>
            <Objective id="9">ojvNoCounter<FinalCount>0</FinalCount></Objective>
            <NotAnObjective id="10" />
        </Objectives></Scenario>"#,
    )
    .expect("objective XML");

    assert_eq!(
        parse_objectives(&document),
        Ok(vec![
            ObjectiveDefinition {
                id: 8,
                required: true,
                players: vec![1, 5],
                score: Some(750),
                tracker_duration_ms: Some(9_000),
                min_tracker_increment: Some(2),
                final_count: Some(100),
            },
            ObjectiveDefinition {
                id: 9,
                ..ObjectiveDefinition::default()
            },
        ])
    );
}

#[test]
fn later_fields_replace_earlier_fields_only_when_valid() {
    let document = Document::from_xml(
        r#"<Scenario><Objectives><Objective id="3">
            <Score>5</Score><Score>bad</Score>
            <TrackerDuration>7</TrackerDuration><TrackerDuration>-1</TrackerDuration>
            <MinTrackerIncrement>4</MinTrackerIncrement><MinTrackerIncrement>x</MinTrackerIncrement>
            <FinalCount>6</FinalCount><FinalCount>-2</FinalCount>
        </Objective></Objectives></Scenario>"#,
    )
    .expect("objective XML");

    assert_eq!(
        parse_objectives(&document),
        Ok(vec![ObjectiveDefinition {
            id: 3,
            score: Some(0),
            tracker_duration_ms: Some(7),
            min_tracker_increment: Some(4),
            final_count: Some(6),
            ..ObjectiveDefinition::default()
        }])
    );
}

#[test]
fn rejects_missing_and_malformed_objective_ids() {
    for (xml, expected) in [
        (
            "<Scenario><Objectives><Objective /></Objectives></Scenario>",
            ObjectiveError::MissingId,
        ),
        (
            "<Scenario><Objectives><Objective id=\"-1\" /></Objectives></Scenario>",
            ObjectiveError::MissingId,
        ),
        (
            "<Scenario><Objectives><Objective ID=\"4\" /></Objectives></Scenario>",
            ObjectiveError::MissingId,
        ),
        (
            "<Scenario><Objectives><Objective id=\"counter\" /></Objectives></Scenario>",
            ObjectiveError::InvalidId {
                value: "counter".to_owned(),
            },
        ),
    ] {
        let document = Document::from_xml(xml).expect("objective XML");
        assert_eq!(parse_objectives(&document), Err(expected), "{xml}");
    }
}
