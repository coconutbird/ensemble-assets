# HW1 asset resolution

This document tells how Halo Wars: Definitive Edition finds the files of a game object. It also
tells which parts the `pipeline` crate implements. Retail data has no `.dep` files. The XML data
holds all references.

## Chain

```
objects.xml.xmb  <Object name="unsc_veh_warthog_01">
  ├─ <Visual>       → art\unsc\vehicle\warthog_01\warthog_01.vis.xmb
  │                     ├─ <asset type="Model"><file>  → art\<file>.ugx
  │                     ├─ <asset type="Anim"><file>   → art\<file>.uax
  │                     ├─ <damagefile>                → art\<damagefile>.dmg.xmb
  │                     └─ <attach type="ModelRef">    → other <model> in the same file
  ├─ <Tactics>      → data\tactics\<name>.tactics.xmb
  └─ <PhysicsInfo>  → physics\<name>.physics.xmb
                        └─ <blueprint> → physics\<name>.blueprint.xmb
                                           └─ shape → physics\<name>.shp.xmb
```

`crates/pipeline/src/hw1/resolve.rs` follows this chain for each object and stores the result in
`ObjectAssets`. `crates/pipeline/src/hw1/manifest.rs` collects the model, animation, texture,
scenario and preload paths in `AssetManifest`.

## ERA archives

| Archive | Contents |
| ------- | -------- |
| `root.era` | Data and manifests: `data\*.xml.xmb`, `.vis.xmb`, `.tactics.xmb`, physics files, `.pfx`. |
| `root_update.era` | Patch files, for example `objects_update.xml.xmb`. |
| `scenarioshared.era` | Art: `.ugx`, `.uax`, `.ddx` and the 58 `.dmg.xmb` files. |
| `locale.era` | Localized strings. |
| `shader.era`, `miniloader.era`, `pregameUI.era`, `ingameUI.era`, `loadingUI.era` | Shaders and UI. |
| `dlc01.era`, `dlc02.era` | DLC content. |
| `PHXscn*.era`, `<map>.era`, `release.era`, `repository.era` | Data for one scenario: `.scn`, terrain, preload lists and art that only that map uses. |

### Load order

The engine loads `root.era` and then `root_update.era` in `BArchiveManager_reloadRootArchive`
@ `0x14017CE90`. It loads the locale archives separately and the scenario archive when a scenario
starts.

`crates/pipeline/src/hw1/loader.rs` uses this order (lowest priority first): `locale.era`,
`root.era`, `root_update.era`, `shader.era`, `miniloader.era`, `pregameUI.era`, `ingameUI.era`,
`scenarioshared.era`, `dlc01.era` to `dlc10.era`. `load_with_scenario` puts the scenario ERA last.
The loader does not load `loadingUI.era` (engine string @ `0x141199910`).
`AssetSource` (`crates/pipeline/src/source.rs`) gives the file from the last archive that has the
path. It normalizes paths to lowercase with backslash separators. Folder layers and the override
directory are above all ERAs.

### Update files

`root_update.era` adds new files. It also holds `objects_update.xml.xmb`,
`squads_update.xml.xmb` and `techs_update.xml.xmb`. Elements in these files have
`update="true"`. The engine merges them into the entries with the same name.

```xml
<Object name="game_CTF_flag_01" is="0" id="2061" dbid="4355" update="true">
    <PhysicsInfo>sentinel</PhysicsInfo>
    <ObjectClass>Squad</ObjectClass>
</Object>
```

Gap: `pipeline` does not load or merge the `*_update.xml` files. `ProtoObject` has the update
flag only.

## Database files

- `data\objects.xml.xmb` has about 2500 objects. An object has no parent. All properties are in
  the object element.
- `data\squads.xml.xmb` lists the units of each squad.
- `<Tactics>` gives a file name without a directory. The file is in `data\tactics\`.
- `<PhysicsInfo>` and `<PhysicsReplacementInfo>` give a bare name. The engine adds `physics\`
  and the extension.

The `.shp.xmb` files are Havok shapes in XML form (`<hke>` root with `hkBoxShape` and other
shapes).

## Visual files

A `.vis.xmb` file has one or more `<model>` elements. `defaultmodel` names the first model. A
`<component>` holds `<asset>` elements and `<attach>` elements. A `<logic type="Tech">` element
holds other assets that the engine uses after a tech is researched. A `ModelRef` attachment names
another `<model>` in the same file. Many attachments can name the same model with different
bones (for example four `Wheel` attachments).

### Paths

- `<Visual>` in `objects.xml` has no `art\` prefix and no `.xmb` extension. The engine adds both.
  `BDatabase_preloadVisFiles` @ `0x1401F1AA0` sets the `art\` prefix.
- `<file>` paths in a visual have no extension. `BGrannyModel::loadFromFile` @ `0x14073D6F0`
  adds the `art\` base path and the `.ugx` extension. Animations use `.uax`.
- `<damagefile>` names a damage model. The file is `art\<damagefile>.dmg.xmb` in
  `scenarioshared.era`.

`manifest.rs` (`visit_asset_refs`) adds `art\` and `.ugx` or `.uax`, and it visits the
`<logic>` assets too.

Gap: `visit_asset_refs` makes `art\<damagefile>.ugx` for a damage file. That file does not exist
in the retail data (for example `art\unsc\vehicle\warthog_01\warthog_01.dmg.xmb` exists but
`warthog_01.ugx` does not). The correct path is `art\<damagefile>.dmg.xmb`.

## Damage models

Gap: no crate in the three repositories parses `.dmg.xmb` files.

A `.dmg.xmb` file has a `<damagetemplate>` root with two sections:

- `<percentagebased>`: events at health thresholds. Actions are `multiframeTextureIndex`
  (change the damage texture frame), `attachparticle` (add an effect) and `throwpart` (detach a
  mesh part with physics).
- `<impactpointbased>`: events at bones `bone_impact_01` to `bone_impact_05`. Actions are
  `swappart` (change a part to its damaged version) and `throwpart`.

The actions name mesh parts in the `.ugx`, not other files.

```xml
<damagetemplate>
    <percentagebased>
        <event><action type="multiframeTextureIndex" fromindex="0" toindex="1"/></event>
    </percentagebased>
    <impactpointbased>
        <impactpoint name="bone_impact_01" bone="bone_impact_01">
            <event><action type="swappart">fender2,damaged_fender2</action></event>
        </impactpoint>
    </impactpointbased>
</damagetemplate>
```

## Textures

The `.ugx` material data names the textures. They are `.ddx` files next to the model, with the
suffixes `_df`, `_sp`, `_nm`, `_em` and `_xf`. `World::resolve_textures` reads them from the
models.

## LCE variants

The Legendary Edition variants (for example `warthoglce_01.vis.xmb`) are separate proto objects.
`sub_140392F90` handles the `LCE_warthog_unlock` and `LCE_wraith_unlock` unlocks. `pipeline`
loads these objects like all other objects. It does not implement the unlock state.
