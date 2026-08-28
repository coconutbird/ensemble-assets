use std::collections::BTreeSet;

use serde::de::DeserializeOwned;

fn assert_table_schema<T>(label: &str, xml: &str, root_name: &str, child_name: &str)
where
    T: DeserializeOwned,
{
    let document = xmb::Document::from_xml(xml)
        .unwrap_or_else(|error| panic!("failed to read {label}: {error}"));
    let root = document.root().expect("fixture has no root node");
    assert_eq!(root.name, root_name, "unexpected root in {label}");

    let mut warnings = BTreeSet::new();
    let mut count = 0;
    for node in root.children.iter().filter(|node| node.name == child_name) {
        count += 1;
        let (_, node_warnings) = bdt_serde::from_node_warned::<T>(node)
            .unwrap_or_else(|error| panic!("failed to parse <{child_name}> in {label}: {error}"));
        warnings.extend(node_warnings.into_iter().map(|warning| warning.to_string()));
    }

    assert!(count > 0, "no <{child_name}> records in {label}");
    assert!(
        warnings.is_empty(),
        "unmapped schema in {label}:\n{}",
        warnings.into_iter().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn shipped_database_xml_has_no_unmapped_fields() {
    assert_table_schema::<database::hw1::ProtoObject>(
        "objects.xml",
        include_str!("../../../save/root.era/data/objects.xml"),
        "Objects",
        "Object",
    );
    assert_table_schema::<database::hw1::Squad>(
        "squads.xml",
        include_str!("../../../save/root.era/data/squads.xml"),
        "Squads",
        "Squad",
    );
    assert_table_schema::<database::hw1::Tech>(
        "techs.xml",
        include_str!("../../../save/root.era/data/techs.xml"),
        "TechTree",
        "Tech",
    );
    assert_table_schema::<database::hw1::Ability>(
        "abilities.xml",
        include_str!("../../../save/root.era/data/abilities.xml"),
        "Abilities",
        "Ability",
    );
    assert_table_schema::<database::hw1::Power>(
        "powers.xml",
        include_str!("../../../save/root.era/data/powers.xml"),
        "Powers",
        "Power",
    );
    assert_table_schema::<database::hw1::Civ>(
        "civs.xml",
        include_str!("../../../save/root.era/data/civs.xml"),
        "Civs",
        "Civ",
    );
    assert_table_schema::<database::hw1::Leader>(
        "leaders.xml",
        include_str!("../../../save/root.era/data/leaders.xml"),
        "Leaders",
        "Leader",
    );
    assert_table_schema::<database::hw1::WeaponType>(
        "weapontypes.xml",
        include_str!("../../../save/root_update.era/data/weapontypes.xml"),
        "WeaponTypes",
        "WeaponType",
    );
    assert_table_schema::<database::hw1::DamageType>(
        "damagetypes.xml",
        include_str!("../../../save/root.era/data/damagetypes.xml"),
        "DamageTypes",
        "DamageType",
    );
}

#[test]
fn shipped_game_data_has_no_unmapped_fields() {
    let document = xmb::Document::from_xml(include_str!(
        "../../../save/root_update.era/data/gamedata.xml"
    ))
    .expect("failed to read gamedata.xml");
    let root = document.root().expect("fixture has no root node");
    let (_, warnings) = bdt_serde::from_node_warned::<database::hw1::GameData>(root)
        .expect("failed to parse gamedata.xml");
    let warnings = warnings
        .into_iter()
        .map(|warning| warning.to_string())
        .collect::<BTreeSet<_>>();
    assert!(
        warnings.is_empty(),
        "unmapped schema in gamedata.xml:\n{}",
        warnings.into_iter().collect::<Vec<_>>().join("\n")
    );
}
