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
    Entry {
        /// Table that holds the entry.
        table: TableId,
        /// Name of the entry.
        name: String,
    },
    /// A specific field on a named entry.
    Field {
        /// Table that holds the entry.
        table: TableId,
        /// Name of the entry.
        name: String,
        /// Name of the field, as in the XML (for example `Visual`).
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
    /// All diagnostics, in the order of the checks.
    pub diagnostics: Vec<Diagnostic>,
    /// Time that the validation pass took.
    pub elapsed: std::time::Duration,
}

impl DiagnosticReport {
    /// Return the number of diagnostics with `Severity::Error`.
    #[must_use]
    pub fn errors(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Return the number of diagnostics with `Severity::Warning`.
    #[must_use]
    pub fn warnings(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// Return the number of diagnostics with `Severity::Info`.
    #[must_use]
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
    let names = NameSets {
        objects: world
            .database
            .objects
            .iter()
            .map(|o| o.name.as_str())
            .collect(),
        squads: world
            .database
            .squads
            .iter()
            .map(|s| s.name.as_str())
            .collect(),
        techs: world
            .database
            .techs
            .iter()
            .map(|t| t.name.as_str())
            .collect(),
        civs: world
            .database
            .civs
            .iter()
            .map(|c| c.name.as_str())
            .collect(),
        weapon_types: world
            .database
            .weapon_types
            .iter()
            .map(|w| w.name.as_str())
            .collect(),
    };

    check_all_duplicates(world, &mut diagnostics);
    check_object_refs(world, &mut diagnostics);
    check_squad_refs(world, &names, &mut diagnostics);
    check_leader_refs(world, &names, &mut diagnostics);
    check_civ_refs(world, &names, &mut diagnostics);
    check_tech_refs(world, &names, &mut diagnostics);
    check_tactics_refs(world, &names, &mut diagnostics);
    check_weapon_type_refs(world, &mut diagnostics);

    DiagnosticReport {
        diagnostics,
        elapsed: start.elapsed(),
    }
}

/// Name lookup sets of the database tables that other tables refer to.
struct NameSets<'a> {
    /// Proto-object names from `data\objects.xml`.
    objects: std::collections::HashSet<&'a str>,
    /// Proto-squad names from `data\squads.xml`.
    squads: std::collections::HashSet<&'a str>,
    /// Tech names from `data\techs.xml`.
    techs: std::collections::HashSet<&'a str>,
    /// Civ names from `data\civs.xml`.
    civs: std::collections::HashSet<&'a str>,
    /// Weapon type names from `data\weapontypes.xml`.
    weapon_types: std::collections::HashSet<&'a str>,
}

/// The location of one reference field, for a diagnostic.
struct RefSite<'a> {
    /// Game path of the file that holds the reference.
    path: &'a str,
    /// Table that holds the reference.
    table: TableId,
    /// Name of the entry that holds the reference.
    entry_name: &'a str,
    /// Name of the field that holds the reference.
    field: &'a str,
    /// Severity of the diagnostic if the reference is broken.
    severity: Severity,
}

/// Check for duplicate names in every database table.
fn check_all_duplicates(world: &super::World, diagnostics: &mut Vec<Diagnostic>) {
    let db = &world.database;
    let tables: [(Vec<&str>, &str, TableId, &str); 9] = [
        (
            db.objects.iter().map(|o| o.name.as_str()).collect(),
            "data\\objects.xml",
            TableId::Objects,
            "object",
        ),
        (
            db.squads.iter().map(|s| s.name.as_str()).collect(),
            "data\\squads.xml",
            TableId::Squads,
            "squad",
        ),
        (
            db.techs.iter().map(|t| t.name.as_str()).collect(),
            "data\\techs.xml",
            TableId::Techs,
            "tech",
        ),
        (
            db.abilities.iter().map(|a| a.name.as_str()).collect(),
            "data\\abilities.xml",
            TableId::Abilities,
            "ability",
        ),
        (
            db.powers.iter().map(|p| p.name.as_str()).collect(),
            "data\\powers.xml",
            TableId::Powers,
            "power",
        ),
        (
            db.leaders.iter().map(|l| l.name.as_str()).collect(),
            "data\\leaders.xml",
            TableId::Leaders,
            "leader",
        ),
        (
            db.civs.iter().map(|c| c.name.as_str()).collect(),
            "data\\civs.xml",
            TableId::Civs,
            "civ",
        ),
        (
            db.weapon_types.iter().map(|w| w.name.as_str()).collect(),
            "data\\weapontypes.xml",
            TableId::WeaponTypes,
            "weapon type",
        ),
        (
            db.damage_types.iter().map(|d| d.name.as_str()).collect(),
            "data\\damagetypes.xml",
            TableId::DamageTypes,
            "damage type",
        ),
    ];
    for (names, path, table, label) in &tables {
        check_duplicates(names, path, *table, label, diagnostics);
    }
}

/// Check the visual, tactics and physics references of each object.
fn check_object_refs(world: &super::World, diagnostics: &mut Vec<Diagnostic>) {
    for obj in &world.database.objects {
        let site = |field, severity| RefSite {
            path: "data\\objects.xml",
            table: TableId::Objects,
            entry_name: &obj.name,
            field,
            severity,
        };
        check_ref_loaded(
            obj.visual.as_deref(),
            &world.visuals,
            &site("Visual", Severity::Warning),
            "visual",
            diagnostics,
        );
        check_ref_loaded(
            obj.tactics.as_deref(),
            &world.tactics,
            &site("Tactics", Severity::Warning),
            "tactics",
            diagnostics,
        );
        check_ref_loaded(
            obj.physics_info.as_deref(),
            &world.physics,
            &site("PhysicsInfo", Severity::Info),
            "physics",
            diagnostics,
        );
    }
}

/// Check that each squad unit entry refers to an existing object.
fn check_squad_refs(world: &super::World, names: &NameSets<'_>, diagnostics: &mut Vec<Diagnostic>) {
    for squad in &world.database.squads {
        if let Some(ref units) = squad.units {
            for unit in &units.entries {
                check_ref_exists(
                    &unit.proto_object,
                    &names.objects,
                    &RefSite {
                        path: "data\\squads.xml",
                        table: TableId::Squads,
                        entry_name: &squad.name,
                        field: "Units",
                        severity: Severity::Warning,
                    },
                    &format!(
                        "squad '{}' references object '{}' which does not exist",
                        squad.name, unit.proto_object
                    ),
                    diagnostics,
                );
            }
        }
    }
}

/// Check the civ, tech, starting unit and starting squad references of each leader.
fn check_leader_refs(
    world: &super::World,
    names: &NameSets<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for leader in &world.database.leaders {
        let site = |field| RefSite {
            path: "data\\leaders.xml",
            table: TableId::Leaders,
            entry_name: &leader.name,
            field,
            severity: Severity::Warning,
        };
        // Leader → Civ
        if let Some(civ) = leader.civ.as_deref() {
            check_ref_exists(
                civ,
                &names.civs,
                &site("Civ"),
                &format!(
                    "leader '{}' references civ '{}' which does not exist",
                    leader.name, civ
                ),
                diagnostics,
            );
        }
        // Leader → Tech
        if let Some(tech) = leader.tech.as_deref() {
            check_ref_exists(
                tech,
                &names.techs,
                &site("Tech"),
                &format!(
                    "leader '{}' references tech '{}' which does not exist",
                    leader.name, tech
                ),
                diagnostics,
            );
        }
        // Leader → StartingUnit (proto-object)
        for su in &leader.starting_units {
            check_ref_exists(
                &su.proto_object,
                &names.objects,
                &site("StartingUnit"),
                &format!(
                    "leader '{}' starting unit '{}' does not exist in objects",
                    leader.name, su.proto_object
                ),
                diagnostics,
            );
            // StartingUnit → BuildOther (proto-object)
            if let Some(bo) = su.build_other.as_deref() {
                check_ref_exists(
                    bo,
                    &names.objects,
                    &site("StartingUnit.BuildOther"),
                    &format!(
                        "leader '{}' starting unit BuildOther '{}' does not exist in objects",
                        leader.name, bo
                    ),
                    diagnostics,
                );
            }
        }
        // Leader → StartingSquads (proto-squad)
        for ss in &leader.starting_squads {
            check_ref_exists(
                &ss.proto_squad,
                &names.squads,
                &site("StartingSquad"),
                &format!(
                    "leader '{}' starting squad '{}' does not exist in squads",
                    leader.name, ss.proto_squad
                ),
                diagnostics,
            );
        }
    }
}

/// Check the tech and object references of each civ.
fn check_civ_refs(world: &super::World, names: &NameSets<'_>, diagnostics: &mut Vec<Diagnostic>) {
    for civ in &world.database.civs {
        if let Some(ct) = civ.civ_tech.as_deref() {
            check_ref_exists(
                ct,
                &names.techs,
                &RefSite {
                    path: "data\\civs.xml",
                    table: TableId::Civs,
                    entry_name: &civ.name,
                    field: "CivTech",
                    severity: Severity::Warning,
                },
                &format!(
                    "civ '{}' references tech '{}' which does not exist",
                    civ.name, ct
                ),
                diagnostics,
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
                    &names.objects,
                    &RefSite {
                        path: "data\\civs.xml",
                        table: TableId::Civs,
                        entry_name: &civ.name,
                        field,
                        severity: Severity::Info,
                    },
                    &format!(
                        "civ '{}' field {} references object '{}' which does not exist",
                        civ.name, field, name
                    ),
                    diagnostics,
                );
            }
        }
    }
}

/// Check that each tech prerequisite refers to an existing tech.
fn check_tech_refs(world: &super::World, names: &NameSets<'_>, diagnostics: &mut Vec<Diagnostic>) {
    for tech in &world.database.techs {
        if let Some(ref prereqs) = tech.prereqs {
            for entry in &prereqs.entries {
                check_ref_exists(
                    &entry.tech,
                    &names.techs,
                    &RefSite {
                        path: "data\\techs.xml",
                        table: TableId::Techs,
                        entry_name: &tech.name,
                        field: "Prereqs",
                        severity: Severity::Warning,
                    },
                    &format!(
                        "tech '{}' prereq '{}' does not exist in techs",
                        tech.name, entry.tech
                    ),
                    diagnostics,
                );
            }
        }
    }
}

/// Check the weapon type and projectile references of each tactics weapon.
fn check_tactics_refs(
    world: &super::World,
    names: &NameSets<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (tac_name, tac_data) in &world.tactics {
        for weapon in &tac_data.weapons {
            // Weapon → WeaponType
            if let Some(wt) = weapon.weapon_type.as_deref()
                && !wt.is_empty()
                && !names.weapon_types.contains(wt)
            {
                diagnostics.push(Diagnostic {
                    path: format!("data\\tactics\\{tac_name}.xml"),
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
                && !names.objects.contains(proj)
            {
                diagnostics.push(Diagnostic {
                    path: format!("data\\tactics\\{tac_name}.xml"),
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
}

/// Check that each weapon type damage modifier refers to an existing damage type.
fn check_weapon_type_refs(world: &super::World, diagnostics: &mut Vec<Diagnostic>) {
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
                message: format!("model '{path}' referenced by visuals not found on disk"),
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
                message: format!("damage model '{path}' referenced by visuals not found on disk"),
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
                message: format!("animation '{path}' referenced by visuals not found on disk"),
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
                message: format!("texture '{path}' not found on disk"),
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
                message: format!("duplicate {label} name '{name}'"),
            });
        }
    }
}

/// Check that an optional reference points to a loaded `HashMap` entry.
fn check_ref_loaded<V>(
    reference: Option<&str>,
    map: &std::collections::HashMap<String, V>,
    site: &RefSite<'_>,
    asset_kind: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(name) = reference
        && !name.is_empty()
        && !map.contains_key(name)
    {
        diagnostics.push(Diagnostic {
            path: site.path.into(),
            location: Location::Field {
                table: site.table,
                name: site.entry_name.to_string(),
                field: site.field.to_string(),
            },
            severity: site.severity,
            code: DiagnosticCode::MissingReference,
            message: format!(
                "'{}' references {asset_kind} '{name}' which was not loaded",
                site.entry_name
            ),
        });
    }
}

/// Check that a string value exists in a lookup set.
fn check_ref_exists(
    value: &str,
    lookup: &std::collections::HashSet<&str>,
    site: &RefSite<'_>,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !value.is_empty() && !lookup.contains(value) {
        diagnostics.push(Diagnostic {
            path: site.path.into(),
            location: Location::Field {
                table: site.table,
                name: site.entry_name.to_string(),
                field: site.field.to_string(),
            },
            severity: site.severity,
            code: DiagnosticCode::MissingReference,
            message: message.to_string(),
        });
    }
}
