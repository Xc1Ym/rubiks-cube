use crate::cube::Cube;
use crate::cube::moves::Face;
use egui::{Color32, Pos2, Rect, Sense, Ui, Vec2};

/// 2D 十字架展开图渲染器
pub struct NetRenderer {
    pub sticker_size: f32,
    pub gap: f32,
}

impl Default for NetRenderer {
    fn default() -> Self {
        Self {
            sticker_size: 40.0,
            gap: 2.0,
        }
    }
}

/// 用户点击的面和格子位置
#[derive(Clone, Copy, Debug)]
pub struct NetClick {
    pub face: Face,
    pub row: usize,
    pub col: usize,
}

impl NetRenderer {
    /// 绘制展开图，返回可能的点击信息
    pub fn show(&self, ui: &mut Ui, cube: &Cube) -> Option<NetClick> {
        let total_size = self.sticker_size * 3.0 + self.gap * 2.0;

        // 十字架布局: 5 个面（U在中间上方实际上是 U 在中心，按标准展开）
        // 标准十字架展开:
        //       [B]
        //   [L] [U] [R]
        //       [F]
        //       [D]
        // 但为了更直观，我们采用:
        //       [U]
        //   [L] [F] [R] [B]
        //       [D]
        // 这是另一种常见展开，F 在前面更直观

        let desired = Vec2::new(total_size * 4.0 + self.gap * 3.0, total_size * 3.0 + self.gap * 2.0);
        let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());

        let painter = ui.painter();
        let mut click_result = None;

        if let Some(pos) = response.interact_pointer_pos() {
            if response.clicked() {
                if let Some(click) = self.detect_click(rect.min, pos) {
                    click_result = Some(click);
                }
            }
        }

        // 绘制各个面
        // 布局（基于 rect.min）:
        //       U 在 (total_size, 0)
        // L 在 (0, total_size), F 在 (total_size, total_size), R 在 (total_size*2, total_size), B 在 (total_size*3, total_size)
        //       D 在 (total_size, total_size*2)

        let offsets = [
            (Face::U, Vec2::new(total_size, 0.0)),
            (Face::L, Vec2::new(0.0, total_size)),
            (Face::F, Vec2::new(total_size, total_size)),
            (Face::R, Vec2::new(total_size * 2.0, total_size)),
            (Face::B, Vec2::new(total_size * 3.0, total_size)),
            (Face::D, Vec2::new(total_size, total_size * 2.0)),
        ];

        for (face, offset) in &offsets {
            self.draw_face(painter, rect.min + *offset, cube, *face);
        }

        // 绘制面标签
        for (face, offset) in &offsets {
            let label_pos = rect.min + *offset + Vec2::new(total_size / 2.0, -15.0);
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                face.name(),
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
        }

        click_result
    }

    fn draw_face(&self, painter: &egui::Painter, origin: Pos2, cube: &Cube, face: Face) {
        let face_data = cube.get_face(face);
        for r in 0..3 {
            for c in 0..3 {
                let x = origin.x + c as f32 * (self.sticker_size + self.gap);
                let y = origin.y + r as f32 * (self.sticker_size + self.gap);
                let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(self.sticker_size));
                let color = face_data[r][c].as_egui();
                super::draw_sticker(painter, rect, color, 1.5);
            }
        }
    }

    fn detect_click(&self, origin: Pos2, pos: Pos2) -> Option<NetClick> {
        let total_size = self.sticker_size * 3.0 + self.gap * 2.0;

        let faces = [
            (Face::U, Vec2::new(total_size, 0.0)),
            (Face::L, Vec2::new(0.0, total_size)),
            (Face::F, Vec2::new(total_size, total_size)),
            (Face::R, Vec2::new(total_size * 2.0, total_size)),
            (Face::B, Vec2::new(total_size * 3.0, total_size)),
            (Face::D, Vec2::new(total_size, total_size * 2.0)),
        ];

        for (face, offset) in &faces {
            let face_origin = origin + *offset;
            let face_rect = Rect::from_min_size(face_origin, Vec2::splat(total_size));
            if face_rect.contains(pos) {
                let local_x = pos.x - face_origin.x;
                let local_y = pos.y - face_origin.y;
                let col = (local_x / (self.sticker_size + self.gap)) as usize;
                let row = (local_y / (self.sticker_size + self.gap)) as usize;
                if col < 3 && row < 3 {
                    return Some(NetClick {
                        face: *face,
                        row,
                        col,
                    });
                }
            }
        }
        None
    }
}
