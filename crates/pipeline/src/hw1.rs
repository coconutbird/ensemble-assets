//! Halo Wars 1 asset pipeline.
//!
//! Provides the full HW1 asset loading pipeline:
//!
//! 1. **ERA loading** ([`loader`]) — mirrors the engine's `BArchiveManager` load order
//! 2. **Database loading** — objects, squads, techs, abilities, civs, leaders, …
//! 3. **Asset resolution** ([`resolve`]) — visual → model/anim, tactics, physics → blueprint → shape
//! 4. **Scenario loading** ([`scenario`]) — scenario-specific ERA layering and map descriptors
//! 5. **Edit & save** ([`edit`], [`World`]) — RAII dirty tracking with selective serialization
//! 6. **Validation** ([`mod@validate`], [`diagnostics`]) — structured cross-reference checks
//! 7. **Manifest** ([`manifest`]) — passive inventory of all binary asset references
//! 8. **String table** ([`stringtable`]) — localized string lookup by `_locID`
//! 9. **Terrain textures** ([`terrain`]) — terrain texture paths and decoded terrain maps
//! 10. **Particle effects** ([`pfx`]) — PFX path resolution and loading
//! 11. **Startup configuration** ([`startup`]) — retail `startup\*.cfg` symbols
//!
//! # Usage
//!
//! ```no_run
//! use pipeline::hw1::World;
//!
//! // Load the base world, then load a scenario.
//! let (mut world, mut src) = World::load("/path/to/HaloWarsDE")
//!     .expect("failed to load world");
//! world.swap_scenario(&mut src, "blood_gulch").expect("failed to load scenario");
//!
//! println!("Loaded {} objects", world.database.objects.len());
//! println!("Resolved {} visuals", world.visuals.len());
//!
//! // Validate cross-references (objects → visuals → tactics, etc.)
//! let report = pipeline::hw1::validate_world(&world);
//! report.print_summary();
//! ```

pub mod diagnostics;
pub mod edit;
pub mod lightset;
pub mod loader;
pub mod manifest;
pub mod merge;
pub mod pfx;
pub mod resolve;
pub mod scenario;
pub mod startup;
pub mod stringtable;
pub mod terrain;
pub mod validate;
pub mod world;

pub use diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticReport, Location, Severity, validate_world,
    validate_world_assets,
};
pub use edit::{AssetKind, DirtyGuard, DirtySet, TableId};
pub use lightset::LightSetData;
pub use manifest::{AssetManifest, BinaryValidation, VerifyResult};
pub use merge::{Conflict, MergeError, MergeOptions, MergeReport, ModOverlay, merge_mods};
pub use resolve::{LoadStats, ObjectAssets, PhysicsChain};
pub use scenario::{
    CinematicRef, ObjectiveRef, ScenarioData, ScenarioDescriptor, ScenarioObject,
    ScenarioObjectFile, ScenarioPlayer, ScenarioPosition, TalkingHeadRef,
};
pub use validate::{ValidateReport, validate};
pub use world::{World, WorldLoadOptions};
