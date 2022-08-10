use vek::{Mat4, Vec2, Vec3, Vec4};

use crate::muleengine::mesh::{Bone, Mesh, VertexBoneWeight};

pub fn create(radius: f32, height: f32, resolution: i32) -> Mesh {
    let mut mesh = Mesh::new();

    mesh.add_bone(Bone::new("root".to_string(), Mat4::identity()));

    // binary and operator for making sure resolution is even
    let resolution = resolution & !0x1;
    let resolution_f32 = resolution as f32;
    let half_resolution = resolution >> 1;

    // rectangle
    let x_half_size = radius;
    let y_half_size = (height - radius * 2.0) / 2.0;
    let z_plane = 0.0;

    mesh.add_vertex(
        Vec3::new(-x_half_size, -y_half_size, z_plane),
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
        Vec3::new(x_half_size, -y_half_size, z_plane),
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
        Vec3::new(x_half_size, y_half_size, z_plane),
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
        Vec3::new(-x_half_size, y_half_size, z_plane),
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
        (mesh.number_of_vertices() - 4) as u32,
        (mesh.number_of_vertices() - 3) as u32,
        (mesh.number_of_vertices() - 2) as u32,
    );
    mesh.add_face(
        (mesh.number_of_vertices() - 4) as u32,
        (mesh.number_of_vertices() - 2) as u32,
        (mesh.number_of_vertices() - 1) as u32,
    );

    // up semicircle
    mesh.add_vertex(
        Vec3::new(0.0, y_half_size, z_plane),
        Vec3::new(0.0, 0.0, 1.0),
        None,
        None,
        vec![Vec2::new(0.5, 0.5)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    let top_center_index = mesh.number_of_vertices() - 1;

    mesh.add_vertex(
        Vec3::new(radius, y_half_size, z_plane),
        Vec3::new(0.0, 0.0, 1.0),
        None,
        None,
        vec![Vec2::new(1.0, 0.5)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );

    for i in 0..half_resolution {
        let i = i as f32;
        let angle = i * std::f32::consts::PI * 2.0 / resolution_f32;
        let x = f32::cos(angle);
        let y = f32::sin(angle);
        mesh.add_vertex(
            Vec3::new(x * radius, y * radius + y_half_size, z_plane),
            Vec3::new(0.0, 0.0, 1.0),
            None,
            None,
            vec![Vec2::new((x + 1.0) * 0.5, (y + 1.0) * 0.5)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );

        mesh.add_face(
            top_center_index as u32,
            (mesh.number_of_vertices() - 2) as u32,
            (mesh.number_of_vertices() - 1) as u32,
        );
    }

    mesh.add_face(
        top_center_index as u32,
        (mesh.number_of_vertices() - 1) as u32,
        (top_center_index + 1) as u32,
    );

    // down semicircle
    mesh.add_vertex(
        Vec3::new(0.0, -y_half_size, z_plane),
        Vec3::new(0.0, 0.0, 1.0),
        None,
        None,
        vec![Vec2::new(0.5, 0.5)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );
    let bottom_center_index = mesh.number_of_vertices() - 1;

    mesh.add_vertex(
        Vec3::new(-radius, -y_half_size, z_plane),
        Vec3::new(0.0, 0.0, 1.0),
        None,
        None,
        vec![Vec2::new(1.0, 0.5)],
        VertexBoneWeight {
            bone_ids: Vec4::broadcast(0),
            weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
        },
    );

    for i in 0..half_resolution {
        let i = i as f32;
        let angle = i * std::f32::consts::PI * 2.0 / resolution_f32 + std::f32::consts::PI;
        let x = f32::cos(angle);
        let y = f32::sin(angle);
        mesh.add_vertex(
            Vec3::new(x * radius, y * radius - y_half_size, z_plane),
            Vec3::new(0.0, 0.0, 1.0),
            None,
            None,
            vec![Vec2::new((x + 1.0) * 0.5, (y + 1.0) * 0.5)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );

        mesh.add_face(
            bottom_center_index as u32,
            (mesh.number_of_vertices() - 2) as u32,
            (mesh.number_of_vertices() - 1) as u32,
        );
    }

    mesh.add_face(
        bottom_center_index as u32,
        (mesh.number_of_vertices() - 1) as u32,
        (bottom_center_index + 1) as u32,
    );

    mesh.compute_tangents(0);

    mesh
}
