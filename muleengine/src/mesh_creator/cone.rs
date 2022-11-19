use vek::{Mat4, Vec2, Vec3, Vec4};

use crate::mesh::{Bone, Mesh, VertexBoneWeight};

pub fn create(radius: f32, height: f32, resolution: i32) -> Mesh {
    let mut mesh = Mesh::new();

    mesh.add_bone(Bone::new("root".to_string(), Mat4::identity()));

    let resolution_f32 = resolution as f32;
    let radian_step = std::f32::consts::PI * 2.0 / resolution_f32;

    let y_half_size = height / 2.0;

    // sides of the cone
    for i in 0..resolution {
        let i = i as f32;
        let x0 = radius * f32::cos(i * radian_step);
        let z0 = radius * f32::sin(i * radian_step);

        let x1 = radius * f32::cos((i + 1.0) * radian_step);
        let z1 = radius * f32::sin((i + 1.0) * radian_step);

        let x05 = radius * f32::cos((i + 0.5) * radian_step);
        let z05 = radius * f32::sin((i + 0.5) * radian_step);

        // TODO(@RoGer) seamless cone normals // the problem is the tip of the cone
        // also not seamless normals are wrong, y should not be 0
        let normal_left = Vec3::new(x05, 0.0, z05).normalized();
        let normal_right = Vec3::new(x05, 0.0, z05).normalized();
        let normal_up = Vec3::new(x05, 0.0, z05).normalized();

        mesh.add_vertex(
            Vec3::new(0.0, y_half_size, 0.0),
            normal_up,
            None,
            None,
            vec![Vec2::new(0.5, 1.0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x0, -y_half_size, z0),
            normal_left,
            None,
            None,
            vec![Vec2::new(0.0, 0.0)],
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
            vec![Vec2::new(1.0, 0.0)],
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

    // bottom of the cone
    for i in 0..resolution {
        let i = i as f32;
        let x0 = radius * f32::cos(i * radian_step);
        let z0 = radius * f32::sin(i * radian_step);

        let x1 = radius * f32::cos((i + 1.0) * radian_step);
        let z1 = radius * f32::sin((i + 1.0) * radian_step);

        let tx0 = f32::cos(i * radian_step) / 2.0 + 0.5;
        let ty0 = f32::sin(i * radian_step) / 2.0 + 0.5;

        let tx1 = f32::cos((i + 1.0) * radian_step) / 2.0 + 0.5;
        let ty1 = f32::sin((i + 1.0) * radian_step) / 2.0 + 0.5;

        let normal = Vec3::new(0.0, -1.0, 0.0).normalized();

        mesh.add_vertex(
            Vec3::new(0.0, -y_half_size, 0.0),
            normal,
            None,
            None,
            vec![Vec2::new(0.5, 0.5)],
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
            vec![Vec2::new(tx1, ty1)],
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
