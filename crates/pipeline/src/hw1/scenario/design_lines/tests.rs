use super::*;

#[test]
fn decodes_position_then_points_in_authored_axes() {
    let document = Document::from_xml(
        r#"<Scenario><DesignObjects><Lines>
            <Lines ID=" 7 " Position="1,2,3">
                <Data />
                <Points>4,5,6| 7, 8, 9 |</Points>
                <Points>10,11,12</Points>
            </Lines>
            <Lines ID="8"><Points /></Lines>
            <Lines ID="4" Position="1,2,3" />
            <Lines ID="4" Position="10,20,30" />
        </Lines></DesignObjects></Scenario>"#,
    )
    .unwrap();

    assert_eq!(
        parse_design_lines(&document).unwrap(),
        vec![
            DesignLine {
                id: 7,
                points: vec![
                    [1.0, 2.0, 3.0],
                    [4.0, 5.0, 6.0],
                    [7.0, 8.0, 9.0],
                    [10.0, 11.0, 12.0],
                ],
            },
            DesignLine {
                id: 8,
                points: Vec::new(),
            },
            DesignLine {
                id: 4,
                points: vec![[1.0, 2.0, 3.0]],
            },
            DesignLine {
                id: 4,
                points: vec![[10.0, 20.0, 30.0]],
            },
        ]
    );
}

#[test]
fn missing_wrapper_gives_no_lines() {
    for xml in ["<Scenario />", "<Scenario><DesignObjects /></Scenario>"] {
        let document = Document::from_xml(xml).unwrap();
        assert_eq!(parse_design_lines(&document), Ok(Vec::new()), "{xml}");
    }
}

#[test]
fn malformed_ids_and_points_are_reported() {
    for (xml, expected) in [
        (
            r"<Scenario><DesignObjects><Lines><Lines /></Lines></DesignObjects></Scenario>",
            DesignLineError::MissingId,
        ),
        (
            r#"<Scenario><DesignObjects><Lines><Lines ID="bad" /></Lines></DesignObjects></Scenario>"#,
            DesignLineError::InvalidId {
                value: "bad".to_owned(),
            },
        ),
        (
            r#"<Scenario><DesignObjects><Lines><Lines ID="9" Position="1,nope,3" /></Lines></DesignObjects></Scenario>"#,
            DesignLineError::InvalidVector {
                line_id: 9,
                field: "Position",
                value: "1,nope,3".to_owned(),
            },
        ),
        (
            r#"<Scenario><DesignObjects><Lines><Lines ID="9"><Points>1,2,3|1,2,3,4</Points></Lines></Lines></DesignObjects></Scenario>"#,
            DesignLineError::InvalidVector {
                line_id: 9,
                field: "Points",
                value: "1,2,3,4".to_owned(),
            },
        ),
        (
            r#"<Scenario><DesignObjects><Lines><Lines ID="9" Position="1,inf,3" /></Lines></DesignObjects></Scenario>"#,
            DesignLineError::InvalidVector {
                line_id: 9,
                field: "Position",
                value: "1,inf,3".to_owned(),
            },
        ),
    ] {
        let document = Document::from_xml(xml).unwrap();
        assert_eq!(parse_design_lines(&document), Err(expected), "{xml}");
    }
}
