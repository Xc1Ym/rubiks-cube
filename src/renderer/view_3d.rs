use crate::cube::Cube;
use crate::cube::moves::Face;
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
}

/// 一个需要绘制的面片
struct FaceQuad {
    verts: [Vec3; 4],
    color: Color32,
    depth: f32,
}

impl View3D {
    pub fn show(&mut self, ui: &mut Ui, cube: &Cube) -> Option<Face> {
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

        // 生成 27 个小方块，每个小方块最多 3 个可见面
        for ix in -1..=1 {
            for iy in -1..=1 {
                for iz in -1..=1 {
                    let pos = Vec3::new(ix as f32, iy as f32, iz as f32);
                    self.add_cubie(&mut quads, cube, pos, scale);
                }
            }
        }

        // 按深度排序（画家算法，远的先画）
        quads.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());

        // 绘制
        let painter = ui.painter();
        for q in quads {
            // 用旋转后的 3D 法向量做背面剔除：z > 0 表示面朝向我们
            let normal = compute_normal(&q.verts);
            if normal.z <= 0.0 {
                continue;
            }

            let points: Vec<Pos2> = q
                .verts
                .iter()
                .map(|v| {
                    // 正交投影 + 中心偏移
                    Pos2::new(center.x + v.x * scale, center.y - v.y * scale)
                })
                .collect();

            // 简单的光照：根据法向量与光源方向点积调整颜色
            let light = Vec3::new(0.3, 0.5, 0.8); // 光源方向
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

        // 点击检测：检测点击了哪个外层面
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                return self.pick_face(center, scale, pos);
            }
        }

        None
    }

    fn add_cubie(&self, quads: &mut Vec<FaceQuad>, cube: &Cube, pos: Vec3, _scale: f32) {
        let s = 0.48; // 略小于 0.5，留出缝隙
        let ix = (pos.x + 1.5) as usize;
        let iy = (1.5 - pos.y) as usize; // y 翻转：y=1 是上面，row=0
        let iz = (pos.z + 1.5) as usize;

        // 对于每个外露面，创建四边形
        // +X (R)
        if pos.x > 0.5 {
            let c = cube.faces[Face::R as usize][iy][2 - iz]; // R 面 row=iy, col 映射
            let color = c.as_egui();
            let v0 = Vec3::new(pos.x + s, pos.y - s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v1 = Vec3::new(pos.x + s, pos.y + s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v2 = Vec3::new(pos.x + s, pos.y + s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v3 = Vec3::new(pos.x + s, pos.y - s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let depth = (v0.z + v1.z + v2.z + v3.z) / 4.0;
            quads.push(FaceQuad { verts: [v0, v1, v2, v3], color, depth });
        }
        // -X (L)
        if pos.x < -0.5 {
            let c = cube.faces[Face::L as usize][iy][iz]; // L 面 row=iy, col=iz
            let color = c.as_egui();
            let v0 = Vec3::new(pos.x - s, pos.y - s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v1 = Vec3::new(pos.x - s, pos.y + s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v2 = Vec3::new(pos.x - s, pos.y + s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v3 = Vec3::new(pos.x - s, pos.y - s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let depth = (v0.z + v1.z + v2.z + v3.z) / 4.0;
            quads.push(FaceQuad { verts: [v0, v1, v2, v3], color, depth });
        }
        // +Y (U)
        if pos.y > 0.5 {
            let c = cube.faces[Face::U as usize][iz][ix]; // U 面 row=iz, col=ix
            let color = c.as_egui();
            let v0 = Vec3::new(pos.x - s, pos.y + s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v1 = Vec3::new(pos.x - s, pos.y + s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v2 = Vec3::new(pos.x + s, pos.y + s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v3 = Vec3::new(pos.x + s, pos.y + s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let depth = (v0.z + v1.z + v2.z + v3.z) / 4.0;
            quads.push(FaceQuad { verts: [v0, v1, v2, v3], color, depth });
        }
        // -Y (D)
        if pos.y < -0.5 {
            let c = cube.faces[Face::D as usize][2 - iz][ix]; // D 面 row=2-iz, col=ix
            let color = c.as_egui();
            let v0 = Vec3::new(pos.x - s, pos.y - s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v1 = Vec3::new(pos.x - s, pos.y - s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v2 = Vec3::new(pos.x + s, pos.y - s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v3 = Vec3::new(pos.x + s, pos.y - s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let depth = (v0.z + v1.z + v2.z + v3.z) / 4.0;
            quads.push(FaceQuad { verts: [v0, v1, v2, v3], color, depth });
        }
        // +Z (F) —— 顶点顺序需保证法向量朝 +Z
        if pos.z > 0.5 {
            let c = cube.faces[Face::F as usize][iy][ix]; // F 面 row=iy, col=ix
            let color = c.as_egui();
            let v0 = Vec3::new(pos.x - s, pos.y - s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v1 = Vec3::new(pos.x + s, pos.y - s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v2 = Vec3::new(pos.x + s, pos.y + s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v3 = Vec3::new(pos.x - s, pos.y + s, pos.z + s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let depth = (v0.z + v1.z + v2.z + v3.z) / 4.0;
            quads.push(FaceQuad { verts: [v0, v1, v2, v3], color, depth });
        }
        // -Z (B) —— 顶点顺序需保证法向量朝 -Z
        if pos.z < -0.5 {
            let c = cube.faces[Face::B as usize][iy][2 - ix]; // B 面 row=iy, col=2-ix
            let color = c.as_egui();
            let v0 = Vec3::new(pos.x + s, pos.y - s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v1 = Vec3::new(pos.x - s, pos.y - s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v2 = Vec3::new(pos.x - s, pos.y + s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let v3 = Vec3::new(pos.x + s, pos.y + s, pos.z - s).rotate_y(self.view_rot_y).rotate_x(self.view_rot_x);
            let depth = (v0.z + v1.z + v2.z + v3.z) / 4.0;
            quads.push(FaceQuad { verts: [v0, v1, v2, v3], color, depth });
        }
    }

    fn pick_face(&self, center: Pos2, scale: f32, mouse: Pos2) -> Option<Face> {
        // 简化的面拾取：找到 z 深度最大（最近）的外层面
        // 我们为每个外层面创建一个代表四边形，投影后检测
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
            // 背面剔除
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
