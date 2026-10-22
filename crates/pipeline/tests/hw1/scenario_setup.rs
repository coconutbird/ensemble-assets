//! Scenario setup fields of the shipped Blood Gulch map, as `BScenario_load` reads them.

use pipeline::hw1::scenario::{PlayerPlacement, ScenarioList, ScenarioRelation};
use test_utils::require_game_dir;

#[test]
#[ignore = "requires HW1_GAME_DIR"]
fn blood_gulch_setup_fields_match_the_engine_reads() {
    let dir = require_game_dir("HW1_GAME_DIR");
    let mut src = pipeline::hw1::loader::load_with_scenario(&dir, "blood_gulch.era");
    let list = ScenarioList::load(&mut src);
    let descriptor = list
        .scenarios
        .get("blood_gulch")
        .expect("blood_gulch descriptor");
    let scenario = descriptor
        .read_scenario(&mut src)
        .expect("blood_gulch .scn");

    let players = scenario.players();
    assert_eq!(
        players.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["Player1", "Player2", "Creeps"]
    );
    let first = &players[0];
    assert!(first.use_starting_units);
    assert!(first.use_player_settings);
    assert!(first.default_resources);
    assert!(first.controllable);
    let creeps = &players[2];
    assert!(!creeps.use_starting_units);
    assert!(!creeps.use_player_settings);
    assert!(!creeps.default_resources);
    assert!(!creeps.controllable);
    assert_eq!(creeps.localised_display_name_id(), Some(25966));
    assert_eq!((creeps.team, creeps.color), (3, 13));

    assert_eq!(scenario.sim_bounds(), [0.0, 0.0, 1024.0, 1024.0]);
    assert_eq!(scenario.sound_banks(), ["amb_shield_int.bnk"]);
    assert_eq!(scenario.player_placement(), PlayerPlacement::Grouped);
    let table = scenario.diplomacy_table().expect("blood_gulch diplomacy");
    assert_eq!(table[1][2], ScenarioRelation::Enemy);
    assert_eq!(table[3][3], ScenarioRelation::Ally);
    assert_eq!(table[4][1], ScenarioRelation::Neutral);

    let positions = scenario.positions();
    assert_eq!(positions.len(), 2);
    assert!(positions.iter().all(|position| position.player == -1
        && position.unit_start_objects() == [-1; 4]
        && position.rally_start_object == -1));

    let no_tie: Vec<i32> = scenario
        .objects()
        .iter()
        .filter(|object| object.no_tie_to_ground())
        .map(|object| object.id)
        .collect();
    assert_eq!(no_tie, [189, 188]);
    let platform = scenario
        .objects()
        .iter()
        .find(|object| object.id == 188)
        .expect("object 188");
    assert_eq!(platform.proto_name, "hook_bldg_sniperplatform_01");

    let sc2 = descriptor
        .read_sc2(&mut src)
        .expect("valid .sc2")
        .expect("blood_gulch .sc2");
    assert_eq!(sc2.objects().len(), 35);
    assert!(
        sc2.objects()
            .iter()
            .all(|object| object.proto_name.starts_with("env_arcadia_pinetree_"))
    );
    let sc3 = descriptor
        .read_sc3(&mut src)
        .expect("valid .sc3")
        .expect("blood_gulch .sc3");
    assert_eq!(sc3.objects().len(), 18);
    assert!(
        sc3.objects()
            .iter()
            .all(|object| object.proto_name.starts_with("snd_play_"))
    );
    assert!(
        scenario
            .objects()
            .iter()
            .all(|object| !object.proto_name.starts_with("env_arcadia_pinetree_"))
    );
}
