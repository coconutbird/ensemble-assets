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
    /// The XMB document is not valid.
    Xmb(xmb::Error),
    /// An XMB node does not deserialize into the typed schema.
    Deserialize(bdt_serde::Error),
    /// The document has no root element.
    MissingRoot,
    /// The root element name is not the name that the parser expects.
    UnexpectedRoot {
        /// Root element name that the parser expects.
        expected: String,
        /// Root element name in the document.
        actual: String,
    },
    /// Two records of a table have the same name, and the engine stops the database setup.
    DuplicateName {
        /// Kind of record, for example `"object"`.
        table: &'static str,
        /// Name of the second record.
        name: String,
    },
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
            Self::DuplicateName { table, name } => {
                write!(f, "duplicate {table} name '{name}'")
            }
        }
    }
}

/// Result type for database parsing. The default error is [`Error`].
pub type Result<T, E = Error> = core::result::Result<T, E>;
