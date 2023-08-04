use crate::mesh_collider::ColliderMeshType;
use bevy::{gltf::GltfExtras, prelude::*};
use bevy_rapier3d::prelude::Collider;
use extras_collider::{process_extras_collider, ColliderExtrasParsingError};
use mesh_collider::{process_mesh_collider, ColliderMeshParsingError};
use serde::{Deserialize, Serialize};

pub mod extras_collider;
pub mod mesh_collider;

#[derive(Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component)]
pub struct SerializedCollider {
    collider: Vec<u8>,
}

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
    mesh_type: ColliderMeshType,
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
        match process_mesh_collider(entity_name, children, world, meshes, mesh_type) {
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

///Pulls all of the scene colliders out of a scene, then parents them to a node with a matching suffix
pub fn extract_insert_scene_colliders(
    meshes: &mut Assets<Mesh>,
    world: &mut World,
    mesh_type: ColliderMeshType,
) -> Result<Vec<(Collider, Transform, Name)>, ColliderFromSceneError> {
    let mut result = Vec::new();
    let mut entities_to_despawn = Vec::new();
    let mut meshes_q = world.query::<(Entity, &Name, Option<&Children>)>();
    let mut names_q = world.query::<(Entity, &Name)>();

    for (entity, entity_name, children) in meshes_q.iter(world) {
        match process_mesh_collider(entity_name, children, world, meshes, mesh_type) {
            None => {}
            Some(Err(err)) => return Err(ColliderFromSceneError::MeshParsingError(err)),
            Some(Ok(collider)) => {
                let transform = *world.get::<Transform>(entity).unwrap();
                result.push((collider, transform, entity_name.clone()));
                entities_to_despawn.push(entity);
            }
        }
    }

    for e in entities_to_despawn {
        despawn_with_children_recursive(world, e);
    }

    //Go over all the found colliders and see if we can find an entity with a matching name
    result.iter_mut().for_each(|(collider, _transform, name)|{
        let Some(new_ent_name) = name.split("collider_").last() else {
            return;
        };

        let Some((new_ent, _new_name)) = names_q.iter(world).find(|val|{val.1.trim() == new_ent_name.trim()})else{
            warn!("Could not find matching Node for collider: {}", name);
            return;
        };

        //If we found one that matches go ahead and add the collider
        if let Ok(serialized_collider) = bincode::serialize(collider){
            world.entity_mut(new_ent).insert(SerializedCollider{collider: serialized_collider});
        }else{
            error!("Could not serialize collider found in GLTF");
        }
    });

    Ok(result)
}

fn hydrate_serialized_colliders(
    colliders_to_add: Query<(&SerializedCollider, Entity), Added<SerializedCollider>>,
    mut cmds: Commands,
) {
    colliders_to_add.iter().for_each(|(collider_to_add, entity)|{
        let Ok(collider) = bincode::deserialize::<Collider>(&collider_to_add.collider) else{
            error!("Could not deserialize SerializedCollider from GLTF Scene to Rapier3d Collider!");
            return;
        };

        let Some(mut entcmds) = cmds.get_entity(entity) else{
            warn!("Could not find entity to attach Deserialized Collider to!");
            return;
        };

        entcmds.insert(collider);
    });
}

pub struct GLTFColliderPlugin;

impl Plugin for GLTFColliderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SerializedCollider>();
        app.add_systems(Update, (hydrate_serialized_colliders,));
    }
}
