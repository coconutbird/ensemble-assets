//! Tests that load a synthetic HW1 database through `Database::load`.
//!
//! The expected values come from the engine load order (`BDatabase_setupDependentData`
//! @ `0x1401ee690`, `BDatabase_setupProtoObjects` @ `0x1401f27f0`, `BDatabase_setupProtoSquads`
//! @ `0x1401f2fc0`, `BDatabase_setupProtoTechs` @ `0x1401f4e80`, `BProtoObject_load`
//! @ `0x140341620` and `BGameMode_load` @ `0x140240e70`).

use std::collections::BTreeMap;

use database::hw1::{Database, Keyed};

/// An in-memory asset resolver that holds packed XMB files.
#[derive(Default)]
struct MemoryAssets(BTreeMap<String, Vec<u8>>);

impl MemoryAssets {
    /// Add the XML text `xml` as the packed file of the game path `path`.
    fn with_xml(mut self, path: &str, xml: &str) -> Self {
        let doc = xmb::Document::from_xml(xml)
            .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
        self.insert(path, &doc);
        self
    }

    /// Add `doc` as the packed file of the game path `path`.
    fn insert(&mut self, path: &str, doc: &xmb::Document) {
        let bytes = xmb::Writer::write_native(doc)
            .unwrap_or_else(|error| panic!("failed to write {path}: {error}"));
        self.0.insert(format!("{path}.xmb"), bytes);
    }
}

impl assets::AssetResolver for MemoryAssets {
    fn resolve(&mut self, path: &str) -> Option<Vec<u8>> {
        self.0.get(path).cloned()
    }

    fn exists(&self, path: &str) -> bool {
        self.0.contains_key(path)
    }
}

const OBJECTS: &str = r#"<Objects>
    <Object name="marine"><LOS>10</LOS></Object>
    <Object name="flag" update="true"><LOS>5</LOS></Object>
    <Object name="hog" update="false"><LOS>12</LOS></Object>
</Objects>"#;

const OBJECTS_UPDATE: &str = r#"<Objects>
    <Object name="flag" update="true"><LOS>14</LOS></Object>
    <Object name="icon" update="true"><LOS>1</LOS></Object>
</Objects>"#;

const SQUADS: &str = r#"<Squads>
    <Squad name="marines"/>
    <Squad name="monitor" update="true"/>
    <Squad name="hogs"/>
</Squads>"#;

const SQUADS_UPDATE: &str = r#"<Squads>
    <Squad name="monitor" update="true"/>
    <Squad name="marines"/>
</Squads>"#;

const TECHS: &str = r#"<TechTree>
    <Tech name="armor"/>
    <Tech name="speed"/>
</TechTree>"#;

const TECHS_UPDATE: &str = r#"<TechTree>
    <Tech name="speed"/>
    <Tech name="ScarabBuff"><Flag>Shadow</Flag></Tech>
</TechTree>"#;

fn overlay_assets() -> MemoryAssets {
    MemoryAssets::default()
        .with_xml("data\\objects.xml", OBJECTS)
        .with_xml("data\\objects_update.xml", OBJECTS_UPDATE)
        .with_xml("data\\squads.xml", SQUADS)
        .with_xml("data\\squads_update.xml", SQUADS_UPDATE)
        .with_xml("data\\techs.xml", TECHS)
        .with_xml("data\\techs_update.xml", TECHS_UPDATE)
}

fn names<T: Keyed>(records: &[T]) -> Vec<&str> {
    records.iter().map(Keyed::key).collect()
}

#[test]
fn load_replaces_flagged_records_and_appends_update_records() {
    let db = Database::load(&mut overlay_assets()).expect("load");

    // Objects and squads: skip `update="true"` base records, then append the update file.
    assert_eq!(names(&db.objects), ["marine", "hog", "flag", "icon"]);
    assert_eq!(db.objects[2].los, Some(14.0));
    // A duplicate squad name keeps its index.
    assert_eq!(names(&db.squads), ["marines", "hogs", "monitor", "marines"]);
    // Techs: append every update record, with no `update` filter.
    assert_eq!(names(&db.techs), ["armor", "speed", "speed", "ScarabBuff"]);
}

#[test]
fn load_without_update_files_still_skips_flagged_base_records() {
    let mut assets = MemoryAssets::default()
        .with_xml("data\\objects.xml", OBJECTS)
        .with_xml("data\\techs.xml", TECHS);

    let db = Database::load(&mut assets).expect("load");

    assert_eq!(names(&db.objects), ["marine", "hog"]);
    assert_eq!(names(&db.techs), ["armor", "speed"]);
}

#[test]
fn load_rejects_a_duplicate_object_name_without_case() {
    let mut assets = MemoryAssets::default()
        .with_xml("data\\objects.xml", OBJECTS)
        .with_xml(
            "data\\objects_update.xml",
            r#"<Objects><Object name="MARINE"/></Objects>"#,
        );

    let error = Database::load(&mut assets).expect_err("duplicate object name");

    assert!(
        matches!(
            &error,
            database::Error::DuplicateName { table: "object", name } if name == "MARINE"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn load_base_files_keeps_the_file_records() {
    let db = Database::load_base_files(&mut overlay_assets()).expect("load");

    assert_eq!(names(&db.objects), ["marine", "flag", "hog"]);
    assert_eq!(db.objects[1].los, Some(5.0));
    assert_eq!(names(&db.squads), ["marines", "monitor", "hogs"]);
    assert_eq!(names(&db.techs), ["armor", "speed"]);
}

#[test]
fn to_documents_reload_to_the_same_tables() {
    let db = Database::load(&mut overlay_assets()).expect("load");

    let mut assets = overlay_assets();
    for (path, doc) in db.to_documents().expect("serialize") {
        assets.insert(&path, &doc);
    }
    let reloaded = Database::load(&mut assets).expect("reload");

    assert_eq!(names(&reloaded.objects), names(&db.objects));
    assert_eq!(reloaded.objects[2].los, Some(14.0));
    assert!(
        reloaded
            .objects
            .iter()
            .all(|object| object.update.is_none())
    );
    assert_eq!(names(&reloaded.squads), names(&db.squads));
    assert_eq!(names(&reloaded.techs), names(&db.techs));
}

#[test]
fn object_type_index_keeps_only_declared_types() {
    let mut assets = MemoryAssets::default()
        .with_xml(
            "data\\objects.xml",
            r#"<Objects>
                <Object name="marine">
                    <ObjectType>Infantry</ObjectType>
                    <ObjectType>Icon</ObjectType>
                    <ObjectType>_icon</ObjectType>
                    <ObjectType>hog</ObjectType>
                    <ObjectType>INFANTRY</ObjectType>
                </Object>
                <Object name="hog"/>
            </Objects>"#,
        )
        .with_xml(
            "data\\objecttypes.xml",
            r"<ObjectTypes>
                <ObjectType>_Icon</ObjectType>
                <ObjectType>Hog</ObjectType>
                <ObjectType>Infantry</ObjectType>
                <ObjectType>infantry</ObjectType>
            </ObjectTypes>",
        );
    let db = Database::load(&mut assets).expect("load");
    let index = db.object_type_index();

    assert_eq!(
        names(&db.object_types),
        ["_Icon", "Hog", "Infantry", "infantry"]
    );
    // Objects come first in the name map; type `j` has the ID `objects.len() + j`.
    assert_eq!(index.id("HOG"), Some(1));
    assert_eq!(index.id("infantry"), Some(4));
    assert_eq!(index.declared_type("Infantry"), Some(2));
    // An object name and an undeclared name are not types.
    assert_eq!(index.declared_type("hog"), None);
    assert_eq!(index.declared_type("Icon"), None);
    // `_Icon` and `Infantry`, in index order and without duplicates.
    assert_eq!(index.types_of(&db.objects[0]), [0, 2]);
}

#[test]
fn game_modes_read_every_engine_field() {
    let mut assets = MemoryAssets::default().with_xml(
        "data\\gamemodes.xml",
        r#"<GameModes>
            <GameMode>
                <Name>Skirmish</Name>
                <WorldScript>SkirmishWorld.triggerscript</WorldScript>
                <PlayerScript>SkirmishPlayer.triggerscript</PlayerScript>
                <Tech/>
                <DisplayNameID>8000</DisplayNameID>
                <DescriptionID>26027</DescriptionID>
                <LongDescriptionID>26028</LongDescriptionID>
                <NPC/>
            </GameMode>
            <GameMode Locked="true" dlc="true">
                <Name>CapFlag</Name>
                <Tech>armor</Tech>
            </GameMode>
        </GameModes>"#,
    );

    let db = Database::load(&mut assets).expect("load");

    assert_eq!(names(&db.game_modes), ["Skirmish", "CapFlag"]);
    let skirmish = &db.game_modes[0];
    assert_eq!(
        skirmish.world_script.as_deref(),
        Some("SkirmishWorld.triggerscript")
    );
    assert_eq!(
        skirmish.player_script.as_deref(),
        Some("SkirmishPlayer.triggerscript")
    );
    assert_eq!(skirmish.tech_name(), None);
    assert_eq!(skirmish.display_name_id, Some(8000));
    assert_eq!(skirmish.description_id, Some(26027));
    assert_eq!(skirmish.long_description_id, Some(26028));
    assert_eq!(skirmish.locked, None);
    assert_eq!(skirmish.dlc, None);
    let cap_flag = &db.game_modes[1];
    assert_eq!(cap_flag.locked, Some(true));
    assert_eq!(cap_flag.dlc, Some(true));
    assert_eq!(cap_flag.tech_name(), Some("armor"));
    assert_eq!(cap_flag.world_script, None);
}

/// `BAI_loadDifficultySettings` @ `0x140150640` reads each `AIDifficultySetting`, its
/// `DefaultValue` children and the `DataPoint` children of its `PiecewiseFunc`.
#[test]
fn ai_difficulty_settings_read_defaults_and_data_points() {
    let mut assets = MemoryAssets::default().with_xml(
        "data\\aidifficultysettings.xml",
        r#"<AIDifficultySettings>
            <AIDifficultySetting Name="MaxMissionFocusTime" InputLabel="Difficulty" OutputLabel="Seconds">
                <DefaultValue Name="SPCEasy" Value="9999"/>
                <PiecewiseFunc>
                    <DataPoint Input="0" Output="60"/>
                    <DataPoint Input="0.67" Output="10"/>
                </PiecewiseFunc>
            </AIDifficultySetting>
            <AIDifficultySetting Name="EconMultiplier"/>
        </AIDifficultySettings>"#,
    );

    let db = Database::load(&mut assets).expect("load");

    assert_eq!(
        names(&db.ai_difficulty_settings),
        ["MaxMissionFocusTime", "EconMultiplier"]
    );
    let focus = &db.ai_difficulty_settings[0];
    assert_eq!(focus.input_label.as_deref(), Some("Difficulty"));
    assert_eq!(focus.output_label.as_deref(), Some("Seconds"));
    assert_eq!(focus.default_values.len(), 1);
    assert_eq!(focus.default_values[0].name, "SPCEasy");
    assert!((focus.default_values[0].value - 9999.0).abs() < 1.0e-3);
    let points = &focus.function.as_ref().expect("function").points;
    assert_eq!(points.len(), 2);
    assert!((points[1].input - 0.67).abs() < 1.0e-6);
    assert!((points[1].output - 10.0).abs() < 1.0e-6);
    assert!(db.ai_difficulty_settings[1].function.is_none());
}
