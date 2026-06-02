use crate::cube::Cube;
use crate::cube::moves::Face;
use crate::renderer::RotationAnim;
use egui::{Color32, Pos2, Sense, Shape, Ui, Vec2};

/// 3D 视图渲染器
pub struct View3D {
    pub sticker_size: f32,
    pub view_rot_y: f32, // 绕 Y 轴旋转角度（弧度）
    pub view_rot_x: f32, // 绕 X 轴旋转角度（弧度）
}

impl Default for View3D {
    fn default() -> Self {
        Self {
            sticker_size: 30.0,
            view_rot_y: 0.5,
            view_rot_x: -0.4,
        }
    }
}

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn rotate_y(&self, angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self::new(self.x * c - self.z * s, self.y, self.x * s + self.z * c)
    }

    fn rotate_x(&self, angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self::new(self.x, self.y * c - self.z * s, self.y * s + self.z * c)
    }

    fn rotate_z(&self, angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self::new(self.x * c - self.y * s, self.x * s + self.y * c, self.z)
    }

    fn add(&self, other: Vec3) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

/// 一个需要绘制的面片
struct FaceQuad {
    verts: [Vec3; 4],
    color: Color32,
    depth: f32,
}

/// 判断小方块是否属于指定旋转层
fn is_in_rotation_layer(pos: Vec3, face: Face) -> bool {
    match face {
        Face::U => pos.y > 0.5,   // y = 1
        Face::D => pos.y < -0.5,  // y = -1
        Face::F => pos.z > 0.5,   // z = 1
        Face::B => pos.z < -0.5,  // z = -1
        Face::R => pos.x > 0.5,   // x = 1
        Face::L => pos.x < -0.5,  // x = -1
    }
}

/// 围绕指定面的法向量轴旋转一个点
fn rotate_around_face_axis(p: Vec3, face: Face, angle: f32) -> Vec3 {
    match face {
        Face::U | Face::D => p.rotate_y(angle),
        Face::F | Face::B => p.rotate_z(angle),
        Face::R | Face::L => p.rotate_x(angle),
    }
}

impl View3D {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        cube: &Cube,
        rotation: Option<RotationAnim>,
    ) -> Option<Face> {
        let size = Vec2::splat(350.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::drag());

        // 处理鼠标拖拽旋转视角
        if response.dragged() {
            let delta = response.drag_delta();
            self.view_rot_y += delta.x * 0.01;
            self.view_rot_x += delta.y * 0.01;
            self.view_rot_x = self.view_rot_x.clamp(-1.2, 1.2);
        }

        let center = rect.center();
        let scale = self.sticker_size;

        let mut quads: Vec<FaceQuad> = Vec::new();

        // 生成 27 个小方块
        for ix in -1..=1 {
            for iy in -1..=1 {
                for iz in -1..=1 {
                    let pos = Vec3::new(ix as f32, iy as f32, iz as f32);
                    self.add_cubie(&mut quads, cube, pos, rotation);
                }
            }
        }

        // 按深度排序（画家算法，远的先画）
        quads.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());

        // 绘制
        let painter = ui.painter();
        for q in quads {
            // 用旋转后的 3D 法向量做背面剔除
            let normal = compute_normal(&q.verts);
            if normal.z <= 0.0 {
                continue;
            }

            let points: Vec<Pos2> = q
                .verts
                .iter()
                .map(|v| {
                    Pos2::new(center.x + v.x * scale, center.y - v.y * scale)
                })
                .collect();

            // 简单的光照
            let light = Vec3::new(0.3, 0.5, 0.8);
            let intensity = 0.6
                + 0.4
                    * ((normal.x * light.x + normal.y * light.y + normal.z * light.z)
                        / (light.x * light.x + light.y * light.y + light.z * light.z).sqrt())
                        .max(0.0);

            let lit_color = apply_lighting(q.color, intensity);

            painter.add(Shape::convex_polygon(
                points.clone(),
                lit_color,
                egui::Stroke::new(1.0, Color32::BLACK),
            ));
        }

        // 点击检测
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                return self.pick_face(center, scale, pos);
            }
        }

        None
    }

    fn add_cubie(
        &self,
        quads: &mut Vec<FaceQuad>,
        cube: &Cube,
        pos: Vec3,
        rotation: Option<RotationAnim>,
    ) {
        let s = 0.48;

        // 判断该小方块是否在当前旋转动画的层中
        let is_rotating = rotation.map_or(false, |r| is_in_rotation_layer(pos, r.face));
        let rot_angle = rotation.map_or(0.0, |r| r.angle);

        // 对小方块中心位置应用旋转动画
        let rot_pos = if is_rotating {
            rotate_around_face_axis(pos, rotation.unwrap().face, rot_angle)
        } else {
            pos
        };

        let ix = (pos.x + 1.5) as usize;
        let iy = (1.5 - pos.y) as usize;
        let iz = (pos.z + 1.5) as usize;

        // 辅助函数：定义面的局部偏移，并在需要时应用动画旋转
        let mut push_face = |_face: Face, should_show: bool, offsets: [Vec3; 4], color_idx_fn: fn(&Cube, usize, usize, usize) -> crate::cube::Color| {
            if !should_show {
                return;
            }
            let color = color_idx_fn(cube, ix, iy, iz).as_egui();
            let mut verts = [Vec3::new(0.0, 0.0, 0.0); 4];
            for i in 0..4 {
                let local = offsets[i];
                // 如果小方块在旋转层中，对局部偏移也应用同样的旋转
                let rotated_local = if is_rotating {
                    rotate_around_face_axis(local, rotation.unwrap().face, rot_angle)
                } else {
                    local
                };
                // 先叠加中心位置，再做视角旋转
                let world = rot_pos.add(rotated_local);
                verts[i] = world.rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            }
            let depth = (verts[0].z + verts[1].z + verts[2].z + verts[3].z) / 4.0;
            quads.push(FaceQuad { verts, color, depth });
        };

        // +X (R)
        push_face(
            Face::R,
            pos.x > 0.5,
            [
                Vec3::new(s, -s, -s),
                Vec3::new(s, s, -s),
                Vec3::new(s, s, s),
                Vec3::new(s, -s, s),
            ],
            |cube, _ix, iy, iz| cube.faces[Face::R as usize][iy][2 - iz],
        );

        // -X (L)
        push_face(
            Face::L,
            pos.x < -0.5,
            [
                Vec3::new(-s, -s, s),
                Vec3::new(-s, s, s),
                Vec3::new(-s, s, -s),
                Vec3::new(-s, -s, -s),
            ],
            |cube, _ix, iy, iz| cube.faces[Face::L as usize][iy][iz],
        );

        // +Y (U)
        push_face(
            Face::U,
            pos.y > 0.5,
            [
                Vec3::new(-s, s, -s),
                Vec3::new(-s, s, s),
                Vec3::new(s, s, s),
                Vec3::new(s, s, -s),
            ],
            |cube, ix, _iy, iz| cube.faces[Face::U as usize][iz][ix],
        );

        // -Y (D)
        push_face(
            Face::D,
            pos.y < -0.5,
            [
                Vec3::new(-s, -s, s),
                Vec3::new(-s, -s, -s),
                Vec3::new(s, -s, -s),
                Vec3::new(s, -s, s),
            ],
            |cube, ix, _iy, iz| cube.faces[Face::D as usize][2 - iz][ix],
        );

        // +Z (F)
        push_face(
            Face::F,
            pos.z > 0.5,
            [
                Vec3::new(-s, -s, s),
                Vec3::new(s, -s, s),
                Vec3::new(s, s, s),
                Vec3::new(-s, s, s),
            ],
            |cube, ix, iy, _iz| cube.faces[Face::F as usize][iy][ix],
        );

        // -Z (B)
        push_face(
            Face::B,
            pos.z < -0.5,
            [
                Vec3::new(s, -s, -s),
                Vec3::new(-s, -s, -s),
                Vec3::new(-s, s, -s),
                Vec3::new(s, s, -s),
            ],
            |cube, ix, iy, _iz| cube.faces[Face::B as usize][iy][2 - ix],
        );
    }

    fn pick_face(&self, center: Pos2, scale: f32, mouse: Pos2) -> Option<Face> {
        let s = 1.5;
        let candidates = [
            (Face::R, Vec3::new(s, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
            (Face::L, Vec3::new(-s, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
            (Face::U, Vec3::new(0.0, s, 0.0), Vec3::new(0.0, 1.0, 0.0)),
            (Face::D, Vec3::new(0.0, -s, 0.0), Vec3::new(0.0, -1.0, 0.0)),
            (Face::F, Vec3::new(0.0, 0.0, s), Vec3::new(0.0, 0.0, 1.0)),
            (Face::B, Vec3::new(0.0, 0.0, -s), Vec3::new(0.0, 0.0, -1.0)),
        ];

        let mut best: Option<(Face, f32)> = None;

        for (face, center_3d, normal) in &candidates {
            let rotated_normal = normal.rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            if rotated_normal.z <= 0.0 {
                continue;
            }

            let rc = center_3d.rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let screen = Pos2::new(center.x + rc.x * scale, center.y - rc.y * scale);
            let dist = (screen - mouse).length();

            if dist < scale * 1.5 {
                if best.map_or(true, |(_, bd)| rc.z > bd) {
                    best = Some((*face, rc.z));
                }
            }
        }

        best.map(|(f, _)| f)
    }
}

fn compute_normal(verts: &[Vec3]) -> Vec3 {
    if verts.len() < 3 {
        return Vec3::new(0.0, 0.0, 1.0);
    }
    let a = Vec3::new(verts[1].x - verts[0].x, verts[1].y - verts[0].y, verts[1].z - verts[0].z);
    let b = Vec3::new(verts[2].x - verts[0].x, verts[2].y - verts[0].y, verts[2].z - verts[0].z);
    Vec3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

fn apply_lighting(color: Color32, intensity: f32) -> Color32 {
    Color32::from_rgb(
        (color.r() as f32 * intensity) as u8,
        (color.g() as f32 * intensity) as u8,
        (color.b() as f32 * intensity) as u8,
    )
}
