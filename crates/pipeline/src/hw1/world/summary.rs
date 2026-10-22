//! Text summary of a loaded `World`.

impl super::World {
    /// Print a summary of the loaded world to stdout.
    pub fn print_summary(&self) {
        println!("=== HW1 World Summary ===\n");
        println!("Database:");
        println!("  Objects:       {}", self.database.objects.len());
        println!("  Squads:        {}", self.database.squads.len());
        println!("  Techs:         {}", self.database.techs.len());
        println!("  Abilities:     {}", self.database.abilities.len());
        println!("  Powers:        {}", self.database.powers.len());
        println!("  Civs:          {}", self.database.civs.len());
        println!("  Leaders:       {}", self.database.leaders.len());
        println!("  Weapon Types:  {}", self.database.weapon_types.len());
        println!("  Damage Types:  {}", self.database.damage_types.len());
        println!(
            "  Game Data:     {}",
            if self.database.game_data.is_some() {
                "loaded"
            } else {
                "missing"
            }
        );
        println!();
        println!("Asset Resolution:");
        println!(
            "  Visuals:  {} / {} ({} failed)",
            self.stats.visuals_resolved,
            self.stats.objects_with_visual,
            self.stats.visuals_failed.len()
        );
        println!(
            "  Tactics:  {} / {} ({} failed)",
            self.stats.tactics_resolved,
            self.stats.objects_with_tactics,
            self.stats.tactics_failed.len()
        );
        println!(
            "  Physics:  {} / {} ({} failed)",
            self.stats.physics_resolved,
            self.stats.objects_with_physics,
            self.stats.physics_failed.len()
        );
        println!("  Blueprints: {}", self.stats.blueprints_resolved);
        println!("  Shapes:     {}", self.stats.shapes_resolved);
        println!();
        println!("Binary Asset References:");
        println!(
            "  Model refs:   {} unique .ugx",
            self.manifest.model_refs.len()
        );
        println!(
            "  Anim refs:    {} unique .uax",
            self.manifest.anim_refs.len()
        );
        println!("  Damage refs:  {}", self.manifest.damage_model_refs.len());
        println!(
            "  Texture refs: {} unique .ddx",
            self.manifest.texture_refs.len()
        );
        println!();
        println!("Preload Lists:");
        println!(
            "  VIS preload:  {} entries",
            self.manifest.preload_vis_refs.len()
        );
        println!(
            "  TFX preload:  {} entries",
            self.manifest.preload_tfx_refs.len()
        );
        println!(
            "  PFX preload:  {} entries",
            self.manifest.preload_pfx_refs.len()
        );
        println!();
        println!("Scenario Refs:");
        println!("  Lightsets:    {} refs", self.manifest.lightset_refs.len());
        println!(
            "  Cinematics:   {} refs",
            self.manifest.cinematic_refs.len()
        );
        println!(
            "  Talking Heads:{} refs",
            self.manifest.talking_head_refs.len()
        );
        println!("  Terrain:      {} refs", self.manifest.terrain_refs.len());
        println!(
            "  Sound Banks:  {} refs",
            self.manifest.sound_bank_refs.len()
        );
        if let Some(sky) = &self.manifest.sky_ref {
            println!("  Sky:          {sky}");
        }
        if let Some(env) = &self.manifest.terrain_env_ref {
            println!("  TerrainEnv:   {env}");
        }
        if let Some(mm) = &self.manifest.minimap_ref {
            println!("  Minimap:      {mm}");
        }
        println!();
        println!(
            "Scenarios:     {} descriptors",
            self.scenario_list.scenarios.len()
        );
        if let Some(desc) = &self.scenario {
            println!("Active:        {}", desc.name());
        }
        if let Some(scn) = &self.scenario_data {
            println!(
                "  SCN objects:  {} placed, {} players, {} positions",
                scn.objects().len(),
                scn.players().len(),
                scn.positions().len()
            );
        }
        if self.terrain_data.is_some() || self.terrain_textures.is_some() {
            println!(
                "  XTD loaded:   {}   XTT loaded: {}",
                self.terrain_data.is_some(),
                self.terrain_textures.is_some()
            );
        }
        if self.lightset.is_some() {
            println!("  Lightset:     loaded");
        }
        if let Some(st) = &self.strings {
            println!(
                "\nString Table:  {} strings ({})",
                st.len(),
                st.language_name,
            );
        }
    }
}
