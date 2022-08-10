use vek::{Mat4, Vec2, Vec3, Vec4};

use crate::muleengine::mesh::{Bone, Mesh, VertexBoneWeight};

pub fn create(radius: f32, resolution: i32) -> Mesh {
    let mut mesh = Mesh::new();

    mesh.add_bone(Bone::new("root".to_string(), Mat4::identity()));

    // binary and operator for making sure resolution is even
    let resolution = resolution & !0x1;
    let resolution_f32 = resolution as f32;
    let radian_step_x = std::f32::consts::PI * 2.0 / resolution_f32;
    let radian_step_y = std::f32::consts::PI / resolution_f32;

    // up hemisphere
    for y in 0..resolution / 2 - 1 {
        for x in 0..resolution {
            let x = x as f32;
            let y = y as f32;

            let tmp_radius_0 = radius * f32::cos(y * radian_step_y);
            let tmp_radius_1 = radius * f32::cos((y + 1.0) * radian_step_y);

            let y0 = radius * f32::sin(y * radian_step_y);
            let y1 = radius * f32::sin((y + 1.0) * radian_step_y);

            let x00 = tmp_radius_0 * f32::cos(x * radian_step_x);
            let z00 = tmp_radius_0 * f32::sin(x * radian_step_x);

            let x10 = tmp_radius_0 * f32::cos((x + 1.0) * radian_step_x);
            let z10 = tmp_radius_0 * f32::sin((x + 1.0) * radian_step_x);

            let x01 = tmp_radius_1 * f32::cos(x * radian_step_x);
            let z01 = tmp_radius_1 * f32::sin(x * radian_step_x);

            let x11 = tmp_radius_1 * f32::cos((x + 1.0) * radian_step_x);
            let z11 = tmp_radius_1 * f32::sin((x + 1.0) * radian_step_x);

            let normal00 = Vec3::new(x00, y0, z00).normalized();
            let normal10 = Vec3::new(x10, y0, z10).normalized();
            let normal11 = Vec3::new(x11, y1, z11).normalized();
            let normal01 = Vec3::new(x01, y1, z01).normalized();

            let tx0 = x / resolution_f32;
            let ty0 = y / resolution_f32 + 0.5;

            let tx1 = (x + 1.0) / resolution_f32;
            let ty1 = (y + 1.0) / resolution_f32 + 0.5;

            mesh.add_vertex(
                Vec3::new(x00, y0, z00),
                normal00,
                None,
                None,
                vec![Vec2::new(tx0, ty0)],
                VertexBoneWeight {
                    bone_ids: Vec4::broadcast(0),
                    weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
                },
            );
            mesh.add_vertex(
                Vec3::new(x10, y0, z10),
                normal10,
                None,
                None,
                vec![Vec2::new(tx1, ty0)],
                VertexBoneWeight {
                    bone_ids: Vec4::broadcast(0),
                    weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
                },
            );
            mesh.add_vertex(
                Vec3::new(x11, y1, z11),
                normal11,
                None,
                None,
                vec![Vec2::new(tx1, ty1)],
                VertexBoneWeight {
                    bone_ids: Vec4::broadcast(0),
                    weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
                },
            );
            mesh.add_vertex(
                Vec3::new(x01, y1, z01),
                normal01,
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
    }

    for x in 0..resolution {
        let x = x as f32;
        let y = resolution_f32 / 2.0 - 1.0;

        let tmp_radius_0 = radius * f32::cos(y * radian_step_y);

        let y0 = radius * f32::sin(y * radian_step_y);

        let x00 = tmp_radius_0 * f32::cos(x * radian_step_x);
        let z00 = tmp_radius_0 * f32::sin(x * radian_step_x);

        let x10 = tmp_radius_0 * f32::cos((x + 1.0) * radian_step_x);
        let z10 = tmp_radius_0 * f32::sin((x + 1.0) * radian_step_x);

        let normal00 = Vec3::new(x00, y0, z00).normalized();
        let normal10 = Vec3::new(x10, y0, z10).normalized();
        let normal_up = Vec3::new(0.0, 1.0, 0.0).normalized();

        let tx0 = x / resolution_f32;
        let tx1 = (x + 1.0) / resolution_f32;

        let ty0 = y / resolution_f32 + 0.5;

        mesh.add_vertex(
            Vec3::new(x00, y0, z00),
            normal00,
            None,
            None,
            vec![Vec2::new(tx0, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x10, y0, z10),
            normal10,
            None,
            None,
            vec![Vec2::new(tx1, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(0.0, radius, 0.0),
            normal_up,
            None,
            None,
            vec![Vec2::new(tx0 * 0.5 + tx1 * 0.5, 1.0)],
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

    // down hemisphere
    for y in 0..resolution / 2 - 1 {
        for x in 0..resolution {
            let x = x as f32;
            let y = y as f32;

            let tmp_radius_0 = radius * f32::cos(-y * radian_step_y);
            let tmp_radius_1 = radius * f32::cos((-y - 1.0) * radian_step_y);

            let y0 = radius * f32::sin(-y * radian_step_y);
            let y1 = radius * f32::sin((-y - 1.0) * radian_step_y);

            let x00 = tmp_radius_0 * f32::cos(x * radian_step_x);
            let z00 = tmp_radius_0 * f32::sin(x * radian_step_x);

            let x10 = tmp_radius_0 * f32::cos((x + 1.0) * radian_step_x);
            let z10 = tmp_radius_0 * f32::sin((x + 1.0) * radian_step_x);

            let x01 = tmp_radius_1 * f32::cos(x * radian_step_x);
            let z01 = tmp_radius_1 * f32::sin(x * radian_step_x);

            let x11 = tmp_radius_1 * f32::cos((x + 1.0) * radian_step_x);
            let z11 = tmp_radius_1 * f32::sin((x + 1.0) * radian_step_x);

            let normal00 = Vec3::new(x00, y0, z00).normalized();
            let normal10 = Vec3::new(x10, y0, z10).normalized();
            let normal11 = Vec3::new(x11, y1, z11).normalized();
            let normal01 = Vec3::new(x01, y1, z01).normalized();

            let tx0 = x / resolution_f32;
            let ty0 = y / resolution_f32 + 0.5;

            let tx1 = (x + 1.0) / resolution_f32;
            let ty1 = 0.5 - (y + 1.0) / resolution_f32;

            mesh.add_vertex(
                Vec3::new(x00, y0, z00),
                normal00,
                None,
                None,
                vec![Vec2::new(tx0, ty0)],
                VertexBoneWeight {
                    bone_ids: Vec4::broadcast(0),
                    weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
                },
            );
            mesh.add_vertex(
                Vec3::new(x10, y0, z10),
                normal10,
                None,
                None,
                vec![Vec2::new(tx1, ty0)],
                VertexBoneWeight {
                    bone_ids: Vec4::broadcast(0),
                    weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
                },
            );
            mesh.add_vertex(
                Vec3::new(x11, y1, z11),
                normal11,
                None,
                None,
                vec![Vec2::new(tx1, ty1)],
                VertexBoneWeight {
                    bone_ids: Vec4::broadcast(0),
                    weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
                },
            );
            mesh.add_vertex(
                Vec3::new(x01, y1, z01),
                normal01,
                None,
                None,
                vec![Vec2::new(tx0, ty1)],
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
        }
    }

    for x in 0..resolution {
        let x = x as f32;
        let y = resolution_f32 / 2.0 - 1.0;

        let tmp_radius_0 = radius * f32::cos(-y * radian_step_y);

        let y0 = radius * f32::sin(-y * radian_step_y);

        let x00 = tmp_radius_0 * f32::cos(x * radian_step_x);
        let z00 = tmp_radius_0 * f32::sin(x * radian_step_x);

        let x10 = tmp_radius_0 * f32::cos((x + 1.0) * radian_step_x);
        let z10 = tmp_radius_0 * f32::sin((x + 1.0) * radian_step_x);

        let normal00 = Vec3::new(x00, y0, z00).normalized();
        let normal10 = Vec3::new(x10, y0, z10).normalized();
        let normal_up = Vec3::new(0.0, -1.0, 0.0).normalized();

        let tx0 = x / resolution_f32;
        let tx1 = (x + 1.0) / resolution_f32;

        let ty0 = 0.5 - y / resolution_f32;

        mesh.add_vertex(
            Vec3::new(x00, y0, z00),
            normal00,
            None,
            None,
            vec![Vec2::new(tx0, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(x10, y0, z10),
            normal10,
            None,
            None,
            vec![Vec2::new(tx0, ty0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );
        mesh.add_vertex(
            Vec3::new(0.0, -radius, 0.0),
            normal_up,
            None,
            None,
            vec![Vec2::new(tx0 * 0.5 + tx1 * 0.5, 0.0)],
            VertexBoneWeight {
                bone_ids: Vec4::broadcast(0),
                weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
            },
        );

        mesh.add_face(
            (mesh.number_of_vertices() - 3) as u32,
            (mesh.number_of_vertices() - 2) as u32,
            (mesh.number_of_vertices() - 1) as u32,
        );
    }

    mesh.compute_tangents(0);

    mesh
}
