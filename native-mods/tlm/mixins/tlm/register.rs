// The register_entity_renderers hook: wire a baked model into the game's renderer registry.
// The type id and sprite are placeholders a real content mod replaces with its own.
fn tlm_register_renderers(
    context: &mod_api::RegistrationContext,
    renderers: &mut dyn mod_api::EntityRenderers,
) {
    let _ = context;
    let mesh = mod_api::MeshDefinition::new();
    let model = mod_api::LayerDefinition::create(mesh, 1, 1).bake_root();
    let Ok(animation) = crate::game::tlm_animation::animation_to_keyframe(
        r#"{"animations":{"idle":{"bones":{}}}}"#,
        "example",
        &model,
    ) else {
        return;
    };
    renderers.register(
        "example:model".into(),
        std::sync::Arc::new(crate::game::tlm_render::AnimatedModelRenderer::new(
            model,
            animation,
            "example:entity/model",
        )),
    );
}
