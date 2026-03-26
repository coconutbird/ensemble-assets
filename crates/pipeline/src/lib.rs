//! Ensemble asset pipeline — ERA loading, scenario loading, full world resolution.
//!
//! This crate provides the infrastructure for loading game assets from ERA
//! archives and resolving the full asset dependency chain. Game-specific
//! pipelines live in sub-modules (`hw1`, future `hw2`).
//!
//! # Architecture
//!
//! - [`source::AssetSource`] — ERA-backed asset resolver (generic over I/O)
//! - [`source::StdFileProvider`] — `std::fs::read`-backed file provider
//! - [`hw1`] — Halo Wars 1 asset pipeline (ERA load order, world loading)

pub mod hw1;
pub mod source;

/// Errors that can occur during pipeline operations.
#[derive(Debug)]
pub enum Error {
    /// ERA archive loading error.
    Era(String),
    /// Database parsing error.
    Database(database::Error),
    /// XMB parsing error.
    Xmb(xmb::Error),
    /// Asset not found.
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
