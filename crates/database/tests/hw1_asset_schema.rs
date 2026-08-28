use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Serialize, de::DeserializeOwned};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("database crate is not inside the workspace")
        .join("save")
}

fn collect_files_with_suffix(directory: &Path, suffix: &str, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
    {
        let entry = entry.unwrap_or_else(|error| {
            panic!(
                "failed to read an entry below {}: {error}",
                directory.display()
            )
        });
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_suffix(&path, suffix, files);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(suffix))
        {
            files.push(path);
        }
    }
}

fn assert_asset_family_schema<T>(
    suffix: &str,
    root_name: &str,
    mut inspect: impl FnMut(&T),
) -> usize
where
    T: DeserializeOwned + Serialize,
{
    let fixture_root = fixture_root();
    let mut files = Vec::new();
    collect_files_with_suffix(&fixture_root, suffix, &mut files);
    files.sort();
    assert!(!files.is_empty(), "no *{suffix} fixtures found");

    let mut problems = BTreeMap::new();
    for path in &files {
        let xml = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let document = xmb::Document::from_xml(&xml)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()));
        let root = document
            .root()
            .unwrap_or_else(|| panic!("{} has no root node", path.display()));
        assert_eq!(
            root.name,
            root_name,
            "unexpected root in {}",
            path.display()
        );

        let (value, warnings) = bdt_serde::from_node_warned::<T>(root)
            .unwrap_or_else(|error| panic!("failed to deserialize {}: {error}", path.display()));
        for warning in warnings {
            problems
                .entry(warning.to_string())
                .or_insert_with(|| path.display().to_string());
        }
        inspect(&value);

        let serialized = bdt_serde::to_node(root_name, &value)
            .unwrap_or_else(|error| panic!("failed to serialize {}: {error}", path.display()));
        let (_, warnings) = bdt_serde::from_node_warned::<T>(&serialized).unwrap_or_else(|error| {
            panic!(
                "failed to deserialize round trip for {}: {error}",
                path.display()
            )
        });
        for warning in warnings {
            problems
                .entry(format!("round trip: {warning}"))
                .or_insert_with(|| path.display().to_string());
        }
    }

    assert!(
        problems.is_empty(),
        "unmapped *{suffix} schema:\n{}",
        problems
            .into_iter()
            .map(|(warning, path)| format!("{warning} (first seen in {path})"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    files.len()
}

fn assert_synthetic_schema<T>(xml: &str, root_name: &str) -> T
where
    T: DeserializeOwned + Serialize,
{
    let document = xmb::Document::from_xml(xml).expect("synthetic XML should parse");
    let root = document.root().expect("synthetic XML should have a root");
    assert_eq!(root.name, root_name);

    let (value, warnings) = bdt_serde::from_node_warned::<T>(root)
        .expect("synthetic XML should deserialize into its schema");
    assert!(
        warnings.is_empty(),
        "unmapped synthetic schema:\n{}",
        warnings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );

    let serialized = bdt_serde::to_node(root_name, &value)
        .expect("synthetic schema should serialize back to a node");
    let (_, warnings) = bdt_serde::from_node_warned::<T>(&serialized)
        .expect("serialized synthetic schema should deserialize");
    assert!(
        warnings.is_empty(),
        "unmapped synthetic round trip:\n{}",
        warnings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    value
}

#[test]
#[allow(clippy::too_many_lines)]
fn engine_supported_tactics_fields_are_typed() {
    let tactics = assert_synthetic_schema::<database::hw1::TacticData>(
        r#"<TacticData>
<Weapon>
  <Name>AllWeaponFields</Name><DamagePerSecond>12.5</DamagePerSecond>
  <DPSRamp startDPSPercent="25" finalDPSPercent="100" rampTime="3" shieldDrain="true"/>
  <DOTrate>2</DOTrate><DOTduration>4</DOTduration>
  <DOTEffect small="SmallFX" medium="MediumFX" large="LargeFX">DefaultFX</DOTEffect>
  <Reapply>1</Reapply><Apply>0.5</Apply>
  <PostAttackCooldownMin>1</PostAttackCooldownMin><PostAttackCooldownMax>2</PostAttackCooldownMax>
  <PreAttackCooldownMin>3</PreAttackCooldownMin><PreAttackCooldownMax>4</PreAttackCooldownMax>
  <AttackRate>0.75</AttackRate><Projectile>Projectile</Projectile>
  <ImpactEffect size="Large" doShockwaveAction="true">Impact</ImpactEffect>
  <ImpactCameraShake Duration="0.5" Strength="2"/>
  <ImpactRumble LeftRumbleType="Constant" RightRumbleType="Pulse" Duration="1" LeftStrength="0.4" RightStrength="0.6"/>
  <ImpactCameraEffect><Name>Inline</Name>
    <ColorTransformFactor><Value time="0">1</Value></ColorTransformFactor>
    <BlurFactor><Value time="0">0.5</Value></BlurFactor>
    <ColorTransformR><Value time="0">1,0,0</Value></ColorTransformR>
    <ColorTransformG><Value time="0">0,1,0</Value></ColorTransformG>
    <ColorTransformB><Value time="0">0,0,1</Value></ColorTransformB>
    <BlurXY><Value time="0">0,0</Value></BlurXY>
    <RadialBlurFactor use3DPos="true" useHoverPointAs3DPos="false"><Value time="0">1,2</Value></RadialBlurFactor>
    <FOV><Value time="0">45</Value></FOV><Zoom><Value time="0">2</Value></Zoom>
    <Yaw><Value time="0">3</Value></Yaw><Pitch><Value time="0">4</Value></Pitch>
  </ImpactCameraEffect>
  <WeaponType>Plasma</WeaponType><VisualAmmo>8</VisualAmmo>
  <AOERadius>5</AOERadius><AOEPrimaryTargetFactor>1</AOEPrimaryTargetFactor>
  <AOEDistanceFactor>0.5</AOEDistanceFactor><AOEDamageFactor>0.25</AOEDamageFactor><AOELinearDamage/>
  <PhysicsLaunchAngleMin>10</PhysicsLaunchAngleMin><PhysicsLaunchAngleMax>20</PhysicsLaunchAngleMax>
  <PhysicsLaunchAxial/><PhysicsForceMin>1</PhysicsForceMin><PhysicsForceMax>2</PhysicsForceMax>
  <PhysicsForceMaxAngle>30</PhysicsForceMaxAngle><ThrowUnits/><ThrowAliveUnits/><ThrowDamageParts/>
  <FlailThrownUnits/><Dodgeable/><Deflectable/><SmallArmsDeflectable/><OverridesRevive/><PullUnits/>
  <UseDPSasDPA/><UseGroupRange/><CarriedObjectAsProjectileVisual/>
  <Hardpoint>Gun</Hardpoint><AlternateHardpoint>Gun2</AlternateHardpoint><TriggerScript>OnFire</TriggerScript>
  <AllowFriendlyFire/><EnableHeightBonusDamage/><MinRange>2</MinRange><MaxRange>40</MaxRange><UsesAmmo/>
  <Beam activationSpeed="1" trackingSpeed="2" turnRate="3" waitTime="4" searchRadius="5"/>
  <MaxDamagePerRam>6</MaxDamagePerRam><ReflectDamageFactor>0.5</ReflectDamageFactor>
  <Accuracy>0.8</Accuracy><MovingAccuracy>0.7</MovingAccuracy><MaxDeviation>1</MaxDeviation>
  <MovingMaxDeviation>2</MovingMaxDeviation><AccuracyDistanceFactor>0.3</AccuracyDistanceFactor>
  <AccuracyDeviationFactor>0.4</AccuracyDeviationFactor><MaxVelocityLead>5</MaxVelocityLead>
  <AirBurstSpan>6</AirBurstSpan>
  <DamageRatingOverride type="Vehicle" halfKillCutoffFactor="0.5">1.5</DamageRatingOverride>
  <TargetPriority type="Infantry">2</TargetPriority><TargetsFootOfUnit/><KeepDPSRamp/>
  <CausePhysicsExplosion particle="Explosion" victimType="Vehicle"/>
  <Stasis SmartTargeting="true"/><StasisDrain/><StasisBomb/><StasisHealToDrainRatio>0.5</StasisHealToDrainRatio>
  <ThrowOffsetAngle>45</ThrowOffsetAngle><ThrowVelocity>10</ThrowVelocity><ApplyKnockback/><Tentacle/>
  <Daze TargetType="Infantry" MovementModifier="0.5" AOEDaze="true" SmartTargeting="true">3</Daze>
  <AOEIgnoresYAxis/><Bounces>2</Bounces><BounceRange>8</BounceRange>
  <CameraRubleShakeScalarNotLocal>0.75</CameraRubleShakeScalarNotLocal><AirBurst/><MaxPullRange>12</MaxPullRange>
</Weapon>
<State><Name>Combat</Name><IdleAnim>Idle</IdleAnim><WalkAnim>Walk</WalkAnim><JogAnim>Jog</JogAnim>
  <RunAnim>Run</RunAnim><DeathAnim>Death</DeathAnim><Action>AllActionFields</Action></State>
<Action>
  <Name>AllActionFields</Name><ActionType>RangedAttack</ActionType><ProjectileSpread>0.2</ProjectileSpread>
  <StopAttackingWhenAmmoDepleted/><DontLoopAttackAnim/><DontAutoRestart/><DontCheckOrientTolerance/>
  <KillSelfOnAttack/><MeleeRange/><InstantAttack/><Infection/><CanOrientOwner>false</CanOrientOwner>
  <MainAttack/><Stationary/><SquadType>Squad</SquadType><Weapon>AllWeaponFields</Weapon>
  <LinkedAction>Other</LinkedAction><Shockwave duration="2"/><SquadMode>Normal</SquadMode>
  <NewSquadMode>Alt</NewSquadMode><NewTacticState>Combat</NewTacticState>
  <Anim noInterrupt="true">Attack</Anim><AnimLowDetail actionCountThreshold="8">AttackLow</AnimLowDetail>
  <SupportAnim>Support</SupportAnim><ReloadAnim>Reload</ReloadAnim>
  <WorkRate>1</WorkRate><WorkRateVariance>0.1</WorkRateVariance><WorkRange>4</WorkRange>
  <DodgeChanceMax>0.9</DodgeChanceMax><DodgeChanceMin>0.2</DodgeChanceMin>
  <DodgeMaxAngle>90</DodgeMaxAngle><DodgeCooldown>2</DodgeCooldown><DodgePhysicsImpulse>3</DodgePhysicsImpulse>
  <WaitForDeflectCooldown/><MultiDeflect/><SmallArms/><DoShakeOnAttackTag/><HideSpawnUntilRelease/><AvoidOnly/>
  <DeflectChanceMax>0.8</DeflectChanceMax><DeflectChanceMin>0.1</DeflectChanceMin>
  <DeflectMaxAngle>80</DeflectMaxAngle><DeflectCooldown>1</DeflectCooldown>
  <DeflectMaxDmg>20</DeflectMaxDmg><DeflectTimeout>4</DeflectTimeout><WaitForDodgeCooldown/>
  <Strafing maxDistance="10" turnRate="90" trackingSpeed="2" jitter="0.1"/>
  <HoverAltitudeOffset>2</HoverAltitudeOffset><MaxTgtDepressionAngle>30</MaxTgtDepressionAngle>
  <MaxPitch>20</MaxPitch><MaxRoll>15</MaxRoll>
  <DamageModifiers damage="1.2" damageTaken="0.8" byCombatValue="true"/>
  <JoinType revertDamagePct="0.5" veterancyOverride="true" boardTime="2" boardAnim="Board" unjoinMaxDist="5" levels="3">Board</JoinType>
  <ParticleEffect>JoinFX</ParticleEffect><MergeType>Air</MergeType>
  <StartAnim noInterrupt="true">Start</StartAnim><EndAnim noInterrupt="true" SoundCue="Done">End</EndAnim>
  <MinIdleDuration>1</MinIdleDuration><ClearTacticState/><Beam collideWithUnits="true" collideWithTerrain="true"/>
  <TargetAir/><Resource>Supplies</Resource><Default>true</Default><NoAutoTarget/><DieOnBuilt/><TeamShare/>
  <ExposedAction>2</ExposedAction><TrampleAttack/><SlaveAttackAction>Slave</SlaveAttackAction>
  <BaseDPSWeapon>AllWeaponFields</BaseDPSWeapon><DisableAttackWaitTimer/><FindBetterAction/>
  <PersistBetweenOpps/><AllowReinforce/><Overrun/><PersistentActionType>RangedAttack</PersistentActionType>
  <Fatality attacker="FatalityAttack" target="FatalityTarget" cooldown="5" offsetTolerance="1"
    orientationOffset="30" toBoneRelative="true" transitionBeforeAnimating="true" air="true"
    airHeight="4" airApexPercent="0.7" transitionDelay="0.2" transitionTime="0.5"
    noMove="false" lastUnitOnly="true" boarding="true" attackerEnd="Idle" targetEnd="Dead">TargetProto</Fatality>
  <StartDisabled/><RequiresLockdown/><Duration DurationSpread="0.5">3</Duration>
  <DetonateThrow HorizontalMax="4" VerticalMax="5"/>
  <DetonateFromPhysics PhysicsDetonationThreshold="6"/><DetonateWhenInRange/>
  <AutoRepair AutoRepairIdleTime="1000" AutoRepairThreshold="0.5" AutoRepairSearchDistance="20"/>
  <InvalidTarget>BadTarget1</InvalidTarget><InvalidTarget>BadTarget2</InvalidTarget>
  <HealTarget/><TargetOfTarget/><UseTeleporter/><VelocityScalar>1.5</VelocityScalar>
  <DetonateOnDeath/><PickupObject/><PickupObjectOnKill/>
  <AttackRunDelayMin>1</AttackRunDelayMin><AttackRunDelayMax>2</AttackRunDelayMax>
  <AlertWhenComplete/><AutoJoin/><ReviveDelay>3</ReviveDelay><HibernateReviveDelay>4</HibernateReviveDelay>
  <ReviveRate>5</ReviveRate><ProtoObject Squad="true" bone="Root">SpawnSquad</ProtoObject>
  <Count>2</Count><MaxNumUnitsPerformAction>3</MaxNumUnitsPerformAction><DamageCharge>4</DamageCharge>
  <ChargeOnTaken/><ChargeOnDealt/><Chargable/>
</Action>
<Tactic><TargetRule><Relation>Enemy</Relation><AutoTargetSquadMode>Normal</AutoTargetSquadMode>
  <TargetsGround/><ContainsUnits/><DamageType>Normal</DamageType><DamageType>Shielded</DamageType>
  <TargetType>Infantry</TargetType><TargetType>Vehicle</TargetType><Action>AllActionFields</Action>
  <TargetState>Damaged</TargetState><OptionalAbility>Ability</OptionalAbility><GaiaOwned/><MergeSquads/><MeleeAttacker/>
  </TargetRule><PersistentAction>AllActionFields</PersistentAction>
  <PersistentSquadAction>AllActionFields</PersistentSquadAction></Tactic>
</TacticData>"#,
        "TacticData",
    );

    assert_eq!(tactics.weapons.len(), 1);
    assert_eq!(tactics.weapons[0].visual_ammo, Some(8));
    assert_eq!(tactics.weapons[0].bounces, Some(2));
    assert_eq!(tactics.actions.len(), 1);
    assert_eq!(
        tactics.actions[0]
            .auto_repair
            .as_ref()
            .and_then(|v| v.idle_time),
        Some(1_000)
    );
    assert_eq!(tactics.states[0].actions, ["AllActionFields"]);
    assert_eq!(tactics.tactic.expect("tactic rules").target_rules.len(), 1);
}

#[test]
fn engine_supported_physics_fields_are_typed() {
    let physics = assert_synthetic_schema::<database::hw1::Physics>(
        r#"<physics>
  <blueprint>body_a</blueprint><blueprint>body_b</blueprint>
  <ThrownByProjectiles>true</ThrownByProjectiles>
  <DestroyedByProjectiles>false</DestroyedByProjectiles>
  <TerrainEffects>effects\terrain</TerrainEffects><PhysicsOnDeath>true</PhysicsOnDeath>
  <Vehicle>warthog<info>
    <suspension springLength="1" wheelRadius="2" springStrength="3" dampingCompression="4"
      dampingRelaxation="5" normalClippingAngle="6"/>
    <chassisInertia yaw="7" pitch="8" roll="9"/>
    <hardpoints><hardpoint>1,2,3</hardpoint><hardpoint>4 5 6</hardpoint></hardpoints>
    <treadpoints height="10" length="11" velocityScalar="-0.1" wheelAttachments="true">
      <treadpoint zOffset="0.5">7,8,9</treadpoint>
    </treadpoints>
  </info></Vehicle>
  <clamshell><upper heightOffset="1">upper_bp</upper><lower heightOffset="2">lower_bp</lower>
    <pelvis>pelvis_bp</pelvis></clamshell>
  <centerOffset>1,2,3</centerOffset><ImpactMinVelocity>4.5</ImpactMinVelocity>
  <Phantom/><MotionType>fixed</MotionType>
</physics>"#,
        "physics",
    );

    assert_eq!(physics.blueprints, ["body_a", "body_b"]);
    assert!((physics.center_offset.expect("center offset").z - 3.0).abs() < f32::EPSILON);
    assert_eq!(
        physics
            .vehicle
            .expect("vehicle")
            .info
            .expect("vehicle info")
            .hardpoints
            .expect("hardpoints")
            .points
            .len(),
        2
    );

    let blueprint = assert_synthetic_schema::<database::hw1::Blueprint>(
        r"<blueprint><mass>10</mass><friction>0.5</friction><restitution>0.25</restitution>
  <centerOfMassOffset>1,2,3</centerOfMassOffset><angularDamping>0.1</angularDamping>
  <linearDamping>0.2</linearDamping><shape>shape_ref</shape><halfExtents>4,5,6</halfExtents>
</blueprint>",
        "blueprint",
    );
    assert!((blueprint.center_of_mass_offset.expect("COM offset").y - 2.0).abs() < f32::EPSILON);
    assert!((blueprint.half_extents.expect("half extents").x - 4.0).abs() < f32::EPSILON);
}

#[test]
fn engine_supported_visual_fields_are_typed() {
    let visual = assert_synthetic_schema::<database::hw1::Visual>(
        r#"<visual defaultmodel="Default">
  <model name="Default"><component>
    <asset type="Model"><file>model</file><damagefile>model_damage</damagefile><weight>2</weight>
      <uvOffset channel="1" uOfs="0.25" vOfs="0.5"/>
      <tag type="Rumble" position="0.5" name="Cue" tobone="Root" lifespan="1" force="2" force2="3"
        checkvisible="true" checkFOW="true" checkSelected="false" end="0.9" start="0.1"
        lockToGround="true" attach="true" userData="u0Offset 1" leftRumbleType="Pulse"
        rightRumbleType="Constant" loop="true" disregardorient="true"/>
      <progression><Stages><Stage><Value>1</Value><Alpha>0</Alpha><ValueVariance>0.1</ValueVariance></Stage>
        <Stage><Value>0</Value><Alpha>1</Alpha><ValueVariance>0</ValueVariance></Stage></Stages>
        <Loop>true</Loop><Cycles>2</Cycles></progression>
    </asset>
    <attach type="ModelRef" name="Turret" frombone="Root" tobone="Mount" syncanims="true" disregardorient="true"/>
    <point pointType="Launch" bone="Muzzle" pointdata="Metal"/>
    <logic type="Tech"><logicdata value="Upgrade" modelref="Alt" weight="2">
      <asset type="Model"><file>upgraded</file></asset>
      <logic type="Variation"><logicdata value="1"><asset type="Model"><file>variant</file></asset></logicdata></logic>
    </logicdata></logic>
  </component>
  <anim type="Attack" exitAction="Transition" tweenTime="3" tweenToAnimation="Idle">
    <asset type="Anim"><file>attack</file><tag type="Attack" position="0.5" tobone="Muzzle"/></asset>
    <attach type="ParticleFile" name="muzzle_fx" tobone="Muzzle"/><meshEnable>OptionalMesh</meshEnable>
  </anim></model>
  <logic type="Variation"><logicdata value="0" modelref="Default" weight="1"/></logic>
</visual>"#,
        "visual",
    );

    assert!(visual.logic.is_some());
    let model = &visual.models[0];
    let component = model.component.as_ref().expect("component");
    assert_eq!(component.assets[0].uv_offsets[0].channel, Some(1));
    assert_eq!(
        component.assets[0]
            .progression
            .as_ref()
            .and_then(|progression| progression.stages.as_ref())
            .map(|stages| stages.stages.len()),
        Some(2)
    );
    assert_eq!(model.anims[0].enabled_meshes, ["OptionalMesh"]);
}

#[test]
fn shipped_visual_files_have_no_unmapped_fields() {
    let mut models = 0;
    let count =
        assert_asset_family_schema::<database::hw1::Visual>(".vis.xml", "visual", |visual| {
            models += visual.models.len();
        });
    assert!(count > 2_000, "unexpectedly few visual fixtures: {count}");
    assert!(
        models > count,
        "visual fixtures did not produce typed models"
    );
}

#[test]
fn shipped_tactics_files_have_no_unmapped_fields() {
    let mut weapons = 0;
    let mut actions = 0;
    let count = assert_asset_family_schema::<database::hw1::TacticData>(
        ".tactics.xml",
        "TacticData",
        |tactics| {
            weapons += tactics.weapons.len();
            actions += tactics.actions.len();
        },
    );
    assert!(count > 190, "unexpectedly few tactics fixtures: {count}");
    assert!(
        weapons > count,
        "tactics fixtures did not produce typed weapons"
    );
    assert!(
        actions > count,
        "tactics fixtures did not produce typed actions"
    );
}

#[test]
fn shipped_physics_files_have_no_unmapped_fields() {
    let count =
        assert_asset_family_schema::<database::hw1::Physics>(".physics.xml", "physics", |_| {});
    assert!(count > 60, "unexpectedly few physics fixtures: {count}");
}

#[test]
fn shipped_blueprints_have_no_unmapped_fields() {
    let count = assert_asset_family_schema::<database::hw1::Blueprint>(
        ".blueprint.xml",
        "blueprint",
        |_| {},
    );
    assert!(count > 60, "unexpectedly few blueprint fixtures: {count}");
}

#[test]
fn shipped_shapes_have_no_unmapped_fields() {
    let mut objects = 0;
    let count = assert_asset_family_schema::<database::hw1::Shape>(".shp.xml", "hke", |shape| {
        objects += shape.objects.len();
    });
    assert!(count > 60, "unexpectedly few shape fixtures: {count}");
    assert!(
        objects > count,
        "shape fixtures did not produce typed objects"
    );
}
