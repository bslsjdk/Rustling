//! An entity renderer example: pose a baked model with a walk animation and draw it. The renderer
//! is stateless per entity; its model and animation are shared, so each frame poses a clone.

use mod_api::{
    BlockEntityDrawContext, EntityRenderState, EntityRenderer, KeyframeAnimation, ModelPart,
};

/// Draws one animated model for every entity registered to this renderer.
pub struct AnimatedModelRenderer {
    model: ModelPart,
    animation: KeyframeAnimation,
    sprite: String,
}

impl AnimatedModelRenderer {
    pub fn new(model: ModelPart, animation: KeyframeAnimation, sprite: impl Into<String>) -> Self {
        Self {
            model,
            animation,
            sprite: sprite.into(),
        }
    }
}

impl EntityRenderer for AnimatedModelRenderer {
    fn submit(
        &self,
        entity: &EntityRenderState,
        draw: &mut dyn BlockEntityDrawContext,
    ) -> Result<(), String> {
        let Some(living) = entity.living else {
            return Ok(());
        };
        // The pose starts at the entity, unturned; face the way the body does.
        draw.rotate_about([0.0, 1.0, 0.0], -living.body_rot, [0.0; 3]);
        draw.scale([living.scale * living.age_scale; 3]);
        let mut model = self.model.clone();
        self.animation.apply_walk(
            &mut model,
            living.walk_animation_pos,
            living.walk_animation_speed,
            1.0,
            1.0,
        );
        draw.model_part(&model, &self.sprite, 0xFFFF_FFFF);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::tlm_animation::animation_to_keyframe;
    use crate::game::tlm_geometry::geometry_to_parts;
    use std::sync::Arc;

    const GEO: &str = r#"{"minecraft:geometry":[{"description":{"texture_width":8,"texture_height":8},
        "bones":[{"name":"body","pivot":[0,0,0],"cubes":[
            {"origin":[0,0,0],"size":[2,2,2],"uv":[0,0]}]}]}]}"#;

    const WALK: &str = r#"{"animations":{"walk":{"animation_length":1.0,"loop":true,
        "bones":{"body":{"rotation":{"0.0":[0,0,0],"0.5":[0,90,0],"1.0":[0,0,0]}}}}}}"#;

    #[test]
    fn a_renderer_holds_a_baked_model_and_animation() {
        let model = geometry_to_parts(GEO).unwrap();
        let animation = animation_to_keyframe(WALK, "example", &model).unwrap();
        let renderer = AnimatedModelRenderer::new(model, animation, "example:entity/model");
        let _: Arc<dyn EntityRenderer> = Arc::new(renderer);
    }
}
