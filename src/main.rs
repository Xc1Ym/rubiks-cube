mod app;
mod cube;
mod renderer;

use app::CubeApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Rust 魔方演示程序"),
        ..Default::default()
    };

    eframe::run_native(
        "Rust 魔方",
        options,
        Box::new(|cc| Ok(Box::new(CubeApp::new(cc)))),
    )
}
