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

const HP_BARS: &str = r#"<HPBarDefinition>
    <ColorStages name="Other"/>
    <ColorStages name="DEFAULT">
        <Hitpoint>
            <Stage value="0.38f" intensity="1.0f">208 16 16</Stage>
            <Stage value="1.00f" intensity="2">146 228 65</Stage>
        </Hitpoint>
        <Shield>
            <Stage value="1.00f">42 238 236</Stage>
        </Shield>
    </ColorStages>
    <HPBar name="UNSC_Health">
        <Texture>ui\flash\HUD\unsc_unit1</Texture>
        <MaskTexture>ui\flash\HUD\unsc_unit1_mask</MaskTexture>
        <SizeX>8.0f</SizeX>
        <SizeY>2.5f</SizeY>
        <HitpointUV u1="0.0500" u2="0.9500" v1="0.2300" v2="0.4300"/>
        <ShieldUV u1="" u2="" v1="" v2=""/>
    </HPBar>
    <HPBar name="COV_Shield"><SizeX>10.0000</SizeX></HPBar>
    <VeterancyBar name="UNSC_Star">
        <Texture>star</Texture>
        <OffsetY>1.3f</OffsetY>
        <BackgroundSizeX>4.0f</BackgroundSizeX>
    </VeterancyBar>
    <PieProgress name="UNSC_Ability">
        <OffsetX>-4.3f</OffsetX>
        <Fill>true</Fill>
        <Clockwise>false</Clockwise>
    </PieProgress>
    <BobbleHead name="ctf_flag">
        <VetAnchor>
            <Anchor offsetX="-0.7500" offsetY="0.2500"/>
            <Anchor offsetX="1.5" offsetY="5"/>
        </VetAnchor>
        <PortraitOffsetX>0</PortraitOffsetX>
    </BobbleHead>
    <BuildingStrength name="UNSC_Base">
        <Padding>1.6f</Padding>
        <Anchors><Anchor offsetX="2" offsetY="3"/></Anchors>
    </BuildingStrength>
</HPBarDefinition>"#;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1.0e-6,
        "expected {expected}, got {actual}"
    );
}

/// `BDatabase_setupHPBars` @ `0x1401ece90` reads the definitions, the `0.38f` literals and the
/// empty UV attributes; an absent tag keeps the default of the reset function.
#[test]
fn hp_bars_read_literals_empty_uvs_and_defaults() {
    let mut assets = MemoryAssets::default().with_xml("data\\hpbars.xml", HP_BARS);

    let db = Database::load(&mut assets).expect("load");
    let definition = db.hp_bars.as_ref().expect("hp bars");

    let stages = definition.default_color_stages().expect("DEFAULT");
    assert_eq!(stages.name, "DEFAULT");
    let hitpoint = &stages.hitpoint.stages;
    assert_eq!(hitpoint.len(), 2);
    assert_close(hitpoint[0].value, 0.38);
    assert_close(hitpoint[0].intensity, 1.0);
    assert_eq!(
        (
            hitpoint[0].color.red,
            hitpoint[0].color.green,
            hitpoint[0].color.blue
        ),
        (208, 16, 16)
    );
    assert_close(hitpoint[1].intensity, 2.0);
    assert!(stages.ammo.stages.is_empty());
    assert_close(stages.shield.stages[0].intensity, 0.0);

    assert_eq!(definition.hp_bar_id("unsc_health"), Some(0));
    assert_eq!(definition.hp_bar_id("COV_Shield"), Some(1));
    assert_eq!(definition.hp_bar_id("missing"), None);
    let bar = &definition.hp_bars[0];
    assert_eq!(bar.texture, "ui\\flash\\HUD\\unsc_unit1");
    assert_eq!(bar.mask_texture, "ui\\flash\\HUD\\unsc_unit1_mask");
    assert_close(bar.size_x, 8.0);
    assert_close(bar.size_y, 2.5);
    let rect = bar
        .hitpoint_uv
        .as_ref()
        .and_then(database::hw1::hpbars::HpBarUv::rect)
        .expect("hitpoint rect");
    for (actual, expected) in rect.into_iter().zip([0.05, 0.23, 0.95, 0.43]) {
        assert_close(actual, expected);
    }
    let shield = bar.shield_uv.expect("ShieldUV element");
    assert_eq!(shield.u1, None);
    assert_eq!(shield.rect(), None);
    assert!(bar.ammo_uv.is_none());
    let cov = &definition.hp_bars[1];
    assert_close(cov.size_x, 10.0);
    assert_close(cov.size_y, 0.0);

    let star = &definition.veterancy_bars[0];
    assert_eq!(star.texture, "star");
    assert_close(star.size_x, 0.0);
    assert_close(star.offset_y, 1.3);
    assert_eq!(star.background_size_x, Some(4.0));

    let pie = &definition.pie_progress[0];
    assert_close(pie.size_x, 1.0);
    assert_close(pie.offset_x, -4.3);
    assert_close(pie.background_size_y, 1.0);
    assert!(pie.fill);
    assert!(!pie.clockwise);

    let bobble = &definition.bobble_heads[0];
    assert_eq!(bobble.vet_anchors.anchors.len(), 2);
    assert_close(bobble.vet_anchors.anchors[0].offset_x, -0.75);
    assert_close(bobble.vet_anchors.anchors[1].offset_y, 5.0);
    assert_close(bobble.vet_angle_range, 90.0);
    assert_close(bobble.vet_radius, 1.0);
    assert_close(bobble.portrait_offset_x, 0.0);

    let strength = &definition.building_strengths[0];
    assert_close(strength.padding, 1.6);
    assert_close(strength.size_x, 1.0);
    assert_eq!(strength.anchors.anchors.len(), 1);
}

const PLAYER_COLORS: &str = r#"<playerColors>
    <skirmish>
        <color num="0" objects="175 175 175" corpse="88 88 88" selection="175 175 175" minimap="175 175 175" ui="175 175 175"/>
        <color num="1" objects="145 163 85" corpse="35 55 12" selection="69 110 24" minimap="69 110 24" ui="69 110 24"/>
        <color num="1" objects="1 2 3"/>
        <color num="16" objects="9 9 9"/>
        <friendOrFoeEnemy objects="190 0 0" corpse="95 0 0" selection="190 0 0" minimap="190 0 0" ui="190 0 0"/>
    </skirmish>
    <spc>
        <color num="2" objects="0 30 200" corpse="0 15 100" selection="0 30 200" minimap="0 30 200" ui="0 30 200"/>
    </spc>
    <civ>Covenant
        <player num="0" color="0"/>
        <player num="7" color="13"/>
    </civ>
</playerColors>"#;

/// `BDatabase_setupPlayerColors` @ `0x1401f9560` writes each `<color num>` to its entry; the
/// last one wins, a `num` above 15 is ignored and an absent attribute is white.
#[test]
fn player_colors_keep_the_last_entry_and_default_to_white() {
    let mut assets = MemoryAssets::default().with_xml("data\\playercolors.xml", PLAYER_COLORS);

    let db = Database::load(&mut assets).expect("load");
    let colors = db.player_colors.as_ref().expect("player colors");
    let skirmish = colors.skirmish.as_ref().expect("skirmish");

    assert_eq!(skirmish.colors.len(), 4);
    let gaia = skirmish.color(0).expect("color 0");
    assert_eq!(gaia.corpse.red, 88);
    let player = skirmish.color(1).expect("color 1");
    assert_eq!(
        (
            player.objects.red,
            player.objects.green,
            player.objects.blue
        ),
        (1, 2, 3)
    );
    assert_eq!(
        (player.ui.red, player.ui.green, player.ui.blue),
        (255, 255, 255)
    );
    assert!(skirmish.color(16).is_none());
    assert!(skirmish.color(2).is_none());
    assert_eq!(
        skirmish.friend_or_foe_enemy.expect("enemy").selection.red,
        190
    );
    assert!(skirmish.friend_or_foe_self.is_none());

    let spc = colors.spc.as_ref().expect("spc");
    assert_eq!(spc.color(2).expect("color 2").objects.blue, 200);

    assert_eq!(colors.civs.len(), 1);
    assert_eq!(colors.civs[0].name, "Covenant");
    assert_eq!(colors.civs[0].players[1].num, Some(7));
    assert_eq!(colors.civs[0].players[1].color, Some(13));
}

/// `Database::to_documents` writes both files so that a reload gives the same values.
#[test]
fn hp_bars_and_player_colors_reload_from_to_documents() {
    let mut assets = MemoryAssets::default()
        .with_xml("data\\hpbars.xml", HP_BARS)
        .with_xml("data\\playercolors.xml", PLAYER_COLORS);
    let db = Database::load(&mut assets).expect("load");

    let mut written = MemoryAssets::default();
    for (path, doc) in db.to_documents().expect("serialize") {
        written.insert(&path, &doc);
    }
    let reloaded = Database::load(&mut written).expect("reload");

    let before = db.hp_bars.as_ref().expect("hp bars");
    let after = reloaded.hp_bars.as_ref().expect("reloaded hp bars");
    assert_eq!(after.hp_bars.len(), before.hp_bars.len());
    assert_eq!(after.hp_bars[0].hitpoint_uv, before.hp_bars[0].hitpoint_uv);
    assert_eq!(after.hp_bars[0].shield_uv, before.hp_bars[0].shield_uv);
    assert_eq!(
        after
            .default_color_stages()
            .expect("DEFAULT")
            .hitpoint
            .stages,
        before
            .default_color_stages()
            .expect("DEFAULT")
            .hitpoint
            .stages
    );
    assert_eq!(after.pie_progress[0].fill, before.pie_progress[0].fill);
    assert_eq!(
        after.bobble_heads[0].vet_anchors.anchors,
        before.bobble_heads[0].vet_anchors.anchors
    );

    let before = db.player_colors.as_ref().expect("player colors");
    let after = reloaded
        .player_colors
        .as_ref()
        .expect("reloaded player colors");
    let skirmish = |colors: &database::hw1::PlayerColors| {
        colors.skirmish.as_ref().expect("skirmish").colors.clone()
    };
    assert_eq!(skirmish(after), skirmish(before));
    assert_eq!(after.civs[0].players, before.civs[0].players);
    assert_eq!(after.civs[0].name, "Covenant");
}
