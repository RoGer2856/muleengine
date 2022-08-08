use vek::{Mat4, Vec2, Vec3, Vec4};

use crate::muleengine::mesh::{Bone, Mesh, VertexBoneWeight};

pub fn create(width: f32, height: f32, depth: f32) -> Mesh {
    let mut mesh = Mesh::new();

    mesh.add_bone(Bone::new("root".to_string(), Mat4::identity()));

    let x_half_size = width / 2.0;
    let y_half_size = height / 2.0;
    let z_half_size = depth / 2.0;

    // left
    mesh.add_vertex(
        Vec3::new(-x_half_size, -y_half_size, -z_half_size),
        Vec3::new(-1.0, 0.0, 0.0),
        None,
        None,
        vec![Vec2::new(0.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(-x_half_size, -y_half_size, z_half_size),
        Vec3::new(-1.0, 0.0, 0.0),
        None,
        None,
        vec![Vec2::new(1.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(-x_half_size, y_half_size, z_half_size),
        Vec3::new(-1.0, 0.0, 0.0),
        None,
        None,
        vec![Vec2::new(1.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(-x_half_size, y_half_size, -z_half_size),
        Vec3::new(-1.0, 0.0, 0.0),
        None,
        None,
        vec![Vec2::new(0.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );

    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 3,
        mesh.number_of_vertices() - 2,
    );
    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 2,
        mesh.number_of_vertices() - 1,
    );

    // right
    mesh.add_vertex(
        Vec3::new(x_half_size, -y_half_size, z_half_size),
        Vec3::new(1.0, 0.0, 0.0),
        None,
        None,
        vec![Vec2::new(0.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, -y_half_size, -z_half_size),
        Vec3::new(1.0, 0.0, 0.0),
        None,
        None,
        vec![Vec2::new(1.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, y_half_size, -z_half_size),
        Vec3::new(1.0, 0.0, 0.0),
        None,
        None,
        vec![Vec2::new(1.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, y_half_size, z_half_size),
        Vec3::new(1.0, 0.0, 0.0),
        None,
        None,
        vec![Vec2::new(0.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );

    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 3,
        mesh.number_of_vertices() - 2,
    );
    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 2,
        mesh.number_of_vertices() - 1,
    );

    // top
    mesh.add_vertex(
        Vec3::new(-x_half_size, y_half_size, z_half_size),
        Vec3::new(0.0, 1.0, 0.0),
        None,
        None,
        vec![Vec2::new(0.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, y_half_size, z_half_size),
        Vec3::new(0.0, 1.0, 0.0),
        None,
        None,
        vec![Vec2::new(1.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, y_half_size, -z_half_size),
        Vec3::new(0.0, 1.0, 0.0),
        None,
        None,
        vec![Vec2::new(1.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(-x_half_size, y_half_size, -z_half_size),
        Vec3::new(0.0, 1.0, 0.0),
        None,
        None,
        vec![Vec2::new(0.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );

    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 3,
        mesh.number_of_vertices() - 2,
    );
    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 2,
        mesh.number_of_vertices() - 1,
    );

    // bottom
    mesh.add_vertex(
        Vec3::new(-x_half_size, -y_half_size, -z_half_size),
        Vec3::new(0.0, -1.0, 0.0),
        None,
        None,
        vec![Vec2::new(0.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, -y_half_size, -z_half_size),
        Vec3::new(0.0, -1.0, 0.0),
        None,
        None,
        vec![Vec2::new(1.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, -y_half_size, z_half_size),
        Vec3::new(0.0, -1.0, 0.0),
        None,
        None,
        vec![Vec2::new(1.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(-x_half_size, -y_half_size, z_half_size),
        Vec3::new(0.0, -1.0, 0.0),
        None,
        None,
        vec![Vec2::new(0.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );

    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 3,
        mesh.number_of_vertices() - 2,
    );
    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 2,
        mesh.number_of_vertices() - 1,
    );

    // front
    mesh.add_vertex(
        Vec3::new(-x_half_size, -y_half_size, z_half_size),
        Vec3::new(0.0, 0.0, 1.0),
        None,
        None,
        vec![Vec2::new(0.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, -y_half_size, z_half_size),
        Vec3::new(0.0, 0.0, 1.0),
        None,
        None,
        vec![Vec2::new(1.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, y_half_size, z_half_size),
        Vec3::new(0.0, 0.0, 1.0),
        None,
        None,
        vec![Vec2::new(1.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(-x_half_size, y_half_size, z_half_size),
        Vec3::new(0.0, 0.0, 1.0),
        None,
        None,
        vec![Vec2::new(0.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );

    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 3,
        mesh.number_of_vertices() - 2,
    );
    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 2,
        mesh.number_of_vertices() - 1,
    );

    // back
    mesh.add_vertex(
        Vec3::new(x_half_size, -y_half_size, -z_half_size),
        Vec3::new(0.0, 0.0, -1.0),
        None,
        None,
        vec![Vec2::new(0.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(-x_half_size, -y_half_size, -z_half_size),
        Vec3::new(0.0, 0.0, -1.0),
        None,
        None,
        vec![Vec2::new(1.0, 0.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(-x_half_size, y_half_size, -z_half_size),
        Vec3::new(0.0, 0.0, -1.0),
        None,
        None,
        vec![Vec2::new(1.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    mesh.add_vertex(
        Vec3::new(x_half_size, y_half_size, -z_half_size),
        Vec3::new(0.0, 0.0, -1.0),
        None,
        None,
        vec![Vec2::new(0.0, 1.0)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );

    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 3,
        mesh.number_of_vertices() - 2,
    );
    mesh.add_face(
        mesh.number_of_vertices() - 4,
        mesh.number_of_vertices() - 2,
        mesh.number_of_vertices() - 1,
    );

    mesh.compute_tangents(0);

    mesh
}
