use bevy::{gltf::GltfExtras, prelude::*};
use bevy_rapier3d::prelude::Collider;
use extras_collider::{process_extras_collider, ColliderExtrasParsingError};
use mesh_collider::{process_mesh_collider, ColliderMeshParsingError};

pub mod extras_collider;
pub mod mesh_collider;

#[derive(Debug)]
pub enum ColliderFromSceneError {
    MeshParsingError(ColliderMeshParsingError),
    ExtrasParsingError(ColliderExtrasParsingError),
    NoCollidersFound,
}

/// Get all colliders from a scene.
///
/// It will search for all nodes with name starting with "collider" and will create a collider from the mesh.
///
/// NOTE: should be called only once per scene as it will remove the colliders meshes from it.
pub fn get_scene_colliders(
    meshes: &mut Assets<Mesh>,
    world: &mut World,
) -> Result<Vec<(Collider, Transform)>, ColliderFromSceneError> {
    let mut result = Vec::new();

    let mut extras_q = world.query::<&GltfExtras>();
    for extras in extras_q.iter(world) {
        match process_extras_collider(extras) {
            None => {}
            Some(Err(err)) => return Err(ColliderFromSceneError::ExtrasParsingError(err)),
            Some(Ok(colliders)) => {
                for c in colliders {
                    result.push(c);
                }
            }
        }
    }

    let mut entities_to_despawn = Vec::new();
    let mut meshes_q = world.query::<(Entity, &Name, Option<&Children>)>();
    for (entity, entity_name, children) in meshes_q.iter(world) {
        match process_mesh_collider(entity_name, children, world, meshes) {
            None => {}
            Some(Err(err)) => return Err(ColliderFromSceneError::MeshParsingError(err)),
            Some(Ok(collider)) => {
                let transform = *world.get::<Transform>(entity).unwrap();
                result.push((collider, transform));
                entities_to_despawn.push(entity);
            }
        }
    }

    for e in entities_to_despawn {
        despawn_with_children_recursive(world, e);
    }

    Ok(result)
}
