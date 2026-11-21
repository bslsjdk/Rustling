//! Bedrock-format animation files (`*.animation.json`) → keyframe animation data.
//!
//! Bedrock animation keyframes are numeric time→value maps for a bone's rotation, position and
//! scale. They become [`Keyframe`]s on the matching [`AnimationTarget`] channel. Rotation is in
//! degrees in the file and radians in the engine, so it is converted while parsing.

use super::keyframe_animation::{
    AnimationChannel, AnimationHolder, AnimationTarget, Interpolation, Keyframe,
};
use crate::Identifier;
use serde_json::Value;

/// Parse one animation file's body into holders. `id_prefix` namespaces each animation id.
pub fn parse_bedrock_animation(id_prefix: &str, text: &str) -> Result<Vec<AnimationHolder>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| format!("animation JSON: {e}"))?;
    let animations = root
        .get("animations")
        .and_then(Value::as_object)
        .ok_or("animation file has no `animations` object")?;
    let mut out = Vec::new();
    for (name, anim) in animations {
        let length = anim
            .get("animation_length")
            .and_then(Value::as_f64)
            .unwrap_or(0.0) as f32;
        let looping = anim
            .get("loop")
            .and_then(Value::as_bool)
            .or_else(|| anim.get("looping").and_then(Value::as_bool))
            .unwrap_or(false);
        let mut bones: Vec<(String, Vec<AnimationChannel>)> = Vec::new();
        if let Some(map) = anim.get("bones").and_then(Value::as_object) {
            for (bone, channels) in map {
                let mut cs = Vec::new();
                for (key, target) in [
                    ("rotation", AnimationTarget::Rotation),
                    ("position", AnimationTarget::Position),
                    ("scale", AnimationTarget::Scale),
                ] {
                    if let Some(kf) = parse_channel(channels.get(key), target) {
                        cs.push(kf);
                    }
                }
                if !cs.is_empty() {
                    bones.push((bone.clone(), cs));
                }
            }
        }
        let id = Identifier::from_parts(id_prefix, name)
            .map_err(|_| format!("invalid animation id {name:?}"))?;
        out.push(AnimationHolder { id, length, looping, bones, types: Vec::new() });
    }
    Ok(out)
}

fn parse_channel(v: Option<&Value>, target: AnimationTarget) -> Option<AnimationChannel> {
    let obj = v?.as_object()?;
    let mut keyframes = Vec::new();
    for (time, val) in obj {
        let t: f32 = match time.parse() {
            Ok(t) => t,
            Err(_) => continue, // an expression in a time key is not a static keyframe
        };
        let arr = val.as_array().or_else(|| val.get("vec").and_then(Value::as_array))?;
        if arr.len() != 3 {
            continue;
        }
        let mut v = std::array::from_fn::<_, 3, _>(|i| arr[i].as_f64().unwrap_or(0.0) as f32);
        if matches!(target, AnimationTarget::Rotation) {
            for x in &mut v {
                *x = x.to_radians();
            }
        }
        keyframes.push(Keyframe { time: t, pre: v, post: v, interpolation: Interpolation::Linear });
    }
    if keyframes.is_empty() {
        return None;
    }
    keyframes.sort_by(|a, b| a.time.total_cmp(&b.time));
    Some(AnimationChannel { target, keyframes })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "format_version": "1.8.0",
      "animations": {
        "walk": {
          "loop": true, "animation_length": 2.0,
          "bones": {
            "body": { "rotation": { "0.0": [0,0,0], "1.0": [90,0,0], "2.0": [0,0,0] } },
            "head": { "position": { "0.0": [0,1,0], "1.0": [0,2,0] } }
          }
        }
      }
    }"#;

    #[test]
    fn parses_bedrock_animation() {
        let holders = parse_bedrock_animation("ns", SAMPLE).unwrap();
        assert_eq!(holders.len(), 1);
        let a = &holders[0];
        assert!(a.looping);
        assert_eq!(a.length, 2.0);
        assert_eq!(a.bones.len(), 2);
        let body = a.bones.iter().find(|(n, _)| n == "body").unwrap();
        let rot = &body.1[0];
        assert_eq!(rot.target, AnimationTarget::Rotation);
        assert_eq!(rot.keyframes.len(), 3);
        assert!((rot.keyframes[1].post[0] - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }
}
