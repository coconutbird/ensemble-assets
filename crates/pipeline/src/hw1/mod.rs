//! Halo Wars 1 asset pipeline.
//!
//! Provides the full HW1 asset loading pipeline:
//!
//! 1. **ERA loading** ([`loader`]) — mirrors the engine's `BArchiveManager` load order
//! 2. **Database loading** — objects, squads, techs, abilities, civs, leaders, …
//! 3. **Asset resolution** ([`resolve`]) — visual → model/anim, tactics, physics → blueprint → shape
//! 4. **Scenario loading** ([`scenario`]) — scenario-specific ERA layering and map descriptors
//! 5. **Edit & save** ([`edit`], [`World`]) — RAII dirty tracking with selective serialization
//! 6. **Validation** ([`validate`], [`diagnostics`]) — structured cross-reference checks
//! 7. **Manifest** ([`manifest`]) — passive inventory of all binary asset references
//! 8. **String table** ([`stringtable`]) — localized string lookup by `_locID`
//!
//! # Usage
//!
//! ```no_run
//! use pipeline::hw1::World;
//!
//! // Load world — accepts a map name, ERA filename, or full SCN path.
//! let world = World::load("/path/to/HaloWarsDE", Some("blood_gulch"))
//!     .expect("failed to load world");
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
pub mod loader;
pub mod manifest;
pub mod resolve;
pub mod scenario;
pub mod stringtable;
pub mod validate;
pub mod world;

pub use diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticReport, Location, Severity, validate_world,
    validate_world_assets,
};
pub use edit::{AssetKind, DirtyGuard, DirtySet, TableId};
pub use manifest::{AssetManifest, BinaryValidation, VerifyResult};
pub use resolve::{LoadStats, ObjectAssets, PhysicsChain};
pub use scenario::{
    CinematicRef, ObjectiveRef, ScenarioData, ScenarioDescriptor, ScenarioObject, ScenarioPlayer,
    ScenarioPosition, TalkingHeadRef,
};
pub use validate::{ValidateReport, validate};
pub use world::World;
