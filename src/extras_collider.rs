use bevy::{gltf::GltfExtras, prelude::*};
use bevy_rapier3d::prelude::Collider;
use serde::Deserialize;
use serde_json::Value;

//  example: {"colliders":[{"collider_type":"cuboid","value":[1,1,1],"translation":{"x":-0.000543,"y":0.087472,"z":-0.00163},"rotation":{"x":0,"y":0,"z":0,"w":1},"scale":{"x":0.107805,"y":0.081654,"z":0.13115}}]}

#[derive(Debug, Deserialize)]
struct Vec3Data {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vec3Data> for Vec3 {
    fn from(data: Vec3Data) -> Self {
        Vec3::new(data.x, data.y, data.z)
    }
}

#[derive(Debug, Deserialize)]
struct QuatData {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl From<QuatData> for Quat {
    fn from(data: QuatData) -> Self {
        Quat::from_xyzw(data.x, data.y, data.z, data.w)
    }
}

const COLLIDER_TYPE_CUBOID: &str = "cuboid";
// const COLLIDER_TYPE_SPHERE: &str = "sphere";
// const COLLIDER_TYPE_CYLINDER: &str = "cylinder";

#[derive(Debug, Deserialize)]
struct ColliderData {
    collider_type: String,
    value: Value,
    translation: Option<Vec3Data>,
    rotation: Option<QuatData>,
    scale: Option<Vec3Data>,
}

#[derive(Debug, Deserialize)]
struct Colliders {
    colliders: Vec<ColliderData>,
}

#[derive(Debug, Deserialize)]
struct ExtrasData {
    collider_data: String,
}

#[derive(Debug)]
pub enum ColliderExtrasParsingError {
    InvalidColliderDataFormat(String),
    UnknownColliderType(String),
    InvalidColliderValue(Value),
}

pub(super) fn process_extras_collider(
    extras: &GltfExtras,
) -> Option<Result<Vec<(Collider, Transform)>, ColliderExtrasParsingError>> {
    let ExtrasData { collider_data } = match serde_json::from_str(&extras.value) {
        Ok(value) => value,
        Err(err) => {
            if err.is_data() {
                return None;
            } else {
                return Some(Err(ColliderExtrasParsingError::InvalidColliderDataFormat(
                    err.to_string(),
                )));
            }
        }
    };

    let Colliders { colliders } = match serde_json::from_str(&collider_data) {
        Ok(value) => value,
        Err(err) => {
            return Some(Err(ColliderExtrasParsingError::InvalidColliderDataFormat(
                err.to_string(),
            )))
        }
    };

    let mut result = Vec::with_capacity(colliders.len());

    for c in colliders.into_iter() {
        let collider = match c.collider_type.as_str() {
            COLLIDER_TYPE_CUBOID => {
                let value = match &c.value {
                    Value::Array(value) => value,
                    _ => {
                        return Some(Err(ColliderExtrasParsingError::InvalidColliderValue(
                            c.value.clone(),
                        )))
                    }
                };

                if value.len() != 3 {
                    return Some(Err(ColliderExtrasParsingError::InvalidColliderValue(
                        c.value.clone(),
                    )));
                }

                let mut args = [0.0; 3];
                for (i, v) in value.iter().enumerate() {
                    args[i] = match v {
                        Value::Number(value) => value.as_f64().unwrap() as f32,
                        _ => {
                            return Some(Err(ColliderExtrasParsingError::InvalidColliderValue(
                                v.clone(),
                            )))
                        }
                    };
                }

                Collider::cuboid(args[0], args[1], args[2])
            }
            _ => {
                return Some(Err(ColliderExtrasParsingError::UnknownColliderType(
                    c.collider_type,
                )));
            }
        };

        let transform = Transform {
            translation: c.translation.map_or(Vec3::default(), |v| v.into()),
            rotation: c.rotation.map_or(Quat::default(), |v| v.into()),
            scale: c.scale.map_or(Vec3::default(), |v| v.into()),
        };

        result.push((collider, transform));
    }

    Some(Ok(result))
}
