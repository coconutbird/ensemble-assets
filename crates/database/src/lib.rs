//! Ensemble game database library.
//!
//! Pure `no_std` parser — takes XMB documents in, gives typed structs out.
//! Has no idea where the bytes come from (ERA, filesystem, network, etc.).
//!
//! Game-specific schemas live in sub-modules (`hw1`, future `hw2`).

#![no_std]
extern crate alloc;

pub mod hw1;
pub mod node_ext;

use alloc::string::String;

/// Errors that can occur when parsing database files.
#[derive(Debug)]
pub enum Error {
    Xmb(xmb::Error),
    Deserialize(bdt_serde::Error),
    MissingRoot,
    UnexpectedRoot { expected: String, actual: String },
}

impl From<xmb::Error> for Error {
    fn from(e: xmb::Error) -> Self {
        Self::Xmb(e)
    }
}

impl From<bdt_serde::Error> for Error {
    fn from(e: bdt_serde::Error) -> Self {
        Self::Deserialize(e)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Xmb(e) => write!(f, "xmb error: {e}"),
            Self::Deserialize(e) => write!(f, "deserialize error: {e}"),
            Self::MissingRoot => f.write_str("missing root element"),
            Self::UnexpectedRoot { expected, actual } => {
                write!(
                    f,
                    "unexpected root element: expected '{expected}', got '{actual}'"
                )
            }
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
