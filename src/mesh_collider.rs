use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};
use bevy_rapier3d::prelude::Collider;
use serde::{Deserialize, Serialize};

const COLLIDER_MESH_NAME: &str = "collider";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColliderFromMeshError {
    MissingPositions,
    MissingIndices,
    InvalidIndicesCount(usize),
    InvalidPositionsType(&'static str),
    NonManifold,
    ConvexHullComputationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColliderMeshParsingError {
    MissingMeshNode,
    MissingMesh,
    MeshColliderError(ColliderFromMeshError),
}

/// How you want your mesh to be treated when creating a collider from it.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColliderMeshType {
    /// Uses [`Collider::convex_hull`] to take the vertices of the mesh and create a convex
    /// collider from them. Your mesh doesn't need to be convex for this to work.
    ///
    /// See also https://rapier.rs/docs/user_guides/bevy_plugin/colliders#convex-meshes
    ConvexHull,
    /// Uses [`Collider::convex_mesh`] create a convex collider from the mesh data. Your mesh must
    /// be convex and contain no non-manifold geometry for this to work!
    ///
    /// Note that I haven't been able to get this to work with any Blender models, including
    /// simple shapes. Help wanted!
    ConvexMesh,
    /// Triangle meshes (in 3D) and polylines (in 2D) can be used to describe the boundary of any
    /// kind of shape. This is generally useful to describe the fixed environment in games
    /// (terrains, buildings, etc.) Triangle meshes and polylines are defined by their vertex
    /// buffer and their index buffer. The winding of the triangles of a triangle mesh does not
    /// matter. Its topology doesn't matter either (it can have holes, cavities, doesn't need to be
    /// closed or manifold). It is however strongly recommended to avoid triangles that are long
    /// and thin because they can result in a lower numerical stability of collision-detection.
    /// https://rapier.rs/docs/user_guides/bevy_plugin/colliders#triangle-meshes-and-polylines
    ///
    /// WARNING: It is discouraged to use a triangle meshes or polylines for colliders attached
    /// to dynamic rigid-bodies. Because they have no interior, it is easy for another object to
    /// get stuck into them.
    TriMesh,
}

pub fn get_collider_from_mesh(
    mesh: &Mesh,
    mesh_type: ColliderMeshType,
) -> Result<Collider, ColliderFromMeshError> {
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

    let triple_indices: Vec<_> = indices.chunks(3).map(|v| [v[0], v[1], v[2]]).collect();
    let vertices: Vec<_> = positions
        .iter()
        .map(|v| Vec3::new(v[0], v[1], v[2]))
        .collect();

    match mesh_type {
        ColliderMeshType::ConvexHull => {
            Collider::convex_hull(&vertices).ok_or(ColliderFromMeshError::NonManifold)
        }
        ColliderMeshType::ConvexMesh => Collider::convex_mesh(vertices, triple_indices.as_slice())
            .ok_or(ColliderFromMeshError::NonManifold),
        ColliderMeshType::TriMesh => Ok(Collider::trimesh(vertices, triple_indices)),
    }
}

pub(super) fn process_mesh_collider(
    node_name: &str,
    children: Option<&Children>,
    world: &World,
    meshes: &mut Assets<Mesh>,
    mesh_type: ColliderMeshType,
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
            Some(get_collider_from_mesh(&mesh, mesh_type))
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
