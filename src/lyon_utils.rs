//! Mainly taken from bevy_input_prototype
use bevy::{prelude::*, render::mesh::Indices};
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ParseError;

struct Geometry(pub VertexBuffers<[f32; 3], u32>);

impl From<Geometry> for Mesh {
    fn from(geometry: Geometry) -> Self {
        let num_vertices = geometry.0.vertices.len();
        let mut mesh = Self::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(geometry.0.indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, geometry.0.vertices);

        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        for _ in 0..num_vertices {
            normals.push([0.0, 0.0, 0.0]);
            uvs.push([0.0, 0.0]);
        }

        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}

/// Returns a `SpriteComponents` bundle with the given [`Geometry`](Geometry)
/// and `ColorMaterial`.
///
/// adapted from [bevy_prototype_lyon](https://github.com/Nilirad/bevy_prototype_lyon/blob/master/src/path.rs)
fn create_sprite(
    material: Handle<ColorMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
    geometry: Geometry,
    translation: Vec3,
) -> SpriteBundle {
    SpriteBundle {
        material,
        mesh: meshes.add(geometry.into()),
        sprite: Sprite {
            size: Vec2::new(1.0, 1.0),
            ..Default::default()
        },
        transform: Transform::from_translation(translation),
        ..Default::default()
    }
}

/// Stroke to bevy components.
///
/// adapted from [bevy_prototype_lyon](https://github.com/Nilirad/bevy_prototype_lyon/blob/master/src/path.rs)
pub fn stroke(
    path: lyon::path::Path,
    material: Handle<ColorMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
    translation: Vec3,
    options: &StrokeOptions,
) -> SpriteBundle {
    let mut tessellator = StrokeTessellator::new();
    let mut geometry = Geometry(VertexBuffers::new());
    tessellator
        .tessellate_path(
            path.as_slice(),
            options,
            &mut BuffersBuilder::new(&mut geometry.0, |pos: StrokeVertex| {
                [pos.position().x, pos.position().y, 0.0]
            }),
        )
        .unwrap();

    create_sprite(material, meshes, geometry, translation)
}

/// Fill to bevy components.
///
/// adapted from [bevy_prototype_lyon](https://github.com/Nilirad/bevy_prototype_lyon/blob/master/src/path.rs)
pub fn fill(
    path: lyon::path::Path,
    material: Handle<ColorMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
    translation: Vec3,
    options: &FillOptions,
) -> SpriteBundle {
    let mut tessellator = FillTessellator::new();
    let mut geometry = Geometry(VertexBuffers::new());
    tessellator
        .tessellate_path(
            path.as_slice(),
            options,
            &mut BuffersBuilder::new(&mut geometry.0, |pos: FillVertex| {
                [pos.position().x, pos.position().y, 0.0]
            }),
        )
        .unwrap();

    create_sprite(material, meshes, geometry, translation)
}
