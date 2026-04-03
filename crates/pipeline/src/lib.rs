//! Ensemble asset pipeline — ERA loading, database parsing, asset resolution,
//! scenario management, and edit/save workflows for Halo Wars titles.
//!
//! This crate ties together the lower-level format crates ([`ugx`], [`ddx`],
//! [`uax`], [`xmb`], [`database`]) into a single, high-level API for loading
//! a complete game world and optionally modifying it.
//!
//! # Quick start
//!
//! ```no_run
//! use pipeline::hw1::World;
//!
//! // Load the base world, then load a scenario.
//! let (mut world, mut src) = World::load("/path/to/HaloWarsDE")
//!     .expect("failed to load world");
//! world.swap_scenario(&mut src, "blood_gulch");
//!
//! // Inspect loaded data.
//! println!("{} objects", world.database.objects.len());
//! println!("{} visuals resolved", world.visuals.len());
//! world.print_summary();
//!
//! // Edit a unit's hit points via the dirty-tracking API.
//! {
//!     let mut objects = world.objects_mut(); // returns DirtyGuard
//!     if let Some(obj) = objects.iter_mut().find(|o| o.name == "unsc_inf_marine_01") {
//!         obj.hitpoints = Some(200.0);
//!     }
//! } // DirtyGuard drops here → Objects table marked dirty
//!
//! // Save only the tables that changed.
//! // world.save(&src).expect("save failed");
//! ```
//!
//! # Modules
//!
//! - [`source`] — ERA-backed asset resolution with filesystem override layer.
//! - [`hw1`] — Halo Wars 1 world loading, editing, validation, and serialization.
//!
//! # Re-exports
//!
//! The lower-level format crates are re-exported so downstream consumers
//! only need to depend on `pipeline`:

pub use database;
pub use ddx;
pub use uax;
pub use ugx;
pub use xmb;
pub use xtd;
pub use xtt;
pub mod hw1;
pub mod source;

/// Errors that can occur during pipeline operations.
///
/// Most methods on [`hw1::World`] return [`Result<T>`](crate::Result) using
/// this error type. For finer-grained error handling (e.g. distinguishing
/// a missing file from a parse error), match on the variants.
#[derive(Debug)]
pub enum Error {
    /// An ERA archive could not be opened or parsed.
    Era(String),
    /// A database XML/XMB file failed to parse or deserialize.
    Database(database::Error),
    /// An XMB binary file was malformed.
    Xmb(xmb::Error),
    /// A required asset was not found in any loaded ERA or override directory.
    NotFound(String),
}

impl From<database::Error> for Error {
    fn from(e: database::Error) -> Self {
        Self::Database(e)
    }
}

impl From<xmb::Error> for Error {
    fn from(e: xmb::Error) -> Self {
        Self::Xmb(e)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Era(e) => write!(f, "ERA error: {e}"),
            Self::Database(e) => write!(f, "database error: {e}"),
            Self::Xmb(e) => write!(f, "XMB error: {e}"),
            Self::NotFound(path) => write!(f, "asset not found: {path}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
