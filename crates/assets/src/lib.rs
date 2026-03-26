//! `no_std` asset abstractions for Ensemble tooling.
//!
//! Two levels of abstraction:
//!
//! - [`FileProvider`] — low-level I/O: "read bytes from this filesystem path".
//!   Implemented by the caller to swap between `std::fs::read`, mmap, etc.
//!
//! - [`AssetResolver`] — high-level game assets: "give me bytes for this
//!   virtual game path" (e.g. `"data\\objects.xml.xmb"`).  The `database`
//!   crate uses this to load XMB documents without knowing anything about
//!   ERAs, load order, or filesystems.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// Low-level: FileProvider
// ---------------------------------------------------------------------------

/// Provides raw file bytes from some backing store.
///
/// The associated [`Data`](Self::Data) type represents the owned handle to
/// the file contents.  It must deref to `&[u8]` via [`AsRef<[u8]>`] so
/// that consumers can work with the bytes generically.
///
/// - For `std::fs::read` → `Data = Vec<u8>`
/// - For mmap → `Data = Mmap` (the mmap handle keeps the mapping alive)
/// - For tests → `Data = &'static [u8]` or `Vec<u8>`
pub trait FileProvider {
    /// Handle to the file contents — must deref to `&[u8]`.
    type Data: AsRef<[u8]>;

    /// Read / map a file by filesystem path, returning a handle to its bytes.
    ///
    /// Returns `None` if the file does not exist.
    fn open(&self, path: &str) -> Option<Self::Data>;

    /// Check whether a file exists without reading it.
    fn exists(&self, path: &str) -> bool;
}

// ---------------------------------------------------------------------------
// High-level: AssetResolver
// ---------------------------------------------------------------------------

/// Resolve game assets by virtual path.
///
/// Implementations provide access to game files regardless of the backing
/// store — ERA archives, loose files, in-memory buffers, etc.
///
/// Paths use the engine convention: **lowercase with backslash separators**
/// (e.g. `"data\\objects.xml.xmb"`).
///
/// The `database` crate uses this trait to load its XMB documents.
/// Concrete implementations (ERA-backed, filesystem, test harness) live
/// in consumer crates.
pub trait AssetResolver {
    /// Read a file by virtual path, returning the raw bytes.
    ///
    /// Returns `None` if the file does not exist in any backing store.
    fn resolve(&mut self, path: &str) -> Option<Vec<u8>>;

    /// Check whether a file exists without reading it.
    fn exists(&self, path: &str) -> bool;
}
