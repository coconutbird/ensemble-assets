use super::*;

fn parse(xml: &str) -> Result<DesignObjects, DesignObjectError> {
    parse_design_objects(&Document::from_xml(xml).unwrap())
}

#[test]
fn decodes_position_then_points_in_authored_axes() {
    let objects = parse(
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
        objects.lines,
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
    assert_eq!(objects.spheres, Vec::new());
}

/// `BDesignSphere_load` reads `ID`, `Position` and `Radius`, then each `Data` child.
/// `BDesignObjectData_load` keeps the `Type` text as written.
#[test]
fn decodes_spheres_with_their_data_in_document_order() {
    let objects = parse(
        r#"<Scenario><DesignObjects>
            <Spheres>
                <Sphere ID="24" Name="DesignSphere24" Position="447.258,0,268.722" Radius="55">
                    <Data>
                        <Type>Creeps</Type>
                        <Attract>1.5</Attract>
                        <Repel>2</Repel>
                        <ChokePoint>3</ChokePoint>
                        <Value>4</Value>
                        <Value1>5</Value1>
                        <Value2/>
                        <Value12>6</Value12>
                    </Data>
                </Sphere>
                <Other ID="25" />
            </Spheres>
            <spheres><Sphere ID="26" Radius="7"><Data><Type>Spawn</Type></Data><data><Type>Late</Type></data></Sphere></spheres>
        </DesignObjects></Scenario>"#,
    )
    .unwrap();

    let mut values = [0.0; 12];
    values[0] = 5.0;
    values[11] = 6.0;
    assert_eq!(
        objects.spheres,
        vec![
            DesignSphere {
                id: 24,
                position: [447.258, 0.0, 268.722],
                radius: 55.0,
                data: DesignData {
                    type_name: "Creeps".to_owned(),
                    attract: 1.5,
                    repel: 2.0,
                    choke_point: 3.0,
                    value: 4.0,
                    values,
                },
            },
            DesignSphere {
                id: 25,
                position: [0.0; 3],
                radius: 0.0,
                data: DesignData::default(),
            },
            DesignSphere {
                id: 26,
                position: [0.0; 3],
                radius: 7.0,
                data: DesignData {
                    type_name: "Late".to_owned(),
                    ..DesignData::default()
                },
            },
        ]
    );
}

#[test]
fn missing_wrapper_gives_no_objects() {
    for xml in ["<Scenario />", "<Scenario><DesignObjects /></Scenario>"] {
        assert_eq!(parse(xml), Ok(DesignObjects::default()), "{xml}");
    }
}

#[test]
fn malformed_ids_vectors_and_numbers_are_reported() {
    for (xml, expected) in [
        (
            r"<Scenario><DesignObjects><Lines><Lines /></Lines></DesignObjects></Scenario>",
            DesignObjectError::MissingId { kind: "line" },
        ),
        (
            r"<Scenario><DesignObjects><Spheres><Sphere /></Spheres></DesignObjects></Scenario>",
            DesignObjectError::MissingId { kind: "sphere" },
        ),
        (
            r#"<Scenario><DesignObjects><Lines><Lines ID="bad" /></Lines></DesignObjects></Scenario>"#,
            DesignObjectError::InvalidId {
                kind: "line",
                value: "bad".to_owned(),
            },
        ),
        (
            r#"<Scenario><DesignObjects><Lines><Lines ID="9" Position="1,nope,3" /></Lines></DesignObjects></Scenario>"#,
            DesignObjectError::InvalidVector {
                object_id: 9,
                field: "Position",
                value: "1,nope,3".to_owned(),
            },
        ),
        (
            r#"<Scenario><DesignObjects><Lines><Lines ID="9"><Points>1,2,3|1,2,3,4</Points></Lines></Lines></DesignObjects></Scenario>"#,
            DesignObjectError::InvalidVector {
                object_id: 9,
                field: "Points",
                value: "1,2,3,4".to_owned(),
            },
        ),
        (
            r#"<Scenario><DesignObjects><Spheres><Sphere ID="9" Position="1,inf,3" /></Spheres></DesignObjects></Scenario>"#,
            DesignObjectError::InvalidVector {
                object_id: 9,
                field: "Position",
                value: "1,inf,3".to_owned(),
            },
        ),
        (
            r#"<Scenario><DesignObjects><Spheres><Sphere ID="9" Radius="big" /></Spheres></DesignObjects></Scenario>"#,
            DesignObjectError::InvalidNumber {
                object_id: 9,
                field: "Radius".to_owned(),
                value: "big".to_owned(),
            },
        ),
        (
            r#"<Scenario><DesignObjects><Spheres><Sphere ID="9"><Data><Repel>x</Repel></Data></Sphere></Spheres></DesignObjects></Scenario>"#,
            DesignObjectError::InvalidNumber {
                object_id: 9,
                field: "Repel".to_owned(),
                value: "x".to_owned(),
            },
        ),
    ] {
        assert_eq!(parse(xml), Err(expected), "{xml}");
    }
}
