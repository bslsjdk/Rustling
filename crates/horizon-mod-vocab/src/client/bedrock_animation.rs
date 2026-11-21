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
            .and_then(finite)
            .unwrap_or(0.0);
        let looping = anim
            .get("loop")
            .and_then(Value::as_bool)
            .or_else(|| anim.get("looping").and_then(Value::as_bool))
            .unwrap_or(false)
            // A zero-length loop would divide by zero (`seconds %= length`) when played.
            && length > 0.0;
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

/// Bedrock values to the engine's `offsetPos`/`offsetRotation`/`offsetScale` units, matching the
/// standard animation converter (`mod_animations.rs`): position y is negated, rotation is degrees
/// to radians, and scale is an offset from 1.
fn convert(v: [f32; 3], target: AnimationTarget) -> [f32; 3] {
    match target {
        AnimationTarget::Position => [v[0], -v[1], v[2]],
        AnimationTarget::Rotation => v.map(f32::to_radians),
        AnimationTarget::Scale => v.map(|d| d - 1.0),
        AnimationTarget::Registered(_) => v,
    }
}

fn finite(x: f64) -> Option<f32> {
    let f = x as f32;
    f.is_finite().then_some(f)
}

/// One value at a time key: a bare `[x,y,z]`, `{"vec":[x,y,z]}`, or a keyframe object
/// `{"pre":[...],"post":[...],"lerp_mode":"catmullrom"}`. Returns `(pre, post, interpolation)`.
fn value_of(val: &Value, target: AnimationTarget) -> Option<([f32; 3], [f32; 3], Interpolation)> {
    let vec = |v: &Value| -> Option<[f32; 3]> {
        // Bedrock allows a scalar for a channel, meaning the same value on all axes (common for
        // `scale`). JSON stores it as a string or a number.
        let scalar = |s: &str| s.parse::<f64>().ok().and_then(finite).map(|x| [x, x, x]);
        if let Some(s) = v.as_str() {
            return scalar(s);
        }
        if let Some(n) = v.as_f64() {
            return finite(n).map(|x| [x, x, x]);
        }
        let a = v.as_array()?;
        if a.len() != 3 {
            return None;
        }
        // An expression vector can't be evaluated here: skip the frame rather than pass off
        // unevaluable components as zero.
        let x = a[0].as_f64().and_then(finite)?;
        let y = a[1].as_f64().and_then(finite)?;
        let z = a[2].as_f64().and_then(finite)?;
        Some([x, y, z])
    };
    let interp = |obj: &serde_json::Map<String, Value>| match obj.get("lerp_mode").and_then(Value::as_str) {
        Some("catmullrom") => Interpolation::CatmullRom,
        _ => Interpolation::Linear,
    };
    // A bare scalar (string or number) as the whole channel value, e.g. `"scale": "0.5"`.
    if val.is_string() || val.is_number() {
        let v = convert(vec(val)?, target);
        return Some((v, v, Interpolation::Linear));
    }
    if val.is_array() {
        let v = convert(vec(val)?, target);
        return Some((v, v, Interpolation::Linear));
    }
    if let Some(obj) = val.as_object() {
        if let Some(v) = obj.get("vec").and_then(vec) {
            let v = convert(v, target);
            return Some((v, v, interp(obj)));
        }
        // A keyframe may give only `post` (the common Bedrock form): `pre` then equals `post`.
        let post = obj.get("post").and_then(vec)?;
        let pre = obj.get("pre").and_then(vec).unwrap_or(post);
        return Some((convert(pre, target), convert(post, target), interp(obj)));
    }
    None
}

fn parse_channel(v: Option<&Value>, target: AnimationTarget) -> Option<AnimationChannel> {
    let v = v?;
    let mut keyframes = Vec::new();
    // A channel that is itself `[x,y,z]`, a scalar, or a vec object is a static pose: one frame.
    if v.is_array() || v.is_string() || v.is_number() || v.get("vec").is_some() {
        let (pre, post, interpolation) = value_of(v, target)?;
        keyframes.push(Keyframe { time: 0.0, pre, post, interpolation });
    } else if let Some(obj) = v.as_object() {
        for (time, val) in obj {
            let t: f32 = match time.parse() {
                Ok(t) => t,
                Err(_) => continue, // an expression as a time key is not a static keyframe
            };
            if let Some((pre, post, interpolation)) = value_of(val, target) {
                keyframes.push(Keyframe { time: t, pre, post, interpolation });
            }
        }
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
    fn converts_units_and_keyframe_variants() {
        let text = r#"{"animations":{"a":{"bones":{
          "static":{"rotation":[90,0,0],"scale":"0.5","position":[0,2,0]},
          "unit_scale":{"scale":1},
          "keyed":{"rotation":{"0.0":[0,0,0],"0.5":{"pre":[45,0,0],"post":[45,0,0],"lerp_mode":"catmullrom"}}}
        }}}}"#;
        let holders = parse_bedrock_animation("ns", text).unwrap();
        let bones = &holders[0].bones;
        let st = &bones.iter().find(|(n, _)| n == "static").unwrap().1;
        assert_eq!(st.len(), 3);
        let rot = st.iter().find(|c| c.target == AnimationTarget::Rotation).unwrap();
        assert!((rot.keyframes[0].pre[0] - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        // Scale is an offset from 1: 0.5 -> -0.5.
        let scale = st.iter().find(|c| c.target == AnimationTarget::Scale).unwrap();
        assert_eq!(scale.keyframes[0].pre, [-0.5, -0.5, -0.5]);
        // Position y is negated: [0,2,0] -> [0,-2,0].
        let pos = st.iter().find(|c| c.target == AnimationTarget::Position).unwrap();
        assert_eq!(pos.keyframes[0].pre, [0.0, -2.0, 0.0]);
        // Scale 1 -> [0,0,0] (no change).
        let unit = &bones.iter().find(|(n, _)| n == "unit_scale").unwrap().1;
        assert_eq!(unit[0].keyframes[0].pre, [0.0, 0.0, 0.0]);
        let kd = &bones.iter().find(|(n, _)| n == "keyed").unwrap().1;
        assert_eq!(kd[0].keyframes.len(), 2);
        assert!(matches!(kd[0].keyframes[1].interpolation, Interpolation::CatmullRom));
    }

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
