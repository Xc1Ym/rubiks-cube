pub mod view_2d;
pub mod view_3d;

use egui::{Color32, Rect};

/// 绘制一个带边框的色块矩形
pub fn draw_sticker(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    stroke_width: f32,
) {
    painter.rect_filled(rect, 2.0, color);
    painter.rect_stroke(rect, 2.0, egui::Stroke::new(stroke_width, Color32::BLACK));
}
