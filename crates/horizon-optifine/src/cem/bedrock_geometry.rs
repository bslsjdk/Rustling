//! A parser for Bedrock-format geometry models in JSON. A document declares bones with an
//! absolute pivot and cubes at an origin and size; this bakes them into the engine's [`Bone`] tree
//! of quads, converting y-up file coordinates to the y-down model axes. The quads share the
//! representation the other model formats produce, so callers need only one geometry type.

use super::json::{Json, get};
use super::model::{Bake, Bone, Shape};

/// One geometry baked to model pixels on the engine's axes (y down). UVs in the quads are a
/// fraction of [`texture_size`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeometryModel {
    /// The declared identifier, when the document gives one.
    pub identifier: Option<String>,
    /// The declared texture dimensions, in pixels.
    pub texture_size: [f32; 2],
    pub bones: Vec<Bone>,
}

/// The bytes one document may hold, so loading can't read or bake without bound.
const MAX_BYTES: usize = 4 << 20;

/// Parses every geometry a document declares. An error means the document as a whole can't be
/// used; a bone or cube with missing or invalid numbers is left out.
pub fn parse_geometry(text: &str) -> Result<Vec<GeometryModel>, String> {
    if text.len() > MAX_BYTES {
        return Err(format!("geometry: larger than {MAX_BYTES} bytes"));
    }
    let root = Json::parse(text).map_err(|e| format!("geometry: {e}"))?;
    let mut out = Vec::new();
    if let Some(geometries) = root.get("minecraft:geometry").and_then(Json::as_array) {
        for value in geometries.iter() {
            if let Some(object) = value.as_object() {
                out.push(geometry(object));
            }
        }
    } else {
        for (key, value) in root.as_object().into_iter().flatten() {
            if key.starts_with("geometry.") {
                if let Some(object) = value.as_object() {
                    out.push(geometry(object));
                }
            }
        }
    }
    if out.is_empty() {
        return Err("geometry: no geometry".into());
    }
    Ok(out)
}

fn geometry(object: &[(String, Json)]) -> GeometryModel {
    let description = get(object, "description").and_then(Json::as_object);
    let identifier = description
        .and_then(|d| get(d, "identifier"))
        .and_then(Json::as_str)
        .map(str::to_owned);
    // One declaration puts the size in `description`; another writes it on the geometry itself.
    let size_key = |key: &str| {
        description
            .and_then(|d| get(d, key))
            .or_else(|| get(object, key))
    };
    let texture_size = [
        num(size_key("texture_width").or_else(|| size_key("texturewidth"))).unwrap_or(64.0),
        num(size_key("texture_height").or_else(|| size_key("textureheight"))).unwrap_or(32.0),
    ];
    let shape = Shape {
        invert: [false, true, false],
        mirror_u: false,
        mirror_v: false,
        size: texture_size,
    };
    let mut bake = Bake::new();
    let origin = [0.0; 3];
    let bones = get(object, "bones")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .filter_map(Json::as_object)
        .map(|b| bone(b, origin, &shape, &mut bake))
        .collect();
    GeometryModel {
        identifier,
        texture_size,
        bones,
    }
}

fn bone(object: &[(String, Json)], parent: [f32; 3], shape: &Shape, bake: &mut Bake) -> Bone {
    let absolute = nums::<3>(get(object, "pivot")).unwrap_or_default();
    // A bone is offset from the pivot of the bone above it; y flips with the coordinate system.
    let pivot = std::array::from_fn(|i| {
        let delta = absolute[i] - parent[i];
        if i == 1 { -delta } else { delta }
    });
    let rotation = nums::<3>(get(object, "rotation")).unwrap_or_default();
    let rotation = [
        rotation[0].to_radians(),
        -rotation[1].to_radians(),
        rotation[2].to_radians(),
    ];
    let scale = match get(object, "scale") {
        Some(Json::Num(s)) => {
            let s = *s as f32;
            [if s.is_finite() { s } else { 1.0 }; 3]
        }
        v => nums::<3>(v).unwrap_or([1.0; 3]),
    };
    let mut quads = Vec::new();
    for cube in get(object, "cubes")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .filter_map(Json::as_object)
    {
        let (Some(origin), Some(size)) =
            (nums::<3>(get(cube, "origin")), nums::<3>(get(cube, "size")))
        else {
            continue;
        };
        let mut shape = shape.clone();
        shape.mirror_u = boolean(get(cube, "mirror")).unwrap_or(false);
        let coordinates = origin
            .into_iter()
            .chain(size)
            .map(|n| Json::Num(f64::from(n)))
            .collect();
        let mut box_definition = vec![("coordinates".into(), Json::Arr(coordinates))];
        match get(cube, "uv") {
            Some(Json::Arr(a)) if a.len() >= 2 => {
                if let (Some(u), Some(v)) = (num(a.first()), num(a.get(1))) {
                    box_definition.push((
                        "textureOffset".into(),
                        Json::Arr(vec![Json::Num(f64::from(u)), Json::Num(f64::from(v))]),
                    ));
                }
            }
            Some(faces) if faces.as_object().is_some() => {
                for (face, key) in [
                    ("up", "uvUp"),
                    ("down", "uvDown"),
                    ("north", "uvNorth"),
                    ("south", "uvSouth"),
                    ("west", "uvWest"),
                    ("east", "uvEast"),
                ] {
                    let Some(rect) = faces.get(face) else {
                        continue;
                    };
                    let uv = rect.get("uv").and_then(Json::as_array);
                    let size_uv = rect.get("uv_size").and_then(Json::as_array);
                    let (Some(u), Some(v), Some(w), Some(h)) = (
                        uv.and_then(|a| num(a.first())),
                        uv.and_then(|a| num(a.get(1))),
                        size_uv.and_then(|a| num(a.first())),
                        size_uv.and_then(|a| num(a.get(1))),
                    ) else {
                        continue;
                    };
                    box_definition.push((
                        key.into(),
                        Json::Arr(vec![
                            Json::Num(f64::from(u)),
                            Json::Num(f64::from(v)),
                            Json::Num(f64::from(u + w)),
                            Json::Num(f64::from(v + h)),
                        ]),
                    ));
                }
            }
            _ => {}
        }
        shape.bake(&box_definition, &mut quads, bake);
    }
    let children = get(object, "children")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .filter_map(Json::as_object)
        .map(|child| bone(child, absolute, shape, bake))
        .collect();
    Bone {
        id: get(object, "name")
            .and_then(Json::as_str)
            .map(str::to_owned),
        pivot,
        rotation,
        scale,
        attachments: Vec::new(),
        quads,
        children,
    }
}

/// A finite number, written as a number or as a string.
fn num(value: Option<&Json>) -> Option<f32> {
    match value? {
        Json::Num(n) => {
            let n = *n as f32;
            n.is_finite().then_some(n)
        }
        Json::Str(s) => s.trim().parse::<f32>().ok().filter(|n| n.is_finite()),
        _ => None,
    }
}

fn boolean(value: Option<&Json>) -> Option<bool> {
    match value? {
        Json::Bool(b) => Some(*b),
        Json::Str(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn nums<const N: usize>(value: Option<&Json>) -> Option<[f32; N]> {
    let array = value?.as_array()?;
    (array.len() >= N).then(|| std::array::from_fn(|i| num(array.get(i)).unwrap_or(0.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_BONES: &str = r#"{"format_version":"1.12.0","minecraft:geometry":[{
        "description":{"identifier":"geometry.test","texture_width":16,"texture_height":16},
        "bones":[
            {"name":"root","pivot":[0,0,0],"cubes":[
                {"origin":[-2,0,-2],"size":[4,4,4],"uv":[0,0]}]},
            {"name":"child","parent":"root","pivot":[0,8,0],"cubes":[
                {"origin":[-1,8,-1],"size":[2,2,2],"uv":[0,0]}]}
        ]}]}"#;

    #[test]
    fn bones_keep_hierarchy_pivots_and_faces() {
        let models = parse_geometry(TWO_BONES).unwrap();
        assert_eq!(models.len(), 1);
        let model = &models[0];
        assert_eq!(model.identifier.as_deref(), Some("geometry.test"));
        assert_eq!(model.texture_size, [16.0, 16.0]);
        assert_eq!(model.bones.len(), 2);
        let root = &model.bones[0];
        let child = &model.bones[1];
        assert_eq!(root.id.as_deref(), Some("root"));
        assert_eq!(root.pivot, [0.0, 0.0, 0.0]);
        assert_eq!(root.quads.len(), 6);
        assert_eq!(child.id.as_deref(), Some("child"));
        assert_eq!(child.pivot, [0.0, -8.0, 0.0]);
        assert_eq!(child.quads.len(), 6);
    }

    #[test]
    fn a_y_up_cube_spans_from_minus_its_top_to_minus_its_bottom() {
        let model = &parse_geometry(TWO_BONES).unwrap()[0];
        let ys = model.bones[0]
            .quads
            .iter()
            .flat_map(|q| q.vertices.iter().map(|v| v[1]));
        for y in ys {
            assert!((-4.0..=0.0).contains(&y), "y-out-of-range: {y}");
        }
    }

    #[test]
    fn per_face_uvs_reach_the_matching_quads() {
        let text = r#"{"minecraft:geometry":[{"description":{"texture_width":8,"texture_height":8},
            "bones":[{"name":"part","cubes":[{"origin":[0,0,0],"size":[1,1,1],"uv":{
                "up":{"uv":[1,2],"uv_size":[3,4]},"north":{"uv":[5,6],"uv_size":[2,2]}}}]}]}]}"#;
        let model = &parse_geometry(text).unwrap()[0];
        let up = model.bones[0]
            .quads
            .iter()
            .find(|q| q.normal == [0.0, -1.0, 0.0])
            .unwrap();
        let north = model.bones[0]
            .quads
            .iter()
            .find(|q| q.normal == [0.0, 0.0, -1.0])
            .unwrap();
        assert_eq!(up.vertices[0][3], 4.0 / 8.0);
        assert_eq!(north.vertices[0][3], 7.0 / 8.0);
    }

    #[test]
    fn named_geometry_object_form_is_read() {
        let text = r#"{"format_version":"1.10.0","geometry.model":{
            "texturewidth":8,"textureheight":8,
            "bones":[{"name":"part","pivot":[0,2,0],"cubes":[
                {"origin":[-1,2,-1],"size":[2,2,2],"uv":[0,0]}]}]}}"#;
        let models = parse_geometry(text).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].texture_size, [8.0, 8.0]);
        assert_eq!(models[0].bones.len(), 1);
        assert_eq!(models[0].bones[0].quads.len(), 6);
    }

    #[test]
    fn a_document_without_geometry_is_rejected() {
        assert!(parse_geometry("{}").is_err());
    }
}
