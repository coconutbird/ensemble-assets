use alloc::format;

use xmb::Document;

use super::{
    BlendMode, LayerBlendMode, ParticleEffect, ParticleEmitterKind, ParticleEmitterShapeKind,
    ParticleType, parse,
};

fn effect_from_emitter(emitter: &str) -> ParticleEffect {
    let document = Document::from_xml(&format!(
        "<ParticleEffect><ParticleEmitter>{emitter}</ParticleEmitter></ParticleEffect>"
    ))
    .expect("test XML should decode");
    parse(&document).expect("test effect should parse")
}

#[test]
fn decoded_xmb_maps_every_authored_choice_directly() {
    let document = Document::from_xml(
        r#"<ParticleEffect Name="effects/test.pfx">
            <ParticleEmitter Name="beam" Active="true">
                <EmitterData>
                    <MaxParticles>12</MaxParticles>
                    <ParticleType>eTrailCross</ParticleType>
                    <BlendMode>eSubtractive</BlendMode>
                    <SoftParticles>true</SoftParticles>
                    <SortParticles>true</SortParticles>
                    <LightBuffer>true</LightBuffer>
                    <LightBufferIntensityScale>2</LightBufferIntensityScale>
                    <SoftParticleFadeRange>3</SoftParticleFadeRange>
                </EmitterData>
                <TextureData>
                    <DiffuseLayer1To2BlendMode>eBlendAlpha</DiffuseLayer1To2BlendMode>
                    <DiffuseLayer2To3BlendMode>eBlendMultiply</DiffuseLayer2To3BlendMode>
                    <Diffuse><Textures><Stage><file>effects/test.tga</file><weight>2</weight></Stage></Textures></Diffuse>
                </TextureData>
            </ParticleEmitter>
        </ParticleEffect>"#,
    )
    .unwrap();
    let effect = parse(&document).unwrap();
    let emitter = &effect.emitters[0];
    assert_eq!(
        emitter.kind,
        ParticleEmitterKind::Render(ParticleType::TrailCross)
    );
    assert_eq!(emitter.material.blend, BlendMode::Subtractive);
    assert_eq!(emitter.material.layer_1_to_2, LayerBlendMode::Alpha);
    assert!(emitter.material.soft_particles);
    assert_eq!(
        emitter.material.soft_fade_scale.to_bits(),
        2.0_f32.to_bits()
    );
    assert!(!emitter.material.light_volume);
    assert!(emitter.sort_particles);
    assert_eq!(emitter.max_particles, 12);
    assert_eq!(
        emitter.material.diffuse[0].stages[0].path,
        "effects/test.tga"
    );
}

#[test]
fn beam_alignment_selects_the_retail_geometry_shader_family() {
    for (alignment, expected) in [
        ("eBeamAlignToCamera", ParticleType::Beam),
        ("eBeamAlignVertical", ParticleType::BeamVertical),
        ("eBeamAlignHorizontal", ParticleType::BeamHorizontal),
    ] {
        let document = Document::from_xml(&format!(
            r"<ParticleEffect><ParticleEmitter><EmitterData>
                <ParticleType>eBeam</ParticleType>
                <BeamAlignmentType>{alignment}</BeamAlignmentType>
            </EmitterData></ParticleEmitter></ParticleEffect>",
        ))
        .unwrap();
        let effect = parse(&document).unwrap();
        assert_eq!(
            effect.emitters[0].kind,
            ParticleEmitterKind::Render(expected)
        );
    }
}

#[test]
fn corner_colors_and_implicit_light_buffer_match_the_retail_loader() {
    let document = Document::from_xml(
        r#"<ParticleEffect>
            <ParticleEmitter Name="alpha">
                <EmitterData><BlendMode>eAlphaBlend</BlendMode></EmitterData>
                <ColorData>
                    <ColorVertex1>-1</ColorVertex1>
                    <ColorVertex2>-65536</ColorVertex2>
                    <ColorVertex3>-16711936</ColorVertex3>
                    <ColorVertex4>-16776961</ColorVertex4>
                </ColorData>
            </ParticleEmitter>
            <ParticleEmitter Name="additive">
                <EmitterData><BlendMode>eAdditive</BlendMode></EmitterData>
            </ParticleEmitter>
        </ParticleEffect>"#,
    )
    .unwrap();
    let effect = parse(&document).unwrap();
    assert_eq!(
        effect.emitters[0].material.corner_colors,
        [
            [1.0, 1.0, 1.0, 1.0],
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ]
    );
    assert!(effect.emitters[0].material.light_volume);
    assert!(!effect.emitters[1].material.light_volume);
}

#[test]
fn retail_defaults_and_trail_lifetime_floor_are_preserved() {
    let effect = effect_from_emitter(
        r"<EmitterData>
            <ParticleType>eTrail</ParticleType>
            <ParticleLife>0.01</ParticleLife>
        </EmitterData>",
    );
    let emitter = &effect.emitters[0];
    assert_eq!(emitter.max_particles, 1000);
    assert_eq!(
        emitter.runtime.timing.update_radius.to_bits(),
        10.0_f32.to_bits()
    );
    assert_eq!(
        emitter.runtime.timing.emission_rate.value.to_bits(),
        100.0_f32.to_bits()
    );
    assert_eq!(
        emitter.runtime.timing.particle_life.value.to_bits(),
        0.11_f32.to_bits()
    );
    assert_eq!(
        emitter.runtime.timing.terrain_y_offset.to_bits(),
        0.125_f32.to_bits()
    );
}

#[test]
fn shape_fields_use_the_retail_axis_prefix_layout() {
    let effect = effect_from_emitter(
        r"<EmitterData/><ShapeData>
            <ShapeType>eCylinder</ShapeType>
            <XSize>1</XSize><YSize>2</YSize><ZSize>3</ZSize>
            <XPosOffset>4</XPosOffset><YPosOffset>5</YPosOffset><ZPosOffset>6</ZPosOffset>
            <TrajectoryInnerAngle>50</TrajectoryInnerAngle>
            <TrajectoryOuterAngle>20</TrajectoryOuterAngle>
        </ShapeData>",
    );
    let shape = &effect.emitters[0].runtime.shape;
    assert_eq!(shape.kind, ParticleEmitterShapeKind::Cylinder);
    assert_eq!(
        shape.size.map(f32::to_bits),
        [1.0, 2.0, 3.0].map(f32::to_bits)
    );
    assert_eq!(
        shape.offset.map(f32::to_bits),
        [4.0, 5.0, 6.0].map(f32::to_bits)
    );
    assert_eq!(shape.trajectory_inner_angle.to_bits(), 20.0_f32.to_bits());
    assert_eq!(shape.trajectory_outer_angle.to_bits(), 50.0_f32.to_bits());
}

#[test]
fn beam_tangents_keep_all_authored_axes_instead_of_the_retail_loader_bug() {
    let effect = effect_from_emitter(
        r"<EmitterData>
            <BeamTangent1X>1</BeamTangent1X><BeamTangent1Y>2</BeamTangent1Y>
            <BeamTangent1Z>3</BeamTangent1Z><BeamTangent2X>4</BeamTangent2X>
            <BeamTangent2Y>5</BeamTangent2Y><BeamTangent2Z>6</BeamTangent2Z>
        </EmitterData>",
    );
    let timing = &effect.emitters[0].runtime.timing;
    assert_eq!(
        timing.beam_tangent_1.map(f32::to_bits),
        [1.0, 2.0, 3.0].map(f32::to_bits)
    );
    assert_eq!(
        timing.beam_tangent_2.map(f32::to_bits),
        [4.0, 5.0, 6.0].map(f32::to_bits)
    );
}
