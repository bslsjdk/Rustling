// Bedrock-format geometry JSON to the engine's model part tree. The format declares bones with
// an absolute pivot and cubes at an origin and size; this builds a mesh of named parts and bakes
// it into a `ModelPart`. File coordinates are y-up, so cube origins and bone pivots are mirrored
// on y into the model's y-down frame.

use mod_api::{
    CubeDeformation, CubeListBuilder, LayerDefinition, MeshDefinition, ModelPart, PartId, PartPose,
};
use serde_json::Value;
use std::collections::HashMap;

/// Converts the first geometry in a document to a baked part tree. An error means the document
/// can't be read or holds no geometry; bones without a usable name are left out.
pub fn geometry_to_parts(text: &str) -> Result<ModelPart, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| format!("geometry: {e}"))?;
    let geometry = find_geometry(&root).ok_or_else(|| "geometry: no geometry".to_string())?;
    let (width, height) = texture_size(geometry);
    let mut mesh = MeshDefinition::new();
    let root_id = mesh.root();
    let mut pivots: HashMap<&str, [f32; 3]> = HashMap::new();
    let mut ids: HashMap<&str, PartId> = HashMap::new();
    let mut pending: Vec<&Value> = geometry
        .get("bones")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|bone| bone.is_object())
        .collect();
    // A parent may be declared after its child, so keep going until no bone can be placed.
    let mut progressed = true;
    while progressed && !pending.is_empty() {
        progressed = false;
        let mut next = Vec::new();
        for bone in pending {
            let name = bone.get("name").and_then(Value::as_str);
            let parent_name = bone.get("parent").and_then(Value::as_str);
            let ready = parent_name.is_none_or(|parent| ids.contains_key(parent));
            let (Some(name), true) = (name, ready) else {
                next.push(bone);
                continue;
            };
            if ids.contains_key(name) {
                progressed = true;
                continue;
            }
            let parent = parent_name
                .and_then(|parent| ids.get(parent))
                .copied()
                .unwrap_or(root_id);
            let parent_pivot = parent_name
                .and_then(|parent| pivots.get(parent))
                .copied()
                .unwrap_or_default();
            let pivot = vec3(bone.get("pivot")).unwrap_or_default();
            let pose = pose(pivot, parent_pivot, bone.get("rotation"), bone.get("scale"));
            let id = mesh.add_or_replace_child(parent, name, cubes(bone), pose);
            ids.insert(name, id);
            pivots.insert(name, pivot);
            progressed = true;
        }
        pending = next;
    }
    Ok(LayerDefinition::create(mesh, width, height).bake_root())
}

/// The array form and the named-object form of a geometry document.
fn find_geometry(root: &Value) -> Option<&Value> {
    if let Some(geometries) = root.get("minecraft:geometry").and_then(Value::as_array) {
        return geometries.first();
    }
    root.as_object()?
        .iter()
        .find(|(key, value)| key.starts_with("geometry.") && value.is_object())
        .map(|(_, value)| value)
}

fn texture_size(geometry: &Value) -> (u32, u32) {
    let description = geometry.get("description");
    let number = |key: &str| {
        description
            .and_then(|d| d.get(key))
            .or_else(|| geometry.get(key))
            .and_then(Value::as_f64)
    };
    let width = number("texture_width")
        .or_else(|| number("texturewidth"))
        .unwrap_or(64.0);
    let height = number("texture_height")
        .or_else(|| number("textureheight"))
        .unwrap_or(32.0);
    (width.max(1.0) as u32, height.max(1.0) as u32)
}

/// A bone's pose is its pivot relative to the parent's pivot, on y-down axes.
fn pose(
    pivot: [f32; 3],
    parent: [f32; 3],
    rotation: Option<&Value>,
    scale: Option<&Value>,
) -> PartPose {
    let angle = vec3(rotation).unwrap_or_default();
    let mut pose = PartPose::offset_and_rotation(
        pivot[0] - parent[0],
        -(pivot[1] - parent[1]),
        pivot[2] - parent[2],
        angle[0].to_radians(),
        -angle[1].to_radians(),
        angle[2].to_radians(),
    );
    let scale = match scale {
        Some(Value::Number(n)) => {
            let s = n.as_f64().unwrap_or(1.0) as f32;
            [s, s, s]
        }
        other => vec3(other).unwrap_or([1.0, 1.0, 1.0]),
    };
    pose.x_scale = scale[0];
    pose.y_scale = scale[1];
    pose.z_scale = scale[2];
    pose
}

fn cubes(bone: &Value) -> CubeListBuilder {
    let mut builder = CubeListBuilder::create();
    let Some(list) = bone.get("cubes").and_then(Value::as_array) else {
        return builder;
    };
    for cube in list.iter().filter(|cube| cube.is_object()) {
        let (Some(origin), Some(size)) = (vec3(cube.get("origin")), vec3(cube.get("size"))) else {
            continue;
        };
        builder = builder.set_mirror(cube.get("mirror").and_then(Value::as_bool).unwrap_or(false));
        // An origin is the box's minimum corner; mirroring y makes its minimum the negated top.
        builder = builder.add_box_at(
            [origin[0], -(origin[1] + size[1]), origin[2]],
            size,
            CubeDeformation::NONE,
            uv(cube.get("uv")),
        );
    }
    builder
}

fn uv(value: Option<&Value>) -> [i32; 2] {
    match value {
        Some(Value::Array(a)) if a.len() >= 2 => [
            a[0].as_f64().unwrap_or(0.0) as i32,
            a[1].as_f64().unwrap_or(0.0) as i32,
        ],
        // A per-face layout has no single offset; the first declared face stands in for it.
        Some(Value::Object(faces)) => faces
            .values()
            .filter_map(Value::as_object)
            .map(|face| uv(face.get("uv")))
            .find(|offset| *offset != [0, 0])
            .unwrap_or([0, 0]),
        _ => [0, 0],
    }
}

fn vec3(value: Option<&Value>) -> Option<[f32; 3]> {
    let array = value?.as_array()?;
    (array.len() >= 3).then(|| std::array::from_fn(|i| array[i].as_f64().unwrap_or(0.0) as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_BONES: &str = r#"{"format_version":"1.12.0","minecraft:geometry":[{
        "description":{"identifier":"geometry.test","texture_width":16,"texture_height":16},
        "bones":[
            {"name":"root","pivot":[0,0,0],"cubes":[
                {"origin":[-2,0,-2],"size":[4,4,4],"uv":[0,0]}]},
            {"name":"child","parent":"root","pivot":[0,8,0],"rotation":[0,90,0],
             "scale":2,"cubes":[{"origin":[-1,8,-1],"size":[2,2,2],"uv":[0,0]}]}
        ]}]}"#;

    #[test]
    fn bones_become_named_parts_with_relative_poses() {
        let baked = geometry_to_parts(TWO_BONES).unwrap();
        assert_eq!(baked.children().count(), 1);
        let root = baked.get_child("root").unwrap();
        assert_eq!((root.x, root.y, root.z), (0.0, 0.0, 0.0));
        assert_eq!(root.children().count(), 1);
        let child = root.get_child("child").unwrap();
        assert_eq!((child.x, child.y, child.z), (0.0, -8.0, 0.0));
        assert_eq!(child.y_rot, -std::f32::consts::FRAC_PI_2);
        assert_eq!((child.x_scale, child.y_scale, child.z_scale), (2.0, 2.0, 2.0));
    }

    #[test]
    fn cubes_bake_into_faces_below_the_pivot() {
        let baked = geometry_to_parts(TWO_BONES).unwrap();
        let root = baked.get_child("root").unwrap();
        let child = root.get_child("child").unwrap();
        assert_eq!(root.polygons.len(), 6);
        assert_eq!(child.polygons.len(), 6);
        let ys = root
            .polygons
            .iter()
            .flat_map(|poly| poly.vertices.iter().map(|vertex| vertex.pos[1]));
        for y in ys {
            assert!((-4.0..=0.0).contains(&y), "y-out-of-range: {y}");
        }
    }

    #[test]
    fn a_child_declared_before_its_parent_is_still_attached() {
        let text = r#"{"minecraft:geometry":[{"description":{"texture_width":8,"texture_height":8},
            "bones":[
                {"name":"child","parent":"root","pivot":[1,2,3]},
                {"name":"root","pivot":[0,0,0]}
            ]}]}"#;
        let baked = geometry_to_parts(text).unwrap();
        let root = baked.get_child("root").unwrap();
        let child = root.get_child("child").unwrap();
        assert_eq!((child.x, child.y, child.z), (1.0, -2.0, 3.0));
    }
}
