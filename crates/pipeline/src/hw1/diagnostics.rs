//! Structured diagnostics for HW1 asset validation.
//!
//! Every diagnostic carries a game path, a location within that file,
//! a severity, a machine-readable code, and a human-readable message.
//! This is the foundation for language-server-style feedback.

use std::fmt;

use super::edit::TableId;

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Prevents the game from loading or causes crashes.
    Error,
    /// Likely a bug but the game can still load.
    Warning,
    /// Informational — style or schema notes.
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Machine-readable diagnostic category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    /// XMB/XML parse failure.
    ParseError,
    /// A field in the data that the struct does not declare.
    UnknownField,
    /// A value whose type didn't match expectations.
    TypeMismatch,
    /// A cross-reference to something that doesn't exist
    /// (e.g. object references a visual that isn't loaded).
    MissingReference,
    /// Two entries share the same unique key.
    DuplicateName,
    /// A binary asset (`.ugx`, `.uax`, `.ddx`) is referenced but not found.
    MissingAsset,
    /// A database file expected in the ERAs was not found at all.
    MissingFile,
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticCode::ParseError => write!(f, "parse-error"),
            DiagnosticCode::UnknownField => write!(f, "unknown-field"),
            DiagnosticCode::TypeMismatch => write!(f, "type-mismatch"),
            DiagnosticCode::MissingReference => write!(f, "missing-reference"),
            DiagnosticCode::DuplicateName => write!(f, "duplicate-name"),
            DiagnosticCode::MissingAsset => write!(f, "missing-asset"),
            DiagnosticCode::MissingFile => write!(f, "missing-file"),
        }
    }
}

/// Where inside a file/table the issue was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// The whole file/table.
    Table(TableId),
    /// A specific named entry within a table (e.g. object "warthog").
    Entry { table: TableId, name: String },
    /// A specific field on a named entry.
    Field {
        table: TableId,
        name: String,
        field: String,
    },
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Table(t) => write!(f, "{t:?}"),
            Location::Entry { table, name } => write!(f, "{table:?}::{name}"),
            Location::Field { table, name, field } => {
                write!(f, "{table:?}::{name}.{field}")
            }
        }
    }
}

/// A single structured diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Game path (e.g. `"data\\objects.xml"`).
    pub path: String,
    /// Where in the file the issue was found.
    pub location: Location,
    /// Severity level.
    pub severity: Severity,
    /// Machine-readable category.
    pub code: DiagnosticCode,
    /// Human-readable description.
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({}): {} — {}",
            self.severity, self.path, self.location, self.code, self.message
        )
    }
}

/// Aggregate diagnostics from a validation pass.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticReport {
    pub diagnostics: Vec<Diagnostic>,
    pub elapsed: std::time::Duration,
}

impl DiagnosticReport {
    pub fn errors(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    pub fn warnings(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    pub fn infos(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .count()
    }

    /// Print a summary to stdout.
    pub fn print_summary(&self) {
        for d in &self.diagnostics {
            println!("  {d}");
        }
        println!(
            "\n  {} errors, {} warnings, {} info ({:.1}s)",
            self.errors(),
            self.warnings(),
            self.infos(),
            self.elapsed.as_secs_f64()
        );
    }
}

/// Run cross-reference validation on a loaded [`World`](super::World).
///
/// Checks:
/// - Each object's `visual` references a loaded visual
/// - Each object's `tactics` references a loaded tactics file
/// - Each object's `physics_info` references a loaded physics entry
/// - Each squad's unit entries reference valid proto-object names
/// - No duplicate object or squad names
pub fn validate_world(world: &super::World) -> DiagnosticReport {
    let start = std::time::Instant::now();
    let mut diagnostics = Vec::new();

    // Build lookup sets
    let object_names: std::collections::HashSet<&str> = world
        .database
        .objects
        .iter()
        .map(|o| o.name.as_str())
        .collect();

    // Check for duplicate object names
    {
        let mut seen = std::collections::HashSet::new();
        for obj in &world.database.objects {
            if !obj.name.is_empty() && !seen.insert(&obj.name) {
                diagnostics.push(Diagnostic {
                    path: "data\\objects.xml".into(),
                    location: Location::Entry {
                        table: TableId::Objects,
                        name: obj.name.clone(),
                    },
                    severity: Severity::Warning,
                    code: DiagnosticCode::DuplicateName,
                    message: format!("duplicate object name '{}'", obj.name),
                });
            }
        }
    }

    // Check for duplicate squad names
    {
        let mut seen = std::collections::HashSet::new();
        for squad in &world.database.squads {
            if !squad.name.is_empty() && !seen.insert(&squad.name) {
                diagnostics.push(Diagnostic {
                    path: "data\\squads.xml".into(),
                    location: Location::Entry {
                        table: TableId::Squads,
                        name: squad.name.clone(),
                    },
                    severity: Severity::Warning,
                    code: DiagnosticCode::DuplicateName,
                    message: format!("duplicate squad name '{}'", squad.name),
                });
            }
        }
    }

    // Object → visual cross-reference
    for obj in &world.database.objects {
        if let Some(ref vis_name) = obj.visual
            && !vis_name.is_empty()
            && !world.visuals.contains_key(vis_name)
        {
            diagnostics.push(Diagnostic {
                path: "data\\objects.xml".into(),
                location: Location::Field {
                    table: TableId::Objects,
                    name: obj.name.clone(),
                    field: "Visual".into(),
                },
                severity: Severity::Warning,
                code: DiagnosticCode::MissingReference,
                message: format!(
                    "object '{}' references visual '{}' which was not loaded",
                    obj.name, vis_name
                ),
            });
        }
    }

    // Object → tactics cross-reference
    for obj in &world.database.objects {
        if let Some(ref tac_name) = obj.tactics
            && !tac_name.is_empty()
            && !world.tactics.contains_key(tac_name)
        {
            diagnostics.push(Diagnostic {
                path: "data\\objects.xml".into(),
                location: Location::Field {
                    table: TableId::Objects,
                    name: obj.name.clone(),
                    field: "Tactics".into(),
                },
                severity: Severity::Warning,
                code: DiagnosticCode::MissingReference,
                message: format!(
                    "object '{}' references tactics '{}' which was not loaded",
                    obj.name, tac_name
                ),
            });
        }
    }

    // Object → physics cross-reference
    for obj in &world.database.objects {
        if let Some(ref phys_name) = obj.physics_info
            && !phys_name.is_empty()
            && !world.physics.contains_key(phys_name)
        {
            diagnostics.push(Diagnostic {
                path: "data\\objects.xml".into(),
                location: Location::Field {
                    table: TableId::Objects,
                    name: obj.name.clone(),
                    field: "PhysicsInfo".into(),
                },
                severity: Severity::Info,
                code: DiagnosticCode::MissingReference,
                message: format!(
                    "object '{}' references physics '{}' which was not loaded",
                    obj.name, phys_name
                ),
            });
        }
    }

    // Squad → object cross-reference
    for squad in &world.database.squads {
        if let Some(ref units) = squad.units {
            for unit in &units.entries {
                if !unit.proto_object.is_empty()
                    && !object_names.contains(unit.proto_object.as_str())
                {
                    diagnostics.push(Diagnostic {
                        path: "data\\squads.xml".into(),
                        location: Location::Field {
                            table: TableId::Squads,
                            name: squad.name.clone(),
                            field: "Units".into(),
                        },
                        severity: Severity::Warning,
                        code: DiagnosticCode::MissingReference,
                        message: format!(
                            "squad '{}' references object '{}' which does not exist",
                            squad.name, unit.proto_object
                        ),
                    });
                }
            }
        }
    }

    DiagnosticReport {
        diagnostics,
        elapsed: start.elapsed(),
    }
}
