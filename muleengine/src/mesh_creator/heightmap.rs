use vek::{Mat4, Vec2, Vec3, Vec4};

use crate::{
    heightmap::HeightMap,
    mesh::{Bone, Mesh, VertexBoneWeight},
};

pub fn create(height_map: &HeightMap) -> Mesh {
    let mut mesh = Mesh::new();

    /*
     *
     * faces
     *  (-x, -z)       (x, -z)
     *          +--1--2
     *          | /| /|
     *          |/ |/ |
     *          0--P--3
     *          | /| /|
     *          |/ |/ |
     *          5--4--+
     *   (-x, z)       (x, z)
     *
     */

    if height_map.get_column_count() != 0 && height_map.get_row_count() != 0 {
        mesh.add_bone(Bone::new("root".to_string(), Mat4::identity()));

        let center_x = 0.5;
        let center_y = 0.5;
        let center_z = 0.5;

        let column_count_f32 = height_map.get_column_count() as f32;
        let row_count_f32 = height_map.get_row_count() as f32;

        for y in 0..height_map.get_row_count() {
            for x in 0..height_map.get_column_count() {
                let x_f32 = x as f32;
                let y_f32 = y as f32;

                let position_pivot = Vec3::new(
                    x_f32 / column_count_f32,
                    height_map.get_height_map()[x][y],
                    y_f32 / row_count_f32,
                );

                let mut position0 = Vec3::new(
                    (x_f32 - 1.0) / column_count_f32,
                    height_map.get_height_map()[x][y],
                    y_f32 / row_count_f32,
                );
                let mut position1 = Vec3::new(
                    (x_f32 - 1.0) / column_count_f32,
                    height_map.get_height_map()[x][y],
                    (y_f32 - 1.0) / row_count_f32,
                );
                let mut position2 = Vec3::new(
                    x_f32 / column_count_f32,
                    height_map.get_height_map()[x][y],
                    (y_f32 - 1.0) / row_count_f32,
                );
                let mut position3 = Vec3::new(
                    (x_f32 + 1.0) / column_count_f32,
                    height_map.get_height_map()[x][y],
                    y_f32 / row_count_f32,
                );
                let mut position4 = Vec3::new(
                    (x_f32 + 1.0) / column_count_f32,
                    height_map.get_height_map()[x][y],
                    (y_f32 + 1.0) / row_count_f32,
                );
                let mut position5 = Vec3::new(
                    x_f32 / column_count_f32,
                    height_map.get_height_map()[x][y],
                    (y_f32 + 1.0) / row_count_f32,
                );

                if x > 0 {
                    position0.y = height_map.get_height_map()[x - 1][y];
                }
                if x > 0 && y > 0 {
                    position1.y = height_map.get_height_map()[x - 1][y - 1];
                }
                if y > 0 {
                    position2.y = height_map.get_height_map()[x][y - 1];
                }
                if x < height_map.get_column_count() - 1 {
                    position3.y = height_map.get_height_map()[x + 1][y];
                }
                if x < height_map.get_column_count() - 1 && y < height_map.get_row_count() - 1 {
                    position4.y = height_map.get_height_map()[x + 1][y + 1];
                }
                if y < height_map.get_row_count() - 1 {
                    position5.y = height_map.get_height_map()[x][y + 1];
                }

                let normal0 = (position1 - position_pivot)
                    .cross(position0 - position_pivot)
                    .normalized();
                let normal1 = (position2 - position_pivot)
                    .cross(position1 - position_pivot)
                    .normalized();
                let normal2 = (position3 - position_pivot)
                    .cross(position2 - position_pivot)
                    .normalized();
                let normal3 = (position4 - position_pivot)
                    .cross(position3 - position_pivot)
                    .normalized();
                let normal4 = (position5 - position_pivot)
                    .cross(position4 - position_pivot)
                    .normalized();
                let normal5 = (position0 - position_pivot)
                    .cross(position5 - position_pivot)
                    .normalized();

                let normal = (normal0 + normal1 + normal2 + normal3 + normal4 + normal5) / 6.0;

                mesh.add_vertex(
                    position_pivot - Vec3::new(center_x, center_y, center_z),
                    normal,
                    None,
                    None,
                    vec![Vec2::new(x_f32 / column_count_f32, y_f32 / row_count_f32)],
                    VertexBoneWeight {
                        bone_ids: Vec4::broadcast(0),
                        weights: Vec4::new(1.0, 0.0, 0.0, 0.0),
                    },
                );
            }
        }

        for y in 0..height_map.get_row_count() - 1 {
            for x in 0..height_map.get_column_count() - 1 {
                if height_map.get_mask_map()[x][y] != 0
                    && height_map.get_mask_map()[x + 1][y] != 0
                    && height_map.get_mask_map()[x + 1][y + 1] != 0
                {
                    mesh.add_face(
                        ((y + 1) * height_map.get_row_count() + x + 1) as u32,
                        ((y + 0) * height_map.get_row_count() + x + 1) as u32,
                        ((y + 0) * height_map.get_row_count() + x + 0) as u32,
                    );
                }
                if height_map.get_mask_map()[x][y] != 0
                    && height_map.get_mask_map()[x + 1][y + 1] != 0
                    && height_map.get_mask_map()[x][y + 1] != 0
                {
                    mesh.add_face(
                        ((y + 1) * height_map.get_row_count() + x + 0) as u32,
                        ((y + 1) * height_map.get_row_count() + x + 1) as u32,
                        ((y + 0) * height_map.get_row_count() + x + 0) as u32,
                    );
                }
            }
        }
    }

    mesh.compute_tangents(0);

    mesh
}
