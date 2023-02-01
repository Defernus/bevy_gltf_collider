use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use bevy_rapier3d::prelude::Collider;

const COLLIDER_MESH_NAME: &'static str = "collider";

pub fn get_collider_from_mesh(mesh_name: &str, mesh: Mesh) -> Option<Collider> {
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();

    let indices = if let Some(indices) = mesh.indices() {
        indices
    } else {
        warn!(
            "Failed to get collider for {}: mesh has no indices",
            mesh_name
        );
        return None;
    };

    let positions = match positions {
        VertexAttributeValues::Float32x3(positions) => positions,
        v => {
            warn!(
                "Failed to get collider for {}: mesh has invalid positions type {}",
                mesh_name,
                v.enum_variant_name(),
            );
            return None;
        }
    };

    let indices: Vec<u32> = match indices {
        Indices::U32(indices) => indices.clone(),
        Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect(),
    };

    if indices.len() % 3 != 0 {
        warn!(
            "Failed to get collider for {}: mesh has invalid indices count {} (not divisible by 3)",
            mesh_name,
            indices.len(),
        );
        return None;
    }

    let triple_indices = indices.chunks(3).map(|v| [v[0], v[1], v[2]]).collect();
    let vertices = positions
        .iter()
        .map(|v| Vec3::new(v[0], v[1], v[2]))
        .collect();

    let collider = Collider::trimesh(vertices, triple_indices);

    return Some(collider);
}

fn colliders_filter(v: &(Entity, &Name)) -> bool {
    v.1.starts_with(COLLIDER_MESH_NAME)
}

/// Get all colliders from a scene.
///
/// It will search for all nodes with name starting with "collider" and will create a collider from the mesh.
///
/// NOTE: should be called only once per scene as it will remove the colliders meshes from it.
pub fn get_scene_colliders(
    scene_name: &str,
    meshes: &mut Assets<Mesh>,
    scene: &mut Scene,
) -> Vec<(Collider, Transform)> {
    let mut entities_to_despawn = Vec::new();
    let mut result = Vec::new();

    let mut meshes_q = scene.world.query::<(Entity, &Name)>();
    for (e, collider_name) in meshes_q.iter(&scene.world).filter(colliders_filter) {
        let children = scene.world.get::<Children>(e).expect(
            format!(
                "Failed to get collider for {}: node \"{}\" has no children",
                scene_name, collider_name
            )
            .as_str(),
        );

        let collider = children.iter().find_map(|&child| {
            if let Some(mesh) = scene.world.get::<Handle<Mesh>>(child) {
                let mesh = meshes.remove(mesh).unwrap();

                let mesh_name = format!("{}:{}", scene_name, collider_name);
                return get_collider_from_mesh(mesh_name.as_str(), mesh);
            }
            return None;
        });

        let collider = collider.expect(
            format!(
                "Failed to get collider for {}: no valid mesh found mesh",
                scene_name
            )
            .as_str(),
        );

        let transform = scene.world.get::<Transform>(e).unwrap().clone();
        result.push((collider, transform));

        entities_to_despawn.push(e);
    }

    for e in entities_to_despawn {
        despawn_with_children_recursive(&mut scene.world, e);
    }

    result
}
