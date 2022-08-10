use vek::{Mat4, Vec2, Vec3, Vec4};

use crate::muleengine::mesh::{Bone, Mesh, VertexBoneWeight};

pub fn create(radius: f32, height: f32, resolution: i32) -> Mesh {
    let mut mesh = Mesh::new();

    mesh.add_bone(Bone::new("root".to_string(), Mat4::identity()));

    let resolution_f32 = resolution as f32;

    let radian_step = std::f32::consts::PI * 2.0 / resolution_f32;

    let y_half_size = height / 2.0;

    let cylinder_trunk_texture_height = 0.75;
    let circle_texture_height = (1.0 - cylinder_trunk_texture_height) / 2.0;

    // sides of the cylinder
    for i in 0..resolution {
        let i = i as f32;
        let x0 = radius * f32::cos(i * radian_step);
        let z0 = radius * f32::sin(i * radian_step);

        let x1 = radius * f32::cos((i + 1.0) * radian_step);
        let z1 = radius * f32::sin((i + 1.0) * radian_step);

        let tx0 = i / resolution_f32;
        let ty0 = 0.0;

        let tx1 = (i + 1.0) / resolution_f32;
        let ty1 = 1.0;

        let normal_left = Vec3::new(x0, 0.0, z0).normalized();
        let normal_right = Vec3::new(x1, 0.0, z1).normalized();

        mesh.add_vertex(
            Vec3::new(x0, -y_half_size, z0),
            normal_left,
            None,
            None,
            vec![Vec2::new(tx0, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x1, -y_half_size, z1),
            normal_right,
            None,
            None,
            vec![Vec2::new(tx1, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x1, y_half_size, z1),
            normal_right,
            None,
            None,
            vec![Vec2::new(tx1, ty1)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x0, y_half_size, z0),
            normal_left,
            None,
            None,
            vec![Vec2::new(tx0, ty1)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );

        mesh.add_face(
            (mesh.number_of_vertices() - 2) as u32,
            (mesh.number_of_vertices() - 3) as u32,
            (mesh.number_of_vertices() - 4) as u32,
        );
        mesh.add_face(
            (mesh.number_of_vertices() - 1) as u32,
            (mesh.number_of_vertices() - 2) as u32,
            (mesh.number_of_vertices() - 4) as u32,
        );
    }

    // top of the cylinder
    for i in 0..resolution {
        let i = i as f32;
        let x0 = radius * f32::cos(i * radian_step);
        let z0 = radius * f32::sin(i * radian_step);

        let x1 = radius * f32::cos((i + 1.0) * radian_step);
        let z1 = radius * f32::sin((i + 1.0) * radian_step);

        let tx0 = i / resolution_f32;
        let tx1 = (i + 1.0) / resolution_f32;

        let ty0 = cylinder_trunk_texture_height + circle_texture_height;

        let normal = Vec3::new(0.0, 1.0, 0.0).normalized();

        mesh.add_vertex(
            Vec3::new(0.0, y_half_size, 0.0),
            normal,
            None,
            None,
            vec![Vec2::new(tx0 * 0.5 + tx1 * 0.5, 1.0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x0, y_half_size, z0),
            normal,
            None,
            None,
            vec![Vec2::new(tx0, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x1, y_half_size, z1),
            normal,
            None,
            None,
            vec![Vec2::new(tx1, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );

        mesh.add_face(
            (mesh.number_of_vertices() - 1) as u32,
            (mesh.number_of_vertices() - 2) as u32,
            (mesh.number_of_vertices() - 3) as u32,
        );
    }

    // bottom of the cylinder
    for i in 0..resolution {
        let i = i as f32;
        let x0 = radius * f32::cos(i * radian_step);
        let z0 = radius * f32::sin(i * radian_step);

        let x1 = radius * f32::cos((i + 1.0) * radian_step);
        let z1 = radius * f32::sin((i + 1.0) * radian_step);

        let tx0 = i / resolution_f32;
        let tx1 = (i + 1.0) / resolution_f32;

        let ty0 = circle_texture_height;

        let normal = Vec3::new(0.0, -1.0, 0.0).normalized();

        mesh.add_vertex(
            Vec3::new(0.0, -y_half_size, 0.0),
            normal,
            None,
            None,
            vec![Vec2::new(tx0 * 0.5 + tx1 * 0.5, -1.0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x1, -y_half_size, z1),
            normal,
            None,
            None,
            vec![Vec2::new(tx1, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x0, -y_half_size, z0),
            normal,
            None,
            None,
            vec![Vec2::new(tx0, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );

        mesh.add_face(
            (mesh.number_of_vertices() - 1) as u32,
            (mesh.number_of_vertices() - 2) as u32,
            (mesh.number_of_vertices() - 3) as u32,
        );
    }

    mesh.compute_tangents(0);

    mesh
}
