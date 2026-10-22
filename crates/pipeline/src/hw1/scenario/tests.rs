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

/// `BScenario_loadObject` reads `VisualVariationIndex` with
/// `BXMLAttribute_getValueAsInt` (`strtol` / `atoi`), so only empty text is absent.
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
            (14, Some(0)),
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

#[test]
fn sc2_and_sc3_paths_follow_the_scenario_path_without_scn() {
    let descriptor = ScenarioDescriptor {
        file: "skirmish/design/blood_gulch/blood_gulch.scn".to_owned(),
        ..ScenarioDescriptor::default()
    };
    assert_eq!(
        descriptor.sc2_path().as_deref(),
        Some("scenario\\skirmish\\design\\blood_gulch\\blood_gulch.sc2")
    );
    assert_eq!(
        descriptor.sc3_path().as_deref(),
        Some("scenario\\skirmish\\design\\blood_gulch\\blood_gulch.sc3")
    );
    assert_eq!(ScenarioDescriptor::default().sc2_path(), None);
}

/// Defaults of `BScenario_loadPlayer` @ `0x1403808a0`: flag bytes `0x10100`
/// at +0x44 and team, colour and leader `-1`.
#[test]
fn player_without_attributes_has_the_engine_defaults() {
    let scenario = ScenarioData::from_xml_str("<Scenario><Players><Player /></Players></Scenario>")
        .expect("valid scenario XML");
    let player = &scenario.players()[0];
    assert!(!player.use_starting_units);
    assert!(player.controllable);
    assert!(player.default_resources);
    assert!(!player.use_player_settings);
    assert_eq!((player.team, player.color), (-1, -1));
    assert_eq!(player.leader, "");
    assert_eq!(player.resource("Supplies").to_bits(), 0.0_f32.to_bits());
    assert_eq!(player.localised_display_name_id(), None);
    assert_eq!(player.pops().len(), 0);
}

#[test]
fn player_attributes_follow_the_engine_readers() {
    let scenario = ScenarioData::from_xml_str(
        r#"<Scenario><Players>
            <Player Name="Creeps" LocalisedDisplayName="25966,Rebels" UseStartingUnits="TRUE"
                    Controllable="0" UsePlayerSettings="yes" DefaultResources="false"
                    Civ="Covenant" Leader="Arbiter" Color="13" Team="3"
                    Supplies="800" Power="12.5" CampaignFoo="2" Collectable="1" Favor="9">
                <Pops><Pop type="Unit" max="99">30</Pop><Pop type="" max="500" /></Pops>
                <ForbidObjects><Object>unsc_bldg_reactor_01</Object></ForbidObjects>
                <ForbidSquads><Squad>unsc_inf_marine_01</Squad></ForbidSquads>
                <ForbidTechs><Tech>unsc_odst_upgrade0</Tech></ForbidTechs>
            </Player>
        </Players></Scenario>"#,
    )
    .expect("valid scenario XML");
    let player = &scenario.players()[0];
    assert_eq!(player.localised_display_name_id(), Some(25966));
    assert!(player.use_starting_units);
    assert!(!player.controllable);
    // `yes` is not an engine boolean: the reader keeps the default.
    assert!(!player.use_player_settings);
    assert!(!player.default_resources);
    assert_eq!(player.leader, "Arbiter");
    assert_eq!((player.team, player.color), (3, 13));
    assert_eq!(player.resource("supplies").to_bits(), 800.0_f32.to_bits());
    assert_eq!(player.resource("Power").to_bits(), 12.5_f32.to_bits());
    assert_eq!(player.resource("CampaignFoo").to_bits(), 2.0_f32.to_bits());
    assert_eq!(player.resource("Collectable").to_bits(), 1.0_f32.to_bits());
    assert_eq!(player.resource("Favor").to_bits(), 0.0_f32.to_bits());
    let pops: Vec<_> = player
        .pops()
        .iter()
        .map(|pop| (pop.pop_type.as_str(), pop.count, pop.max))
        .collect();
    assert_eq!(pops, vec![("Unit", 30.0, 99.0), ("", 0.0, 500.0)]);
    assert_eq!(player.forbidden_objects(), ["unsc_bldg_reactor_01"]);
    assert_eq!(player.forbidden_squads(), ["unsc_inf_marine_01"]);
    assert_eq!(player.forbidden_techs(), ["unsc_odst_upgrade0"]);

    let leader1 = ScenarioData::from_xml_str(
        r#"<Scenario><Players><Player Leader1="Cutter" /></Players></Scenario>"#,
    )
    .expect("valid scenario XML");
    assert_eq!(leader1.players()[0].leader, "Cutter");
}

/// Defaults of `BScenario_loadPosition` @ `0x14037f430`.
#[test]
fn position_fields_follow_the_engine_defaults() {
    let scenario = ScenarioData::from_xml_str(
        r#"<Scenario><Positions>
            <Position Position="1,2,3" />
            <Position Player="2" Number="1" Position="4,5,6" Forward="" DefaultCamera="false"
                      UnitStartObject1="118" UnitStartObject3="14" RallyStartObject="1145" />
            <Position Position="bad" Forward="0.6755,0,-0.7374" />
        </Positions></Scenario>"#,
    )
    .expect("valid scenario XML");
    let [absent, explicit, invalid] = scenario.positions() else {
        panic!("three positions");
    };
    assert_eq!((absent.player, absent.number), (-1, -1));
    assert!(absent.default_camera);
    assert_eq!(absent.unit_start_objects(), [-1; 4]);
    assert_eq!(absent.rally_start_object, -1);
    assert_eq!(absent.position_vec3(), Some([1.0, 2.0, 3.0]));
    assert_eq!(absent.forward_vec3(), [0.0, 0.0, 1.0]);

    assert_eq!((explicit.player, explicit.number), (2, 1));
    assert!(!explicit.default_camera);
    assert_eq!(explicit.unit_start_objects(), [118, -1, 14, -1]);
    assert_eq!(explicit.rally_start_object, 1145);
    // A present but empty `Forward` clears the vector.
    assert_eq!(explicit.forward_vec3(), [0.0; 3]);

    assert_eq!(invalid.position_vec3(), None);
    assert_eq!(invalid.forward_vec3(), [0.6755, 0.0, -0.7374]);
}

#[test]
fn player_placement_reads_spacing_only_for_consecutive() {
    for (xml, expected) in [
        ("<Scenario />", PlayerPlacement::Fixed),
        (
            r#"<Scenario><PlayerPlacement Type="Fixed" Spacing="0" /></Scenario>"#,
            PlayerPlacement::Fixed,
        ),
        (
            r#"<Scenario><PlayerPlacement Type="grouped" Spacing="3" /></Scenario>"#,
            PlayerPlacement::Grouped,
        ),
        (
            r#"<Scenario><PlayerPlacement Type="Consecutive" Spacing="3" /></Scenario>"#,
            PlayerPlacement::Consecutive { spacing: 3 },
        ),
        (
            r#"<Scenario><PlayerPlacement Type="Consecutive" /></Scenario>"#,
            PlayerPlacement::Consecutive { spacing: 0 },
        ),
        (
            r#"<Scenario><PlayerPlacement Type="Random" /></Scenario>"#,
            PlayerPlacement::Random,
        ),
    ] {
        let scenario = ScenarioData::from_xml_str(xml).expect("valid scenario XML");
        assert_eq!(scenario.player_placement(), expected, "{xml}");
    }
}

/// `BScenario_loadDiplomacy` @ `0x14037f840` fills `table[5 * team + id]`.
#[test]
fn diplomacy_table_follows_the_engine_loader() {
    assert_eq!(ScenarioData::default().diplomacy_table(), None);
    let empty = ScenarioData::from_xml_str("<Scenario><Diplomacies /></Scenario>")
        .expect("valid scenario XML");
    assert_eq!(empty.diplomacy_table(), None);

    let scenario = ScenarioData::from_xml_str(
        r#"<Scenario><Diplomacies>
            <Diplomacy Team="1">
                <Team ID="1">Ally</Team><Team ID="2">enemy</Team><Team ID="3">Neutral</Team>
                <Team ID="4">Self</Team><Team ID="5">Enemy</Team><Team ID="0">Other</Team>
            </Diplomacy>
            <Diplomacy Team="7"><Team ID="1">Enemy</Team></Diplomacy>
            <Diplomacy Team="1"><Team ID="1">Enemy</Team></Diplomacy>
        </Diplomacies></Scenario>"#,
    )
    .expect("valid scenario XML");
    let table = scenario.diplomacy_table().expect("diplomacy rows");
    assert_eq!(
        table[1],
        [
            ScenarioRelation::Any,
            ScenarioRelation::Enemy,
            ScenarioRelation::Enemy,
            ScenarioRelation::Neutral,
            ScenarioRelation::SelfTeam,
        ]
    );
    assert_eq!(table[2], [ScenarioRelation::Any; DIPLOMACY_TEAM_COUNT]);
    assert_eq!(ScenarioRelation::Neutral.engine_value(), 4);
}

/// `BScenario_load` defaults: `SimBoundsMin*` `0`, `SimBoundsMax*` `FLT_MAX`.
#[test]
fn top_level_sim_bounds_and_sound_banks_are_read() {
    assert_eq!(
        ScenarioData::default().sim_bounds(),
        [0.0, 0.0, f32::MAX, f32::MAX]
    );
    let scenario = ScenarioData::from_xml_str(
        "<Scenario>
            <SoundBank>amb_one.bnk</SoundBank>
            <SimBoundsMinX>10</SimBoundsMinX><SimBoundsMinZ /><SimBoundsMaxX>1024</SimBoundsMaxX>
            <SoundBank>amb_two.bnk</SoundBank>
        </Scenario>",
    )
    .expect("valid scenario XML");
    assert_eq!(scenario.sim_bounds(), [10.0, 0.0, 1024.0, f32::MAX]);
    assert_eq!(scenario.sound_banks(), ["amb_one.bnk", "amb_two.bnk"]);
}

/// `BScenario_loadObject` @ `0x14037d6a0` defaults and `<Flag>` names.
#[test]
fn object_fields_follow_the_engine_loader() {
    let scenario = ScenarioData::from_xml_str(
        r#"<Scenario><Objects>
            <Object ID="188" IsSquad="false" Position="1,2,3" Forward="0,0,-2" Right="3,0,4">hook_bldg_sniperplatform_01<Flag>NoTieToGround</Flag><Flag>includeinsimrep</Flag><Flag>OrientUnitWithGround</Flag></Object>
            <Object ID="2" IsSquad="maybe" Position="1,2" Forward="bad">plain</Object>
        </Objects></Scenario>"#,
    )
    .expect("valid scenario XML");
    let [flagged, plain] = scenario.objects() else {
        panic!("two objects");
    };
    assert_eq!(flagged.proto_name, "hook_bldg_sniperplatform_01");
    assert_eq!(flagged.is_squad, Some(false));
    assert!(flagged.no_tie_to_ground());
    assert!(flagged.include_in_sim_rep());
    assert_eq!(flagged.forward_vec3(), [0.0, 0.0, -1.0]);
    assert_eq!(flagged.right_vec3(), [0.6, 0.0, 0.8]);
    assert_eq!(flagged.tint_value.to_bits(), 1.0_f32.to_bits());

    assert_eq!(plain.is_squad, None);
    assert!(!plain.no_tie_to_ground());
    assert_eq!(plain.position_vec3(), None);
    assert_eq!(plain.forward_vec3(), [0.0, 0.0, 1.0]);
    assert_eq!(plain.right_vec3(), [1.0, 0.0, 0.0]);
}

#[test]
fn load_steps_keep_document_order() {
    let document = xmb::Document::from_xml(
        r#"<Scenario>
            <ExternalTriggerScript FileName="first.triggerscript" />
            <players />
            <TriggerSystem Name="Inline" />
            <ExternalTriggerScript FileName="" />
            <Objects />
            <externaltriggerscript filename="second.triggerscript" />
        </Scenario>"#,
    )
    .expect("valid scenario XML");
    let root = document.root().expect("scenario root");
    let steps: Vec<String> = scenario_load_steps(root)
        .into_iter()
        .map(|step| match step {
            ScenarioLoadStep::Players => "players".to_owned(),
            ScenarioLoadStep::Objects => "objects".to_owned(),
            ScenarioLoadStep::TriggerSystem(node) => format!("embedded {}", node.name),
            ScenarioLoadStep::ExternalTriggerScript(file_name) => format!("external {file_name}"),
        })
        .collect();
    assert_eq!(
        steps,
        [
            "external first.triggerscript",
            "players",
            "embedded TriggerSystem",
            "objects",
            "external second.triggerscript",
        ]
    );
    let scenario = parse_scenario_data(root);
    // The data model keeps the element with the empty `FileName`.
    assert_eq!(scenario.external_trigger_scripts().len(), 3);
}

#[test]
fn serialization_round_trips_the_setup_fields() {
    let scenario = ScenarioData::from_xml_str(
        r#"<Scenario>
            <Diplomacies><Diplomacy Team="1"><Team ID="2">Enemy</Team></Diplomacy></Diplomacies>
            <PlayerPlacement Type="Consecutive" Spacing="2" />
            <Positions><Position Position="1,0,2" UnitStartObject2="7" /></Positions>
            <Players><Player Name="P" Controllable="false" Leader="Cutter" Supplies="5">
                <Pops><Pop type="Unit" max="40">30</Pop></Pops>
                <ForbidTechs><Tech>tech_a</Tech></ForbidTechs>
            </Player></Players>
            <Objects><Object ID="1" Position="0,0,0">thing<Flag>NoTieToGround</Flag></Object></Objects>
            <ExternalTriggerScript FileName="extra.triggerscript" />
            <SimBoundsMaxZ>512</SimBoundsMaxZ>
            <SoundBank>amb.bnk</SoundBank>
        </Scenario>"#,
    )
    .expect("valid scenario XML");
    let document = scenario.to_document().expect("serializable scenario");
    let reparsed = parse_scenario_data(document.root().expect("scenario root"));

    assert_eq!(reparsed.diplomacy_table(), scenario.diplomacy_table());
    assert_eq!(
        reparsed.player_placement(),
        PlayerPlacement::Consecutive { spacing: 2 }
    );
    assert_eq!(
        reparsed.positions()[0].unit_start_objects(),
        [-1, 7, -1, -1]
    );
    assert_eq!(reparsed.positions()[0].forward_vec3(), [0.0, 0.0, 1.0]);
    let player = &reparsed.players()[0];
    assert!(!player.controllable);
    assert_eq!(player.leader, "Cutter");
    assert_eq!(player.resource("Supplies").to_bits(), 5.0_f32.to_bits());
    assert_eq!(player.power, None);
    assert_eq!(player.pops()[0].count.to_bits(), 30.0_f32.to_bits());
    assert_eq!(player.forbidden_techs(), ["tech_a"]);
    assert_eq!(reparsed.objects()[0].proto_name, "thing");
    assert!(reparsed.objects()[0].no_tie_to_ground());
    assert_eq!(
        reparsed.external_trigger_scripts()[0].file_name,
        "extra.triggerscript"
    );
    assert_eq!(reparsed.sim_bounds(), [0.0, 0.0, f32::MAX, 512.0]);
    assert_eq!(reparsed.sound_banks(), ["amb.bnk"]);
}

#[test]
fn object_file_reads_only_objects() {
    let document = xmb::Document::from_xml(
        r#"<Scenario><Objects>
            <Object ID="10000008" Player="0" Position="1,2,3">env_arcadia_pinetree_08</Object>
        </Objects><Players><Player Name="ignored" /></Players></Scenario>"#,
    )
    .expect("valid object file XML");
    let file =
        ScenarioObjectFile::from_node(document.root().expect("root")).expect("valid object file");
    assert_eq!(file.objects().len(), 1);
    assert_eq!(file.objects()[0].proto_name, "env_arcadia_pinetree_08");
    let round_trip = ScenarioObjectFile::from_node(
        file.to_document()
            .expect("serializable objects")
            .root()
            .expect("root"),
    )
    .expect("valid object file");
    assert_eq!(round_trip.objects()[0].id, 10_000_008);
}
