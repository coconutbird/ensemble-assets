use pipeline::hw1::{World, WorldLoadOptions};
use pipeline::source::{AssetSource, LoadRule, StdFileProvider};

#[test]
fn runtime_load_defers_and_selectively_populates_visuals() {
    let fixture = tempfile::tempdir().expect("temporary asset directory");
    let data_dir = fixture.path().join("data");
    let art_dir = fixture.path().join("art");
    std::fs::create_dir_all(&data_dir).expect("data directory");
    std::fs::create_dir_all(&art_dir).expect("art directory");
    std::fs::write(
        data_dir.join("objects.xml.xmb"),
        br#"<Objects>
            <Object name="used"><Visual>used.vis</Visual></Object>
            <Object name="deferred"><Visual>deferred.vis</Visual></Object>
        </Objects>"#,
    )
    .expect("objects fixture");
    std::fs::write(
        art_dir.join("used.vis.xmb"),
        br#"<visual defaultmodel="Default">
            <model name="Default"><component>
                <asset type="Model"><file>objects\used</file></asset>
            </component></model>
        </visual>"#,
    )
    .expect("visual fixture");

    let mut source = AssetSource::with_provider(StdFileProvider);
    source
        .add_folder(&fixture.path().to_string_lossy(), LoadRule::Replace)
        .expect("fixture layer");
    let mut world = World::load_from_source_with_options(&mut source, WorldLoadOptions::runtime())
        .expect("runtime world");

    assert_eq!(world.database.objects.len(), 2);
    assert!(world.assets.is_empty());
    assert!(world.visuals.is_empty());

    assert_eq!(world.load_visuals_for(&mut source, ["USED", "used"]), 1);
    assert!(world.visuals.contains_key("used"));
    assert!(!world.visuals.contains_key("deferred"));
    assert_eq!(world.load_visuals_for(&mut source, ["used"]), 0);
    assert!(world.manifest.model_refs.contains("art\\objects\\used.ugx"));
}
