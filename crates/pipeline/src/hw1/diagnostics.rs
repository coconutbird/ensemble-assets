//! Structured diagnostics for HW1 asset validation.
//!
//! Every diagnostic carries a game path, a location within that file,
//! a severity, a machine-readable code, and a human-readable message.
//! This is the foundation for language-server-style feedback.

use std::fmt;

use assets::AssetResolver;

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

    // ── Build lookup sets ───────────────────────────────────────────
    let object_names: std::collections::HashSet<&str> = world
        .database
        .objects
        .iter()
        .map(|o| o.name.as_str())
        .collect();

    let squad_names: std::collections::HashSet<&str> = world
        .database
        .squads
        .iter()
        .map(|s| s.name.as_str())
        .collect();

    let tech_names: std::collections::HashSet<&str> = world
        .database
        .techs
        .iter()
        .map(|t| t.name.as_str())
        .collect();

    let civ_names: std::collections::HashSet<&str> = world
        .database
        .civs
        .iter()
        .map(|c| c.name.as_str())
        .collect();

    let weapon_type_names: std::collections::HashSet<&str> = world
        .database
        .weapon_types
        .iter()
        .map(|w| w.name.as_str())
        .collect();

    // ── Duplicate name checks ───────────────────────────────────────
    check_duplicates(
        &world
            .database
            .objects
            .iter()
            .map(|o| o.name.as_str())
            .collect::<Vec<_>>(),
        "data\\objects.xml",
        TableId::Objects,
        "object",
        &mut diagnostics,
    );
    check_duplicates(
        &world
            .database
            .squads
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        "data\\squads.xml",
        TableId::Squads,
        "squad",
        &mut diagnostics,
    );
    check_duplicates(
        &world
            .database
            .techs
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>(),
        "data\\techs.xml",
        TableId::Techs,
        "tech",
        &mut diagnostics,
    );
    check_duplicates(
        &world
            .database
            .abilities
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>(),
        "data\\abilities.xml",
        TableId::Abilities,
        "ability",
        &mut diagnostics,
    );
    check_duplicates(
        &world
            .database
            .powers
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        "data\\powers.xml",
        TableId::Powers,
        "power",
        &mut diagnostics,
    );
    check_duplicates(
        &world
            .database
            .leaders
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        "data\\leaders.xml",
        TableId::Leaders,
        "leader",
        &mut diagnostics,
    );
    check_duplicates(
        &world
            .database
            .civs
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        "data\\civs.xml",
        TableId::Civs,
        "civ",
        &mut diagnostics,
    );
    check_duplicates(
        &world
            .database
            .weapon_types
            .iter()
            .map(|w| w.name.as_str())
            .collect::<Vec<_>>(),
        "data\\weapontypes.xml",
        TableId::WeaponTypes,
        "weapon type",
        &mut diagnostics,
    );
    check_duplicates(
        &world
            .database
            .damage_types
            .iter()
            .map(|d| d.name.as_str())
            .collect::<Vec<_>>(),
        "data\\damagetypes.xml",
        TableId::DamageTypes,
        "damage type",
        &mut diagnostics,
    );

    // ── Object cross-references ─────────────────────────────────────
    for obj in &world.database.objects {
        check_ref_loaded(
            &obj.visual,
            &world.visuals,
            "data\\objects.xml",
            TableId::Objects,
            &obj.name,
            "Visual",
            "visual",
            Severity::Warning,
            &mut diagnostics,
        );
        check_ref_loaded(
            &obj.tactics,
            &world.tactics,
            "data\\objects.xml",
            TableId::Objects,
            &obj.name,
            "Tactics",
            "tactics",
            Severity::Warning,
            &mut diagnostics,
        );
        check_ref_loaded(
            &obj.physics_info,
            &world.physics,
            "data\\objects.xml",
            TableId::Objects,
            &obj.name,
            "PhysicsInfo",
            "physics",
            Severity::Info,
            &mut diagnostics,
        );
    }

    // ── Squad → object cross-references ─────────────────────────────
    for squad in &world.database.squads {
        if let Some(ref units) = squad.units {
            for unit in &units.entries {
                check_ref_exists(
                    &unit.proto_object,
                    &object_names,
                    "data\\squads.xml",
                    TableId::Squads,
                    &squad.name,
                    "Units",
                    &format!(
                        "squad '{}' references object '{}' which does not exist",
                        squad.name, unit.proto_object
                    ),
                    Severity::Warning,
                    &mut diagnostics,
                );
            }
        }
    }

    // ── Leader cross-references ─────────────────────────────────────
    for leader in &world.database.leaders {
        // Leader → Civ
        if let Some(civ) = leader.civ.as_deref() {
            check_ref_exists(
                civ,
                &civ_names,
                "data\\leaders.xml",
                TableId::Leaders,
                &leader.name,
                "Civ",
                &format!(
                    "leader '{}' references civ '{}' which does not exist",
                    leader.name, civ
                ),
                Severity::Warning,
                &mut diagnostics,
            );
        }
        // Leader → Tech
        if let Some(tech) = leader.tech.as_deref() {
            check_ref_exists(
                tech,
                &tech_names,
                "data\\leaders.xml",
                TableId::Leaders,
                &leader.name,
                "Tech",
                &format!(
                    "leader '{}' references tech '{}' which does not exist",
                    leader.name, tech
                ),
                Severity::Warning,
                &mut diagnostics,
            );
        }
        // Leader → StartingUnit (proto-object)
        if let Some(su) = leader.starting_unit.as_ref() {
            check_ref_exists(
                &su.proto_object,
                &object_names,
                "data\\leaders.xml",
                TableId::Leaders,
                &leader.name,
                "StartingUnit",
                &format!(
                    "leader '{}' starting unit '{}' does not exist in objects",
                    leader.name, su.proto_object
                ),
                Severity::Warning,
                &mut diagnostics,
            );
            // StartingUnit → BuildOther (proto-object)
            if let Some(bo) = su.build_other.as_deref() {
                check_ref_exists(
                    bo,
                    &object_names,
                    "data\\leaders.xml",
                    TableId::Leaders,
                    &leader.name,
                    "StartingUnit.BuildOther",
                    &format!(
                        "leader '{}' starting unit BuildOther '{}' does not exist in objects",
                        leader.name, bo
                    ),
                    Severity::Warning,
                    &mut diagnostics,
                );
            }
        }
        // Leader → StartingSquads (proto-squad)
        for ss in &leader.starting_squads {
            check_ref_exists(
                &ss.proto_squad,
                &squad_names,
                "data\\leaders.xml",
                TableId::Leaders,
                &leader.name,
                "StartingSquad",
                &format!(
                    "leader '{}' starting squad '{}' does not exist in squads",
                    leader.name, ss.proto_squad
                ),
                Severity::Warning,
                &mut diagnostics,
            );
        }
    }

    // ── Civ cross-references ────────────────────────────────────────
    for civ in &world.database.civs {
        if let Some(ct) = civ.civ_tech.as_deref() {
            check_ref_exists(
                ct,
                &tech_names,
                "data\\civs.xml",
                TableId::Civs,
                &civ.name,
                "CivTech",
                &format!(
                    "civ '{}' references tech '{}' which does not exist",
                    civ.name, ct
                ),
                Severity::Warning,
                &mut diagnostics,
            );
        }
        // Object references from civs
        for (field, val) in [
            ("CommandAckObject", &civ.command_ack_object),
            ("RallyPointObject", &civ.rally_point_object),
            ("LocalRallyPointObject", &civ.local_rally_point_object),
            ("Transport", &civ.transport),
            ("TransportTrigger", &civ.transport_trigger),
        ] {
            if let Some(name) = val {
                check_ref_exists(
                    name,
                    &object_names,
                    "data\\civs.xml",
                    TableId::Civs,
                    &civ.name,
                    field,
                    &format!(
                        "civ '{}' field {} references object '{}' which does not exist",
                        civ.name, field, name
                    ),
                    Severity::Info,
                    &mut diagnostics,
                );
            }
        }
    }

    // ── Tech prereq cross-references ────────────────────────────────
    for tech in &world.database.techs {
        if let Some(ref prereqs) = tech.prereqs {
            for entry in &prereqs.entries {
                check_ref_exists(
                    &entry.tech,
                    &tech_names,
                    "data\\techs.xml",
                    TableId::Techs,
                    &tech.name,
                    "Prereqs",
                    &format!(
                        "tech '{}' prereq '{}' does not exist in techs",
                        tech.name, entry.tech
                    ),
                    Severity::Warning,
                    &mut diagnostics,
                );
            }
        }
    }

    // ── Tactics weapon cross-references ─────────────────────────────
    for (tac_name, tac_data) in &world.tactics {
        for weapon in &tac_data.weapons {
            // Weapon → WeaponType
            if let Some(wt) = weapon.weapon_type.as_deref()
                && !wt.is_empty()
                && !weapon_type_names.contains(wt)
            {
                diagnostics.push(Diagnostic {
                    path: format!("data\\tactics\\{}.xml", tac_name),
                    location: Location::Field {
                        table: TableId::Tactics,
                        name: tac_name.clone(),
                        field: format!("Weapon.{}.WeaponType", weapon.name),
                    },
                    severity: Severity::Warning,
                    code: DiagnosticCode::MissingReference,
                    message: format!(
                        "tactics '{}' weapon '{}' references weapon type '{}' which does not exist",
                        tac_name, weapon.name, wt
                    ),
                });
            }
            // Weapon → Projectile (proto-object)
            if let Some(proj) = weapon.projectile.as_deref()
                && !proj.is_empty()
                && !object_names.contains(proj)
            {
                diagnostics.push(Diagnostic {
                    path: format!("data\\tactics\\{}.xml", tac_name),
                    location: Location::Field {
                        table: TableId::Tactics,
                        name: tac_name.clone(),
                        field: format!("Weapon.{}.Projectile", weapon.name),
                    },
                    severity: Severity::Info,
                    code: DiagnosticCode::MissingReference,
                    message: format!(
                        "tactics '{}' weapon '{}' references projectile '{}' which does not exist in objects",
                        tac_name, weapon.name, proj
                    ),
                });
            }
        }
    }

    // ── WeaponType → DamageType cross-references ────────────────────
    let damage_type_names: std::collections::HashSet<&str> = world
        .database
        .damage_types
        .iter()
        .map(|d| d.name.as_str())
        .collect();

    for wt in &world.database.weapon_types {
        for dm in &wt.damage_modifiers {
            if !dm.damage_type.is_empty() && !damage_type_names.contains(dm.damage_type.as_str()) {
                diagnostics.push(Diagnostic {
                    path: "data\\weapontypes.xml".into(),
                    location: Location::Field {
                        table: TableId::WeaponTypes,
                        name: wt.name.clone(),
                        field: "DamageModifier".into(),
                    },
                    severity: Severity::Warning,
                    code: DiagnosticCode::MissingReference,
                    message: format!(
                        "weapon type '{}' references damage type '{}' which does not exist",
                        wt.name, dm.damage_type
                    ),
                });
            }
        }
    }

    DiagnosticReport {
        diagnostics,
        elapsed: start.elapsed(),
    }
}

/// Validate that all manifest-referenced binary assets exist on disk.
///
/// This supplements [`validate_world`] with file-existence checks for
/// models (`.ugx`), animations (`.uax`), textures (`.ddx`), and
/// damage models referenced by visuals.
pub fn validate_world_assets(
    world: &super::World,
    src: &crate::source::AssetSource<impl assets::FileProvider>,
) -> DiagnosticReport {
    let start = std::time::Instant::now();
    let mut diagnostics = Vec::new();

    // Models
    for path in &world.manifest.model_refs {
        if !src.exists(path) {
            diagnostics.push(Diagnostic {
                path: path.clone(),
                location: Location::Table(TableId::Visuals),
                severity: Severity::Warning,
                code: DiagnosticCode::MissingAsset,
                message: format!("model '{}' referenced by visuals not found on disk", path),
            });
        }
    }
    // Damage models
    for path in &world.manifest.damage_model_refs {
        if !src.exists(path) {
            diagnostics.push(Diagnostic {
                path: path.clone(),
                location: Location::Table(TableId::Visuals),
                severity: Severity::Info,
                code: DiagnosticCode::MissingAsset,
                message: format!(
                    "damage model '{}' referenced by visuals not found on disk",
                    path
                ),
            });
        }
    }
    // Animations
    for path in &world.manifest.anim_refs {
        if !src.exists(path) {
            diagnostics.push(Diagnostic {
                path: path.clone(),
                location: Location::Table(TableId::Visuals),
                severity: Severity::Info,
                code: DiagnosticCode::MissingAsset,
                message: format!(
                    "animation '{}' referenced by visuals not found on disk",
                    path
                ),
            });
        }
    }
    // Textures
    for path in &world.manifest.texture_refs {
        if !src.exists(path) {
            diagnostics.push(Diagnostic {
                path: path.clone(),
                location: Location::Table(TableId::Visuals),
                severity: Severity::Info,
                code: DiagnosticCode::MissingAsset,
                message: format!("texture '{}' not found on disk", path),
            });
        }
    }

    DiagnosticReport {
        diagnostics,
        elapsed: start.elapsed(),
    }
}

// ── Helper functions ─────────────────────────────────────────────────

/// Check for duplicate names in a list and emit diagnostics.
fn check_duplicates(
    names: &[&str],
    path: &str,
    table: TableId,
    label: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = std::collections::HashSet::new();
    for name in names {
        if !name.is_empty() && !seen.insert(*name) {
            diagnostics.push(Diagnostic {
                path: path.into(),
                location: Location::Entry {
                    table,
                    name: (*name).to_string(),
                },
                severity: Severity::Warning,
                code: DiagnosticCode::DuplicateName,
                message: format!("duplicate {} name '{}'", label, name),
            });
        }
    }
}

/// Check that an `Option<String>` reference points to a loaded HashMap entry.
#[allow(clippy::too_many_arguments)]
fn check_ref_loaded<V>(
    reference: &Option<String>,
    map: &std::collections::HashMap<String, V>,
    path: &str,
    table: TableId,
    entry_name: &str,
    field: &str,
    asset_kind: &str,
    severity: Severity,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(name) = reference
        && !name.is_empty()
        && !map.contains_key(name.as_str())
    {
        diagnostics.push(Diagnostic {
            path: path.into(),
            location: Location::Field {
                table,
                name: entry_name.to_string(),
                field: field.to_string(),
            },
            severity,
            code: DiagnosticCode::MissingReference,
            message: format!(
                "'{}' references {} '{}' which was not loaded",
                entry_name, asset_kind, name
            ),
        });
    }
}

/// Check that a string value exists in a lookup set.
#[allow(clippy::too_many_arguments)]
fn check_ref_exists(
    value: &str,
    lookup: &std::collections::HashSet<&str>,
    path: &str,
    table: TableId,
    entry_name: &str,
    field: &str,
    message: &str,
    severity: Severity,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !value.is_empty() && !lookup.contains(value) {
        diagnostics.push(Diagnostic {
            path: path.into(),
            location: Location::Field {
                table,
                name: entry_name.to_string(),
                field: field.to_string(),
            },
            severity,
            code: DiagnosticCode::MissingReference,
            message: message.to_string(),
        });
    }
}
