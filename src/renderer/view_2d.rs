use crate::cube::Cube;
use crate::cube::moves::Face;
use crate::renderer::RotationAnim;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

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

/// 用户点击的面
#[derive(Clone, Copy, Debug)]
pub struct NetClick {
    pub face: Face,
}

/// 每个面在十字架展开图中的偏移（相对于左上角）
fn net_offset(face: Face, total_size: f32) -> Vec2 {
    match face {
        Face::U => Vec2::new(total_size, 0.0),
        Face::L => Vec2::new(0.0, total_size),
        Face::F => Vec2::new(total_size, total_size),
        Face::R => Vec2::new(total_size * 2.0, total_size),
        Face::B => Vec2::new(total_size * 3.0, total_size),
        Face::D => Vec2::new(total_size, total_size * 2.0),
    }
}

/// F/B/L/R 旋转时，相邻面向被旋转面方向的挤压滑动。
/// 仅影响相邻面的边缘行/列，被旋转面自身做面内刚体旋转，不走这里。
fn edge_slides(rotating: Face) -> Vec<(Face, bool, usize, Vec2)> {
    match rotating {
        Face::F => vec![
            (Face::U, true, 2, Vec2::new(0.0, 1.0)),   // U 底行向下 → F
            (Face::R, false, 0, Vec2::new(-1.0, 0.0)), // R 左列向左 → F
            (Face::D, true, 0, Vec2::new(0.0, -1.0)),  // D 顶行向上 → F
            (Face::L, false, 2, Vec2::new(1.0, 0.0)),  // L 右列向右 → F
        ],
        Face::B => vec![
            (Face::R, false, 2, Vec2::new(1.0, 0.0)),  // R 右列向右 → B
            // B 与 U/D/L 在展开图上距离较远，不做挤压
        ],
        Face::L => vec![
            (Face::F, false, 0, Vec2::new(-1.0, 0.0)), // F 左列向左 → L
            (Face::U, true, 2, Vec2::new(-1.0, 0.0)),  // U 底行（左段）向左 → L
            (Face::D, true, 0, Vec2::new(-1.0, 0.0)),  // D 顶行（左段）向左 → L
        ],
        Face::R => vec![
            (Face::F, false, 2, Vec2::new(1.0, 0.0)),  // F 右列向右 → R
            (Face::B, false, 0, Vec2::new(-1.0, 0.0)), // B 左列向左 → R
        ],
        Face::U | Face::D => vec![],
    }
}

impl NetRenderer {
    /// 绘制展开图，返回可能的点击信息
    pub fn show(
        &self,
        ui: &mut Ui,
        cube: &Cube,
        rotation: Option<RotationAnim>,
    ) -> Option<NetClick> {
        let total_size = self.sticker_size * 3.0 + self.gap * 2.0;
        let step = self.sticker_size + self.gap;

        let desired = Vec2::new(
            total_size * 4.0 + self.gap * 3.0,
            total_size * 3.0 + self.gap * 2.0,
        );
        let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());

        let painter = ui.painter();
        let mut click_result = None;

        if let Some(pos) = response.interact_pointer_pos() {
            if response.clicked() {
                if let Some(click) = self.detect_click(rect.min, pos, total_size) {
                    click_result = Some(click);
                }
            }
        }

        let rotating_face = rotation.map(|r| r.face);
        let is_u = rotating_face == Some(Face::U);
        let is_d = rotating_face == Some(Face::D);
        let is_face_rot = rotating_face.map_or(false, |f| {
            matches!(f, Face::F | Face::B | Face::L | Face::R)
        });

        // ── 1) U/D 条带循环滚动 ──
        if is_u {
            if let Some(rot) = rotation {
                self.draw_band_row(painter, cube, rot, rect.min, total_size, step, true);
            }
        } else if is_d {
            if let Some(rot) = rotation {
                self.draw_band_row(painter, cube, rot, rect.min, total_size, step, false);
            }
        }

        // ── 2) F/B/L/R 面旋转：被旋转面做刚体旋转，相邻边挤压滑动 ──
        if is_face_rot {
            if let Some(rot) = rotation {
                let slide_offset = rot.angle.abs() / std::f32::consts::FRAC_PI_2 * step;
                let slides = edge_slides(rot.face);

                // 先绘制相邻面的挤压边块
                for face in Face::ALL {
                    if face == rot.face {
                        continue;
                    }
                    let offset = net_offset(face, total_size);
                    self.draw_face_with_slides(
                        painter,
                        rect.min + offset,
                        cube,
                        face,
                        &slides,
                        slide_offset,
                    );
                }

                // 再绘制被旋转面（面内刚体旋转）
                let offset = net_offset(rot.face, total_size);
                self.draw_rotating_face(
                    painter,
                    rect.min + offset,
                    cube,
                    rot.face,
                    rot.angle,
                );
            }
        }

        // ── 3) 绘制普通面（无动画或未被覆盖的部分） ──
        if rotating_face.is_none() {
            for face in Face::ALL {
                let offset = net_offset(face, total_size);
                self.draw_face_plain(painter, rect.min + offset, cube, face);
            }
        }

        // ── 4) 面标签 ──
        for face in Face::ALL {
            let offset = net_offset(face, total_size);
            let label_pos = rect.min + offset + Vec2::new(total_size / 2.0, -15.0);
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

    // =========================================================================
    // U/D 条带循环滚动（保持原有逻辑，颜色固定，仅位置移动）
    // =========================================================================
    fn draw_band_row(
        &self,
        painter: &egui::Painter,
        cube: &Cube,
        rotation: RotationAnim,
        origin: Pos2,
        total_size: f32,
        step: f32,
        is_top: bool,
    ) {
        let faces = [Face::L, Face::F, Face::R, Face::B];
        let row_idx = if is_top { 0 } else { 2 };

        let band_left_x = origin.x + net_offset(Face::L, total_size).x;
        let band_width = 4.0 * total_size;

        let target = std::f32::consts::FRAC_PI_2;
        let progress = (rotation.angle.abs() / target).clamp(0.0, 1.0);
        let is_clockwise = rotation.angle < 0.0;

        let shift = progress * total_size;
        let direction = match (rotation.face, is_clockwise) {
            (Face::U, true) => -1.0,
            (Face::U, false) => 1.0,
            (Face::D, true) => 1.0,
            (Face::D, false) => -1.0,
            _ => 0.0,
        };

        for face_idx in 0..4 {
            let face = faces[face_idx];
            let face_origin = origin + net_offset(face, total_size);
            let y = face_origin.y + row_idx as f32 * step;

            for col in 0..3 {
                let color = cube.get_face(face)[row_idx][col].as_egui();
                let initial_x = face_origin.x + col as f32 * step;

                let raw_x = initial_x + direction * shift;
                let current_x =
                    ((raw_x - band_left_x) % band_width + band_width) % band_width + band_left_x;

                let rect = Rect::from_min_size(
                    Pos2::new(current_x, y),
                    Vec2::splat(self.sticker_size),
                );
                painter.rect_filled(rect, 2.0, color);
                painter.rect_stroke(rect, 2.0, Stroke::new(1.5, Color32::BLACK));

                if current_x + self.sticker_size > band_left_x + band_width {
                    let wrap_rect = Rect::from_min_size(
                        Pos2::new(current_x - band_width, y),
                        Vec2::splat(self.sticker_size),
                    );
                    painter.rect_filled(wrap_rect, 2.0, color);
                    painter.rect_stroke(wrap_rect, 2.0, Stroke::new(1.5, Color32::BLACK));
                }
            }
        }
    }

    // =========================================================================
    // F/B/L/R 面内刚体旋转（颜色固定，仅位置旋转）
    // =========================================================================
    fn draw_rotating_face(
        &self,
        painter: &egui::Painter,
        origin: Pos2,
        cube: &Cube,
        face: Face,
        angle: f32, // 带符号角度，负=顺时针
    ) {
        let face_data = cube.get_face(face);
        let step = self.sticker_size + self.gap;
        let center = origin + Vec2::splat(self.sticker_size * 1.5 + self.gap);

        for r in 0..3 {
            for c in 0..3 {
                let color = face_data[r][c].as_egui();

                // 相对于面中心的偏移
                let dx = (c as f32 - 1.0) * step;
                let dy = (r as f32 - 1.0) * step;

                // 转到标准数学坐标（y向上）做旋转
                let x = dx;
                let y = -dy;

                // 旋转
                let x_rot = x * angle.cos() - y * angle.sin();
                let y_rot = x * angle.sin() + y * angle.cos();

                // 转回 egui 坐标
                let dx_rot = x_rot;
                let dy_rot = -y_rot;

                let cx = center.x + dx_rot;
                let cy = center.y + dy_rot;

                let x = cx - self.sticker_size / 2.0;
                let y = cy - self.sticker_size / 2.0;

                let rect = Rect::from_min_size(
                    Pos2::new(x, y),
                    Vec2::splat(self.sticker_size),
                );

                let stroke = Stroke::new(2.0, Color32::from_rgb(255, 255, 100));
                painter.rect_filled(rect, 2.0, color);
                painter.rect_stroke(rect, 2.0, stroke);
            }
        }
    }

    // =========================================================================
    // 相邻面挤压滑动绘制（颜色固定，仅边缘行/列平移）
    // =========================================================================
    fn draw_face_with_slides(
        &self,
        painter: &egui::Painter,
        origin: Pos2,
        cube: &Cube,
        face: Face,
        slides: &[(Face, bool, usize, Vec2)],
        slide_offset: f32,
    ) {
        let face_data = cube.get_face(face);
        let step = self.sticker_size + self.gap;

        // 计算该面上哪些行/列需要滑动
        let mut row_slide = [Vec2::ZERO; 3];
        let mut col_slide = [Vec2::ZERO; 3];
        for &(slide_face, is_row, idx, dir) in slides.iter() {
            if slide_face == face {
                if is_row {
                    row_slide[idx] = dir * slide_offset;
                } else {
                    col_slide[idx] = dir * slide_offset;
                }
            }
        }

        for r in 0..3 {
            for c in 0..3 {
                let base_x = origin.x + c as f32 * step;
                let base_y = origin.y + r as f32 * step;

                let final_x = base_x + row_slide[r].x + col_slide[c].x;
                let final_y = base_y + row_slide[r].y + col_slide[c].y;

                let rect = Rect::from_min_size(
                    Pos2::new(final_x, final_y),
                    Vec2::splat(self.sticker_size),
                );

                let color = face_data[r][c].as_egui();
                painter.rect_filled(rect, 2.0, color);
                painter.rect_stroke(rect, 2.0, Stroke::new(1.5, Color32::BLACK));
            }
        }
    }

    // =========================================================================
    // 普通面绘制（无动画）
    // =========================================================================
    fn draw_face_plain(
        &self,
        painter: &egui::Painter,
        origin: Pos2,
        cube: &Cube,
        face: Face,
    ) {
        let face_data = cube.get_face(face);
        let step = self.sticker_size + self.gap;

        for r in 0..3 {
            for c in 0..3 {
                let x = origin.x + c as f32 * step;
                let y = origin.y + r as f32 * step;

                let rect = Rect::from_min_size(
                    Pos2::new(x, y),
                    Vec2::splat(self.sticker_size),
                );

                let color = face_data[r][c].as_egui();
                painter.rect_filled(rect, 2.0, color);
                painter.rect_stroke(rect, 2.0, Stroke::new(1.5, Color32::BLACK));
            }
        }
    }

    fn detect_click(&self, origin: Pos2, pos: Pos2, total_size: f32) -> Option<NetClick> {
        for face in Face::ALL {
            let offset = net_offset(face, total_size);
            let face_origin = origin + offset;
            let face_rect = Rect::from_min_size(face_origin, Vec2::splat(total_size));
            if face_rect.contains(pos) {
                return Some(NetClick { face });
            }
        }
        None
    }
}
