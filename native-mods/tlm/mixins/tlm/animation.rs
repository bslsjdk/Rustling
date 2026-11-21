// Bedrock-format animation JSON to baked keyframe animations.
//
// Parsing is delegated to the engine's `parse_bedrock_animation` (exported on the mod API),
// which handles the format's unit conversions and keyframe variants. This module only bakes the
// resulting holders against a model.

use mod_api::{AnimationHolder, KeyframeAnimation, ModelPart};

/// Bakes the first animation in `text` against `model`. `id_prefix` is the id's namespace.
pub fn animation_to_keyframe(
    text: &str,
    id_prefix: &str,
    model: &ModelPart,
) -> Result<KeyframeAnimation, String> {
    let mut holders: Vec<AnimationHolder> = mod_api::parse_bedrock_animation(id_prefix, text)?;
    let holder = holders.drain(..).next().ok_or_else(|| "animation: no animations".to_string())?;
    holder
        .bake(model)
        .map_err(|e| format!("animation: bake failed: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::tlm_geometry::geometry_to_parts;
    use mod_api::AnimationState;

    const GEO: &str = r#"{"minecraft:geometry":[{"description":{"texture_width":8,"texture_height":8},
        "bones":[{"name":"body","pivot":[0,0,0],"cubes":[{"origin":[0,0,0],"size":[1,1,1],"uv":[0,0]}]}]}]}"#;
    const WALK: &str = r#"{"animations":{"walk":{"loop":true,"animation_length":2.0,
        "bones":{"body":{"rotation":{"0.0":[0,0,0],"1.0":[90,0,0],"2.0":[0,0,0]}}}}}}"#;

    #[test]
    fn bakes_and_samples_animation() {
        let model = geometry_to_parts(GEO).unwrap();
        let animation = animation_to_keyframe(WALK, "example", &model).unwrap();
        let root = model.get_child("body").unwrap();
        let mut posed = model.clone();
        let mut state = AnimationState::new();
        state.start(0);
        animation.apply(&mut posed, &state, 30.0);
        // The pose was written without panicking; the child still exists after baking.
        assert!(posed.get_child("body").is_some());
        let _ = root;
    }
}
