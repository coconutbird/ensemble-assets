//! Schema tests that parse synthetic HW1 database XML and check typed round trips.

#[path = "support/schema.rs"]
mod schema;

use std::collections::BTreeSet;

use serde::de::DeserializeOwned;

use schema::assert_xml_round_trip;

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

#[test]
fn ida_object_handler_fields_round_trip() {
    let (_, object) = assert_xml_round_trip::<database::hw1::ProtoObject>(
        "synthetic proto object",
        r#"<Object name="SyntheticObject">
            <ActiveScanChance radiusScale="1.5">0.25</ActiveScanChance>
            <AddResource Amount="12.5">Supplies</AddResource>
            <AutoParkingLot Rotation="90" Offset="1, 2, 3">ParkingObject</AutoParkingLot>
            <BuildOffset>4,5,6</BuildOffset>
            <Cost ResourceType="Supplies">100</Cost>
            <Cost ResourceType="Power">25</Cost>
            <CaptureCost Civ="unsc" ResourceType="Supplies">50</CaptureCost>
            <ChildObjects>
                <Object Type="Object">ChildObject</Object>
                <Object Type="ParkingLot" AttachBone="root" Offset="7,8,9" Rotation="45" UserCiv="unsc">ChildA</Object>
                <Object Type="Foundation">ChildB</Object>
            </ChildObjects>
            <Command Type="Train" Position="2" AutoClose="true">TargetSquad</Command>
            <DeathSpawnSquad CheckPos="true" MaxPopCount="7">DeathSquad</DeathSpawnSquad>
            <ExistSound bone="engine" />
            <GaiaRolloverTextID civ="gaia">1234</GaiaRolloverTextID>
            <GatherLink Radius="8.5" Target="Resource" Self="true">GatherObject</GatherLink>
            <GroundIK ikRange="2.5" linkCount="3" x="0.25" z="0.75">foot</GroundIK>
            <GroundIKTilt factor="0.5">root</GroundIKTilt>
            <HitZone Hitpoints="50" Shieldpoints="25" Active="true" HasShields="true">body</HitZone>
            <HoveringRumble LeftRumbleType="Low" RightRumbleType="High" Duration="1.25" LeftStrength="0.4" RightStrength="0.8" />
            <HPBar sizeX="2" sizeY="3" offset="1,0,2">Vehicle</HPBar>
            <ImpactDecal sizeX="4" sizeZ="5" timeFullyOpaque="1" fadeOutTime="2" orientation="forward">decal_material</ImpactDecal>
            <MinimapColor red="0.1" green="0.2" blue="0.3" />
            <MinimapIcon size="1.75">icon_name</MinimapIcon>
            <PerturbInitialVelocity minTime="0.1" maxTime="0.9">3.5</PerturbInitialVelocity>
            <Pop type="Infantry">2</Pop>
            <PopCapAddition type="Infantry">4</PopCapAddition>
            <Rate rate="Gather">1.25</Rate>
            <SingleBoneIK>arm_l</SingleBoneIK>
            <SingleBoneIK>arm_r</SingleBoneIK>
            <Socket player="Self" AutoSocket="true">SocketObject</Socket>
            <Sound Type="Create" Squad="SquadA" Action="Spawn">cue_create</Sound>
            <SquadModeAnim Mode="Lockdown">mode_anim</SquadModeAnim>
            <SweetSpotIK linkCount="2">spine</SweetSpotIK>
            <TrainerType ApplyFormation="true">3</TrainerType>
            <TrainLimit Type="Unit" Count="6" Bucket="1">UnitA</TrainLimit>
            <TrainLimit Type="Squad" Count="4" Bucket="2">SquadA</TrainLimit>
        </Object>"#,
        "Object",
    );

    assert_eq!(object.name, "SyntheticObject");
    assert_eq!(object.costs.len(), 2);
    assert_eq!(object.capture_costs.len(), 1);
    assert_eq!(object.commands.len(), 1);
    assert_eq!(object.single_bone_ik, ["arm_l", "arm_r"]);
    assert_eq!(object.train_limits.len(), 2);
    assert_eq!(object.build_offset.as_ref().map(|value| value.z), Some(6.0));
    assert_eq!(
        object
            .child_objects
            .as_ref()
            .map(|children| children.objects.len()),
        Some(3)
    );
    assert_eq!(
        object
            .child_objects
            .as_ref()
            .and_then(|children| children.objects.first())
            .and_then(|child| child.child_type),
        Some(database::hw1::objects::ChildObjectType::Object)
    );
    assert_eq!(
        object
            .death_spawn_squad
            .as_ref()
            .and_then(|spawn| spawn.check_position),
        Some(true)
    );
    assert_eq!(
        object
            .impact_decal
            .as_ref()
            .and_then(|decal| decal.orientation),
        Some(database::hw1::objects::ImpactDecalOrientation::Forward)
    );
}

#[test]
fn ida_ability_and_power_handler_fields_round_trip() {
    let (_, ability) = assert_xml_round_trip::<database::hw1::Ability>(
        "synthetic ability",
        r#"<Ability Name="SyntheticAbility">
            <Icon>ui\ability_icon</Icon>
            <Object>UnitA</Object>
            <Object>UnitB</Object>
            <AmmoCost>2.5</AmmoCost>
            <AttackSquadMode>true</AttackSquadMode>
            <Sprinting />
            <DodgeModifier>0.25</DodgeModifier>
        </Ability>"#,
        "Ability",
    );
    assert_eq!(ability.icon.as_deref(), Some("ui\\ability_icon"));
    assert_eq!(ability.objects, ["UnitA", "UnitB"]);
    assert_eq!(ability.sprinting, Some(true));

    let (_, power) = assert_xml_round_trip::<database::hw1::Power>(
        "synthetic power",
        r#"<Power name="SyntheticPower">
            <TriggerScript>trigger.lua</TriggerScript>
            <CommandTriggerScript>command.lua</CommandTriggerScript>
            <Attributes>
                <PowerType>Transport</PowerType>
                <AutoRecharge>5000</AutoRecharge>
                <InfiniteUses />
                <LeaderPower />
                <IconLocation>1</IconLocation>
                <IconLocation>4</IconLocation>
                <Cost Supplies="100" Power="20" CampaignFoo="3" Collectable="2" />
                <DynamicCost ObjectType="UnitA">1.5</DynamicCost>
                <TargetEffectiveness ObjectType="UnitB">4</TargetEffectiveness>
                <MultiRechargePower />
                <NotDisruptable />
                <SequentialRecharge />
                <ShowInPowerMenu />
                <ShowTargetHighlight ObjectType="UnitA" Relation="Enemy">true</ShowTargetHighlight>
                <TechPrereq>TechA</TechPrereq>
                <TechPrereq>TechB</TechPrereq>
                <UnitPower />
                <Pop type="Infantry">3</Pop>
                <ChildObjects><Object>ChildA</Object><Object>ChildB</Object></ChildObjects>
                <CameraEnableAutoZoom />
                <CameraEnableAutoZoomInstant />
                <BaseDataLevel><Data type="float" name="Radius">5.5</Data></BaseDataLevel>
                <DataLevel level="1"><Data type="int" name="Count">2</Data></DataLevel>
            </Attributes>
        </Power>"#,
        "Power",
    );
    let attributes = power.attributes.expect("power attributes");
    assert_eq!(attributes.icon_locations, [1, 4]);
    assert_eq!(attributes.auto_recharge, Some(5000));
    assert_eq!(attributes.tech_prerequisites, ["TechA", "TechB"]);
    assert_eq!(attributes.multi_recharge_power, Some(true));
    assert_eq!(attributes.dynamic_costs.len(), 1);
    assert_eq!(attributes.target_effectiveness.len(), 1);
    assert_eq!(attributes.data_levels[0].level, Some(1));
    assert_eq!(
        attributes.cost.as_ref().and_then(|cost| cost.campaign_foo),
        Some(3.0)
    );
    assert_eq!(
        attributes.cost.as_ref().and_then(|cost| cost.collectable),
        Some(2.0)
    );
    assert_eq!(
        attributes
            .child_objects
            .as_ref()
            .map(|children| children.objects.len()),
        Some(2)
    );
}

#[test]
fn ida_tech_effect_handler_fields_round_trip() {
    let (_, tech) = assert_xml_round_trip::<database::hw1::Tech>(
        "synthetic tech",
        r#"<Tech name="SyntheticTech" type="Normal">
            <OrPrereqs>
                <TechStatus tech="TechA" status="Active" />
                <TypeCount unit="UnitA" operator="GreaterEqual" count="2" />
            </OrPrereqs>
            <Effects>
                <Effect type="Data" amount="1.5" subtype="DamageModifier" relativity="Percent"
                    action="Attack" allactions="true" commandType="ProtoUnit" CommandData="UnitB"
                    Rate="Gather" WeaponType="Cannon" DamageType="Heavy" StringID="1234"
                    FromType="UnitA" ToType="UnitB" Hardpoint="turret" hpbar="Vehicle"
                    iconName="icon_name" iconType="building" impactEffect="impact_fx"
                    popType="Infantry" power="PowerA" Resource="Supplies" squadName="SquadA"
                    SquadType="SquadB" unitType="UnitC" Ability="AbilityA">
                    <Target type="ProtoUnit">UnitA</Target>
                </Effect>
                <Effect type="TransformUnit">UnitD</Effect>
                <Effect type="AttachSquad" squadType="SquadC" />
            </Effects>
        </Tech>"#,
        "Tech",
    );
    let effects = &tech.effects.expect("tech effects").entries;
    assert_eq!(effects.len(), 3);
    let data = &effects[0];
    assert_eq!(data.rate.as_deref(), Some("Gather"));
    assert_eq!(data.weapon_type.as_deref(), Some("Cannon"));
    assert_eq!(data.damage_type.as_deref(), Some("Heavy"));
    assert_eq!(data.string_id, Some(1234));
    assert_eq!(data.squad_type.as_deref(), Some("SquadB"));
    assert_eq!(data.allactions, Some(true));
    assert_eq!(effects[2].squad_type.as_deref(), Some("SquadC"));
    assert_eq!(
        tech.or_prereqs
            .as_ref()
            .map(|prereqs| prereqs.type_counts.len()),
        Some(1)
    );
}

#[test]
fn ida_leader_and_squad_handler_fields_round_trip() {
    let (_, leader) = assert_xml_round_trip::<database::hw1::Leader>(
        "synthetic leader",
        r#"<Leader Name="SyntheticLeader">
            <Civ>unsc</Civ>
            <Power>PrimaryPower</Power>
            <StartingUnit Offset="1,2,3" BuildOther="UnitB" DoppleOnStart="true">UnitA</StartingUnit>
            <StartingUnit Offset="4,5,6">UnitC</StartingUnit>
            <RallyPointOffset>7,8,9</RallyPointOffset>
            <SupportPower IconLocation="2" TechPrereq="TechA">
                <Power>SupportA</Power><Power>SupportB</Power>
            </SupportPower>
        </Leader>"#,
        "Leader",
    );
    assert_eq!(leader.power.as_deref(), Some("PrimaryPower"));
    assert_eq!(leader.starting_units.len(), 2);
    assert_eq!(
        leader.starting_units[0]
            .offset
            .as_ref()
            .map(|offset| offset.y),
        Some(2.0)
    );
    assert_eq!(leader.support_powers[0].powers, ["SupportA", "SupportB"]);

    let (_, squad) = assert_xml_round_trip::<database::hw1::Squad>(
        "synthetic squad",
        r#"<Squad name="SyntheticSquad">
            <HPBar sizeX="2" sizeY="3" offset="1,2,3">Vehicle</HPBar>
            <Birth spawnPoint="start" endPoint="end" anim0="a0" anim1="a1" anim2="a2" anim3="a3" trainerAnim="trainer">FlyIn</Birth>
            <AbilityRecoveryBar Centered="Centered" sizeX="4" sizeY="5" offsetX="6" offsetY="7">Recovery</AbilityRecoveryBar>
            <VeterancyBar Centered="Centered" sizeX="8" sizeY="9" offsetX="10" offsetY="11">Veterancy</VeterancyBar>
            <Selection>
                <RadiusX>12</RadiusX><RadiusZ>13</RadiusZ><YOffset>14</YOffset><ZOffset>15</ZOffset>
                <ConformToTerrain>true</ConformToTerrain><AllowOrientation>true</AllowOrientation>
            </Selection>
            <Sound Type="Select" Squad="SquadA" World="WorldA" CastingUnitOnly="true">cue_select</Sound>
            <Level>3</Level><TechLevel>4</TechLevel>
        </Squad>"#,
        "Squad",
    );
    assert_eq!(squad.hp_bar.as_ref().and_then(|bar| bar.size_x), Some(2.0));
    assert_eq!(
        squad
            .birth
            .as_ref()
            .and_then(|birth| birth.animation_3.as_deref()),
        Some("a3")
    );
    assert_eq!(
        squad
            .selection
            .as_ref()
            .and_then(|selection| selection.allow_orientation),
        Some(true)
    );
    assert_eq!(squad.sounds.len(), 1);
    assert_eq!(squad.level, Some(3));
    assert_eq!(squad.tech_level, Some(4));
}

#[test]
fn ida_game_data_scalar_types_round_trip() {
    let (_, game_data) = assert_xml_round_trip::<database::hw1::GameData>(
        "synthetic game data",
        r#"<GameData>
            <DifficultyNormali>1.25</DifficultyNormali>
            <GoodAgainstMinAttackGrade>7</GoodAgainstMinAttackGrade>
            <MaxSquadPathsPerFrame>8</MaxSquadPathsPerFrame>
            <MaxPlatoonPathsPerFrame>9</MaxPlatoonPathsPerFrame>
            <ObjectiveArrowMaxIndex>255</ObjectiveArrowMaxIndex>
            <TransportMax>10</TransportMax>
            <TransportMaxSearchLocations>11</TransportMaxSearchLocations>
            <TransportBlockTime>15</TransportBlockTime>
            <TransportLoadBlockTime>5</TransportLoadBlockTime>
            <TransportMaxBlockAttempts>12</TransportMaxBlockAttempts>
            <ShieldBarColor>10, 20, 30</ShieldBarColor>
            <AmmoBarColor>40 50 60</AmmoBarColor>
            <GoodAgainstReticle>
                <NoEffect>1</NoEffect><Weak>2</Weak><Fair>3</Fair><Good>4</Good><Extreme>5</Extreme>
            </GoodAgainstReticle>
            <Dot small="0.75" medium="1.25" />
            <InfectionMap>
                <InfectionMapEntry base="UnitA" infected="InfectedUnitA" infectedSquad="InfectedSquadA" />
                <InfectionMapEntry base="UnitB" infected="InfectedUnitB" infectedSquad="InfectedSquadB" />
            </InfectionMap>
        </GameData>"#,
        "GameData",
    );
    assert_eq!(game_data.difficulty_normal, Some(1.25));
    assert_eq!(game_data.good_against_min_attack_rating, Some(7));
    assert_eq!(game_data.objective_arrow_max_index, Some(255));
    assert_eq!(game_data.transport_block_time, Some(15));
    assert_eq!(game_data.transport_load_block_time, Some(5));
    assert_eq!(game_data.transport_max_block_attempts, Some(12));
    assert_eq!(
        game_data
            .good_against_reticle
            .as_ref()
            .and_then(|reticle| reticle.extreme),
        Some(5)
    );
    assert_eq!(
        game_data.dot.as_ref().and_then(|dot| dot.medium),
        Some(1.25)
    );
    assert_eq!(
        game_data
            .infection_map
            .as_ref()
            .map(|infection_map| infection_map.entries.len()),
        Some(2)
    );
    assert_eq!(
        game_data
            .infection_map
            .as_ref()
            .and_then(|infection_map| infection_map.entries.first())
            .map(|entry| entry.infected_squad.as_str()),
        Some("InfectedSquadA")
    );
    assert_eq!(
        game_data.shield_bar_color,
        Some(database::hw1::RgbColor {
            red: 10,
            green: 20,
            blue: 30,
        })
    );
}
