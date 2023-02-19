use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use bevy_rapier3d::prelude::Collider;

const COLLIDER_MESH_NAME: &str = "collider";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColliderFromMeshError {
    MissingPositions,
    MissingIndices,
    InvalidIndicesCount(usize),
    InvalidPositionsType(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColliderMeshParsingError {
    MissingMeshNode,
    MissingMesh,
    MeshColliderError(ColliderFromMeshError),
}

pub fn get_collider_from_mesh(mesh: &Mesh) -> Result<Collider, ColliderFromMeshError> {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .map_or(Err(ColliderFromMeshError::MissingPositions), Ok)?;

    let indices = mesh
        .indices()
        .map_or(Err(ColliderFromMeshError::MissingIndices), Ok)?;

    let positions = match positions {
        VertexAttributeValues::Float32x3(positions) => positions,
        v => {
            return Err(ColliderFromMeshError::InvalidPositionsType(
                v.enum_variant_name(),
            ));
        }
    };

    let indices: Vec<u32> = match indices {
        Indices::U32(indices) => indices.clone(),
        Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect(),
    };

    if indices.len() % 3 != 0 {
        return Err(ColliderFromMeshError::InvalidIndicesCount(indices.len()));
    }

    let triple_indices = indices.chunks(3).map(|v| [v[0], v[1], v[2]]).collect();
    let vertices = positions
        .iter()
        .map(|v| Vec3::new(v[0], v[1], v[2]))
        .collect();

    let collider = Collider::trimesh(vertices, triple_indices);

    Ok(collider)
}

pub(super) fn process_mesh_collider(
    node_name: &str,
    children: Option<&Children>,
    world: &World,
    meshes: &mut Assets<Mesh>,
) -> Option<Result<Collider, ColliderMeshParsingError>> {
    if !node_name.starts_with(COLLIDER_MESH_NAME) {
        return None;
    }

    let children = if let Some(children) = children {
        children
    } else {
        return Some(Err(ColliderMeshParsingError::MissingMeshNode));
    };

    let collider = children.iter().find_map(|&child| {
        if let Some(mesh) = world.get::<Handle<Mesh>>(child) {
            let mesh = meshes.remove(mesh).unwrap();

            Some(get_collider_from_mesh(&mesh))
        } else {
            None
        }
    });

    Some(
        collider.map_or(Err(ColliderMeshParsingError::MissingMesh), |v| {
            v.map_err(ColliderMeshParsingError::MeshColliderError)
        }),
    )
}
