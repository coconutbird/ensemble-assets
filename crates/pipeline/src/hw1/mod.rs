//! Halo Wars 1 asset pipeline.
//!
//! Provides the full HW1 asset loading pipeline:
//!
//! 1. **ERA loading** — mirrors the engine's `BArchiveManager` load order
//! 2. **Database loading** — objects, squads, techs, abilities, etc.
//! 3. **Asset resolution** — visuals, tactics, physics chains per object
//! 4. **Scenario loading** — scenario-specific ERA layering and descriptors
//!
//! # Usage
//!
//! ```no_run
//! use pipeline::hw1::World;
//!
//! let world = World::load("/path/to/game", Some("scenario/skirmish/design/blood_gulch/blood_gulch.scn"))
//!     .expect("failed to load world");
//!
//! println!("Loaded {} objects", world.database.objects.len());
//! println!("Resolved {} visuals", world.visuals.len());
//! ```

pub mod loader;
pub mod manifest;
pub mod resolve;
pub mod scenario;
pub mod validate;
pub mod world;

pub use manifest::{AssetManifest, BinaryValidation, VerifyResult};
pub use resolve::{LoadStats, ObjectAssets, PhysicsChain};
pub use scenario::{
    ScenarioData, ScenarioDescriptor, ScenarioObject, ScenarioPlayer, ScenarioPosition,
};
pub use validate::{ValidateReport, validate};
pub use world::World;
