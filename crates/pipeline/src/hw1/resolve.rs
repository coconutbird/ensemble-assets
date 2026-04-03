//! Per-object asset resolution — visual, tactics, and physics chains.
//!
//! Each proto-object in the database can reference a visual (→ models,
//! animations, damage models), tactics, and physics (→ blueprint → shape).
//! This module contains the types and helpers for walking those chains.

use crate::source::AssetSource;

// ── Types ───────────────────────────────────────────────────────────────

/// A fully resolved physics chain for a single object.
#[derive(Debug, Clone, Default)]
pub struct PhysicsChain {
    /// The parsed `.physics.xmb` data.
    pub physics: database::hw1::Physics,
    /// The parsed `.blueprint.xmb` data (if the physics references one).
    pub blueprint: Option<database::hw1::Blueprint>,
    /// The parsed `.shp.xmb` data (if the blueprint references one).
    pub shape: Option<database::hw1::Shape>,
}

/// All file paths associated with a single proto-object.
///
/// Built eagerly during [`super::World::load`] by walking the visual,
/// tactics, and physics chains. Lets you answer "give me everything
/// related to the Gorgon" without touching the archives again.
#[derive(Debug, Clone, Default)]
pub struct ObjectAssets {
    /// Object name (same key as the `HashMap`).
    pub name: String,
    /// Object class from the database (e.g. `"Unit"`, `"Building"`, `"Projectile"`).
    pub object_class: Option<String>,
    /// Object type tags (e.g. `["Military", "CovVehicle"]`).
    pub object_types: Vec<String>,
    /// Path to the `.vis` / `.vis.xmb` file.
    pub visual: Option<String>,
    /// Path to the `.tactics` / `.tactics.xmb` file.
    pub tactics: Option<String>,
    /// Path to the `.physics` / `.physics.xmb` file.
    pub physics: Option<String>,
    /// Path to the `.blueprint` / `.blueprint.xmb` file.
    pub blueprint: Option<String>,
    /// Path to the `.shp` / `.shp.xmb` file.
    pub shape: Option<String>,
    /// Model files (.ugx) referenced by the visual.
    pub models: Vec<String>,
    /// Animation files (.uax) referenced by the visual.
    pub anims: Vec<String>,
    /// Damage model files (.ugx) referenced by the visual.
    pub damage_models: Vec<String>,
}

/// Statistics from the world loading process.
#[derive(Debug, Clone, Default)]
pub struct LoadStats {
    pub objects_total: usize,
    pub objects_with_visual: usize,
    pub objects_with_tactics: usize,
    pub objects_with_physics: usize,
    pub visuals_resolved: usize,
    pub visuals_failed: Vec<String>,
    pub tactics_resolved: usize,
    pub tactics_failed: Vec<String>,
    pub physics_resolved: usize,
    pub physics_failed: Vec<String>,
    pub blueprints_resolved: usize,
    pub shapes_resolved: usize,
}

impl ObjectAssets {
    /// All file paths referenced by this object, in no particular order.
    pub fn all_files(&self) -> Vec<&str> {
        let mut files = Vec::new();
        if let Some(v) = &self.visual {
            files.push(v.as_str());
        }
        if let Some(t) = &self.tactics {
            files.push(t.as_str());
        }
        if let Some(p) = &self.physics {
            files.push(p.as_str());
        }
        if let Some(b) = &self.blueprint {
            files.push(b.as_str());
        }
        if let Some(s) = &self.shape {
            files.push(s.as_str());
        }
        for m in &self.models {
            files.push(m.as_str());
        }
        for a in &self.anims {
            files.push(a.as_str());
        }
        for d in &self.damage_models {
            files.push(d.as_str());
        }
        files
    }
}

// ── Physics chain resolution ────────────────────────────────────────

/// Resolve the blueprint → shape chain from a physics entry.
pub(crate) fn resolve_physics_chain(
    src: &mut AssetSource<impl assets::FileProvider>,
    chain: &mut PhysicsChain,
    stats: &mut LoadStats,
) {
    if let Some(bp_ref) = &chain.physics.blueprint {
        let bp_base = format!("physics\\{}.blueprint", bp_ref);
        if let Some(bp_doc) = src.read_xmb(&bp_base)
            && let Ok(bp) = database::hw1::physics::parse_blueprint(&bp_doc)
        {
            stats.blueprints_resolved += 1;

            // Shape chain
            if let Some(shp_ref) = &bp.shape {
                let shp_base = format!("physics\\{}.shp", shp_ref);
                if let Some(shp_doc) = src.read_xmb(&shp_base)
                    && let Ok(shp) = database::hw1::physics::parse_shape(&shp_doc)
                {
                    stats.shapes_resolved += 1;
                    chain.shape = Some(shp);
                }
            }

            chain.blueprint = Some(bp);
        }
    }
}
