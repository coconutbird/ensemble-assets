# ensemble-assets

Game data loading for Halo Wars: Definitive Edition (HW1). This workspace turns the files in the
game ERA archives into typed Rust data. It also edits that data and saves it to an override
directory.

## Crates

The crates are in four layers. A crate depends only on crates in lower layers (see
`AGENTS.md`).

| Layer | Crate | Purpose |
| ----- | ----- | ------- |
| 1 | `assets` | `no_std` traits that resolve game assets (`AssetResolver`, `FileProvider`). |
| 2 | `database` | `no_std` parser that changes XMB documents into typed structs (`Database`, `ProtoObject`, `Visual`, `TacticData`, ...). |
| 3 | `pipeline` | ERA loading, scenario loading, asset resolution, lightsets, string tables, validation, and the edit and save workflow (`World`, `AssetSource`). |
| 4 | `pipeline-cli` | Command-line tool to validate, load and save the game data. |
| 4 | `watch` | File watcher that reloads changed override files into a `World` (`WorldWatcher`). |
| 4 | `test-utils` | Test helpers. `require_game_dir` reads the game directory for ignored tests. |

`database` and `assets` do no file I/O. `pipeline` supplies the bytes.

## Dependencies and consumers

- The binary file formats (`era`, `xmb`, `bdt`, `bdt-serde`, `ugx`, `uax`, `ddx`, `xsd`,
  `xtd`, `xtt`) come from the `ensemble-formats` repository as git dependencies.
- `openensemble` uses this workspace through one path dependency:
  `pipeline = { path = "../ensemble-assets/crates/pipeline" }`.

## Tests

Most tests run without game files. Tests that need the installed game have
`#[ignore = "requires HW1_GAME_DIR"]`.

1. Set `HW1_GAME_DIR` to the game directory. You can set it in the environment or in a `.env`
   file at the workspace root.
2. Run the ignored tests:

```sh
cargo test --workspace -- --ignored
```

## pipeline-cli

Show the commands:

```sh
cargo run -p pipeline-cli -- --help
```

Commands:

- `validate --game-dir <DIR> [--era <ERA>]...`: parse each database XMB file and show the
  result.
- `load --game-dir <DIR> [--scenario <ERA>]`: load the full world and show a summary.
- `save --game-dir <DIR> --out <DIR> [--mode unpacked|packed] [--scenario <ERA>]`: write the
  database, visuals, tactics and physics to an override directory as XML or XMB.

## Documents

- `docs/asset-resolution.md`: how the engine maps an object to its visual, model, animation,
  damage and physics files.

## Data coverage

This section compares the data that the retail engine `xgameFinal.exe` loads with the data
that this workspace loads. The engine evidence comes from IDA (function name and address). The
tag diff also uses the XML in `root.era` and `root_update.era`.

### Files

Loaded now:

- `database`: `objects`, `squads`, `techs`, `abilities`, `powers`, `civs`, `leaders`,
  `weapontypes`, `damagetypes`, `gamedata` (`data\*.xml.xmb`), `*.vis`, `*.tactics`,
  `*.physics`, `*.blueprint`, `*.shp`, `*.pfx`.
- `pipeline`: `.scn` scenario data, `scenariodescriptions.xml`, `stringtable-<lang>.xml`,
  `.gls` and `.fls` lightsets, `.xsd`, `.xtd`, `.xtt` terrain, and the preload lists
  `visFileList.txt`, `tfxFileList.txt` and `pfxFileList.txt`.
- `openensemble` (not this workspace) reads `terrainTileTypes.xml`, `impacteffects.xml`, `.tfx`
  and `.lgt` in `crates/render`.

Not loaded (the engine loads these; no crate in the three repositories parses them):

| File | Engine evidence | Notes |
| ---- | --------------- | ----- |
| `objects_update.xml`, `squads_update.xml`, `techs_update.xml` | strings @ `0x1411a36b8`, `0x1411a37f0`, `0x1411a37d0` | Patch overlays. `ProtoObject` has an update flag, but no loader merges these files. |
| `.sc2`, `.sc3` | `BScenario_load` @ `0x140377be0` (strings @ `0x1411b6f90`, `0x1411b6f88`) | Companion files of the `.scn`, read with the same reader. |
| `.lrp` | `BScenario_load` @ `0x140377be0` (string @ `0x1411b713c`) | Read in the `Pathing` branch of the scenario load. |
| `.xth` | `sub_140667A90` (string @ `0x1411d68e0`) | Terrain file next to `.xtd` and `.xtt`. |
| `objecttypes.xml` | string @ `0x1411a2688` | `gamedata` reads only `CodeObjectTypes`. |
| `playercolors.xml` | "BDatabase: Loaded playerColors.xml" @ `0x1411a4f60` | |
| `hpbars.xml`, `impactsounds.xml`, `rumble.xml` | strings @ `0x1411a35c8`, `0x1411a33e8`, `0x1411a3418` | |
| `gamemodes.xml`, `deathmanager.xml`, `fatalityData.xml`, `skulls.xml`, `concepts.xml` | strings @ `0x1411a53b8`, `0x1411a55d0`, `0x1411a6f50`, `0x1411b8508`, `0x1411a8cd8` | |
| `preloadxmlfiles.xml` | `BDatabase_setupPreloadXmlFiles` @ `0x1401fcab0` | |
| `soundInfo.xml`, `soundTable.xml`, `chattertable.xml` | strings @ `0x1411c9310`, `0x1411a2608`, `0x1411e8620` | Sound data. |
| `fogOfWar.xml`, `gamesettings.xml`, `stats.xml`, `userclasses.xml`, `achievements.xml` | strings @ `0x1411daa68`, `0x1411a8860`, `0x1411bb638`, `0x1411c11e8`, `0x14118f870` | |
| `aidifficultysettings.xml`, `aifactoids.xml`, `data\aidata\*` | strings @ `0x141196b38`, `0x141196ff8` | AI data. |
| `.cin` | string `*.cin` @ `0x1411abab0` | The manifest records `.cin` paths, but no crate parses them. |
| `dlc%02u.xml` | string @ `0x1411a8448` | `loader.rs` loads `dlcNN.era`, but not the DLC XML files. |

UI files (`art\ui\flash\**.xml`, `renderPasses.xml`, `renderTargets.xml`, `fonts.xml`,
`gamepads.xml`, `controllerconfigs.xml`) are also not loaded. They are outside the scope of the
data layer.

### Tags

The engine reads tags with string compares in the `load` functions below. The table lists tags
that occur in the retail XML but that our serde structs in `crates/database/src/hw1/**` do not
read.

| Schema | Engine loader | Tags in retail XML that we do not read |
| ------ | ------------- | -------------------------------------- |
| objects | `BProtoObject_load` @ `0x140341620` | `@direction`; `CircleMenuFrameIDNormal`, `CircleMenuFrameIDHighlight` (no engine string: data only) |
| squads | `BProtoSquad_load` @ `0x140350d90`, `BDatabase_setupProtoSquads` | `MergedSquads`/`MergedSquad`, `ShieldBubbleTypes`/`ShieldBubble` (engine reads both in `BDatabase_setupProtoSquads`) |
| techs | `BProtoTech_load` @ `0x140356ad0`, `BTechEffect_load` @ `0x14040e2c0` | `TechDataCommandType`, `TechDataRelativity`, `TechDataSubType`, `TechDataSubTypeProtoSquad` (no engine string: data only) |
| powers | `BProtoPower_load` @ `0x14034cb00` | `CircleMenuFrameIDNormal`, `CircleMenuFrameIDHighlight` (data only) |
| tactics | `BTactic_loadTactic` @ `0x140408e80`, `BWeapon_loadWeapon` @ `0x1403fea70`, `BProtoAction_loadProtoAction` @ `0x140401f40` | `DodgePercent`, `HoverAltitude`, `ReloadAnimation`, `RestrictedAutoTarget`, `Walk{Forward,Back,Left,Right}Anim` (no engine string: data only) |
| physics | `BPhysicsInfo_load` @ `0x1402d3cd0` | `ImpactSound` (engine string @ `0x1411a19b8`, read in `sub_1401D83B0`); `DestroyedByBallistics`, `ThrownByBallistics` (data only) |
| blueprint | `BPhysicsObjectBlueprint_load` @ `0x1407ba2f0` | `angularDampening` (data only) |
| pfx | particle loader `sub_14079D440` | `Masks` (engine string @ `0x1411e2ba4`); `UseExternalWind`, `ExternalWindDelay`, `ExternalWindInfluence`, `CycleTime`, `CycleVariance`, `GradientPoint` (data only) |

Abilities, civs, leaders, weapon types, damage types, game data and visuals have no unread tag
in the retail XML.

"Data only" means that the tag is in the retail XML but `xgameFinal.exe` has no string for it.
The engine thus ignores it, so it is not a gap.

The engine load functions also hold strings that the retail XML does not use. Most are enum
values and flag names (for example the `Flag` values of objects). Approximate counts of such
strings that our structs do not name: objects 222, techs 136, squads 63, tactics 35, powers 27,
abilities 27, physics 24, visuals 22. These counts include enum values, so they are an upper
limit, not a list of missing tags.

Tags that we read but that the engine does not name in its load functions: objects
`DazeResist`, `FlashUI`, `MinimapIconName`, `TrackInterceptDistance`, `UIVisual`; squads
`MinimapIcon`; powers `CampaignFoo`, `Collectable`, `FlashUI`, `Supplies`; visuals `Cycles`,
`Stages`, `ValueVariance`. The engine can read some of these in helper functions deeper than one
call level, so treat this list as a hint.
