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

/// F/B/L/R 旋转时的相邻边滑动定义（被旋转面 + 相邻面边缘对滑）。
/// U/D 旋转使用条带循环滚动（band scrolling），不走这里。
fn rotation_slides(rotating: Face) -> Vec<(Face, bool, usize, Vec2)> {
    match rotating {
        Face::F => vec![
            (Face::U, true, 2, Vec2::new(0.0, 1.0)),   // U 底行向下 → F
            (Face::R, false, 0, Vec2::new(-1.0, 0.0)), // R 左列向左 → F
            (Face::D, true, 0, Vec2::new(0.0, -1.0)),  // D 顶行向上 → F
            (Face::L, false, 2, Vec2::new(1.0, 0.0)),  // L 右列向右 → F
            (Face::F, true, 0, Vec2::new(0.0, -1.0)),  // F 顶行向上 ← 远离
            (Face::F, false, 2, Vec2::new(1.0, 0.0)),  // F 右列向右 ← 远离
            (Face::F, true, 2, Vec2::new(0.0, 1.0)),   // F 底行向下 ← 远离
            (Face::F, false, 0, Vec2::new(-1.0, 0.0)), // F 左列向左 ← 远离
        ],
        Face::B => vec![
            (Face::R, false, 2, Vec2::new(1.0, 0.0)),  // R 右列向右 → B
            (Face::B, false, 0, Vec2::new(-1.0, 0.0)), // B 左列向左 ← 远离
        ],
        Face::L => vec![
            (Face::F, false, 0, Vec2::new(-1.0, 0.0)), // F 左列向左 → L
            (Face::L, false, 2, Vec2::new(1.0, 0.0)),  // L 右列向右 ← 远离
        ],
        Face::R => vec![
            (Face::F, false, 2, Vec2::new(1.0, 0.0)),  // F 右列向右 → R
            (Face::B, false, 0, Vec2::new(-1.0, 0.0)), // B 左列向左 → R
            (Face::R, false, 0, Vec2::new(-1.0, 0.0)), // R 左列向左 ← 远离
            (Face::R, false, 2, Vec2::new(1.0, 0.0)),  // R 右列向右 ← 远离
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

        let progress = rotation.map(|r| {
            let target = std::f32::consts::FRAC_PI_2;
            (r.angle.abs() / target).clamp(0.0, 1.0)
        });
        let elastic = progress.map(|p| (p * std::f32::consts::PI).sin());

        let rotating_face = rotation.map(|r| r.face);
        let u_band = rotating_face == Some(Face::U);
        let d_band = rotating_face == Some(Face::D);
        let slides = rotation.map(|r| rotation_slides(r.face));

        // ── 1) 先绘制 U/D 条带循环滚动 ──
        if u_band {
            if let Some(rot) = rotation {
                self.draw_band_row(painter, cube, rot, rect.min, total_size, true);
            }
        } else if d_band {
            if let Some(rot) = rotation {
                self.draw_band_row(painter, cube, rot, rect.min, total_size, false);
            }
        }

        // ── 2) 再绘制各个面（U/D 旋转时跳过对应行） ──
        for face in Face::ALL {
            let offset = net_offset(face, total_size);
            let is_rotating = rotating_face == Some(face);

            let face_scale = if is_rotating {
                1.0 - elastic.unwrap_or(0.0) * 0.05
            } else {
                1.0
            };

            let skip_top = u_band && face != Face::U;
            let skip_bottom = d_band && face != Face::D;

            self.draw_face(
                painter,
                rect.min + offset,
                cube,
                face,
                face_scale,
                is_rotating,
                slides.as_ref(),
                elastic,
                skip_top,
                skip_bottom,
            );
        }

        // ── 3) 绘制面标签 ──
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

    /// 绘制循环滚动的条带（U旋转=顶行, D旋转=底行）。
    ///
    /// 核心思想：把 L→F→R→B 的对应行看作一条传送带，每个贴纸保持自己的颜色，
    /// 只根据动画进度整体平移。移出边界的贴纸从另一侧绕回，形成无缝循环。
    fn draw_band_row(
        &self,
        painter: &egui::Painter,
        cube: &Cube,
        rotation: RotationAnim,
        origin: Pos2,
        total_size: f32,
        is_top: bool,
    ) {
        let sticker_step = self.sticker_size + self.gap;
        let faces = [Face::L, Face::F, Face::R, Face::B];
        let row_idx = if is_top { 0 } else { 2 };

        let band_left_x = origin.x + net_offset(Face::L, total_size).x;
        let band_width = 4.0 * total_size;

        let target = std::f32::consts::FRAC_PI_2;
        let progress = (rotation.angle.abs() / target).clamp(0.0, 1.0);
        let is_clockwise = rotation.angle < 0.0;

        // 移动方向和距离（像素）
        let shift = progress * total_size;
        let direction = match (rotation.face, is_clockwise) {
            (Face::U, true) => -1.0,   // U 顺时针：条带向左
            (Face::U, false) => 1.0,   // U 逆时针：条带向右
            (Face::D, true) => 1.0,    // D 顺时针：条带向右
            (Face::D, false) => -1.0,  // D 逆时针：条带向左
            _ => 0.0,
        };

        // 为 12 个贴纸分别计算位置并绘制
        for face_idx in 0..4 {
            let face = faces[face_idx];
            let face_origin = origin + net_offset(face, total_size);
            let y = face_origin.y + row_idx as f32 * sticker_step;

            for col in 0..3 {
                // 颜色始终保持该贴纸自身的颜色（动画前状态）
                let color = cube.get_face(face)[row_idx][col].as_egui();
                let initial_x = face_origin.x + col as f32 * sticker_step;

                // 计算当前位置（平移 + 循环 wrap）
                let raw_x = initial_x + direction * shift;
                let current_x =
                    ((raw_x - band_left_x) % band_width + band_width) % band_width + band_left_x;

                // 主绘制
                let rect = Rect::from_min_size(
                    Pos2::new(current_x, y),
                    Vec2::splat(self.sticker_size),
                );
                painter.rect_filled(rect, 2.0, color);
                painter.rect_stroke(rect, 2.0, Stroke::new(1.5, Color32::BLACK));

                // 如果 sticker 跨越 band 右边界，在左侧也画一份（保证无缝）
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

    fn draw_face(
        &self,
        painter: &egui::Painter,
        origin: Pos2,
        cube: &Cube,
        face: Face,
        face_scale: f32,
        is_rotating: bool,
        slides: Option<&Vec<(Face, bool, usize, Vec2)>>,
        elastic: Option<f32>,
        skip_top: bool,
        skip_bottom: bool,
    ) {
        let face_data = cube.get_face(face);
        let center = origin + Vec2::splat(self.sticker_size * 1.5 + self.gap);
        let slide_offset = elastic.unwrap_or(0.0) * (self.sticker_size + self.gap) * 0.5;

        let mut row_slide = [Vec2::ZERO; 3];
        let mut col_slide = [Vec2::ZERO; 3];
        if let Some(slides) = slides {
            for &(slide_face, is_row, idx, dir) in slides.iter() {
                if slide_face == face {
                    if is_row {
                        row_slide[idx] = dir * slide_offset;
                    } else {
                        col_slide[idx] = dir * slide_offset;
                    }
                }
            }
        }

        for r in 0..3 {
            if (skip_top && r == 0) || (skip_bottom && r == 2) {
                continue;
            }

            for c in 0..3 {
                let base_x = origin.x + c as f32 * (self.sticker_size + self.gap);
                let base_y = origin.y + r as f32 * (self.sticker_size + self.gap);

                let (x, y) = if face_scale != 1.0 {
                    let sticker_center = Pos2::new(
                        base_x + self.sticker_size / 2.0,
                        base_y + self.sticker_size / 2.0,
                    );
                    let dx = sticker_center.x - center.x;
                    let dy = sticker_center.y - center.y;
                    (
                        center.x + dx * face_scale - self.sticker_size / 2.0,
                        center.y + dy * face_scale - self.sticker_size / 2.0,
                    )
                } else {
                    (base_x, base_y)
                };

                let final_x = x + row_slide[r].x + col_slide[c].x;
                let final_y = y + row_slide[r].y + col_slide[c].y;

                let rect = Rect::from_min_size(
                    Pos2::new(final_x, final_y),
                    Vec2::splat(self.sticker_size * face_scale),
                );

                let color = face_data[r][c].as_egui();

                let stroke = if is_rotating {
                    let glow = (elastic.unwrap_or(0.0) * 255.0) as u8;
                    Stroke::new(
                        2.5,
                        Color32::from_rgb(255, 255, 100 + (155u8).saturating_sub(glow)),
                    )
                } else {
                    Stroke::new(1.5, Color32::BLACK)
                };

                painter.rect_filled(rect, 2.0, color);
                painter.rect_stroke(rect, 2.0, stroke);
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
