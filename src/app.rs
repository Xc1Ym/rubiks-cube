use crate::cube::moves::{parse_moves, scramble_moves, Face, Move};
use crate::cube::Cube;
use crate::renderer::view_2d::NetRenderer;
use crate::renderer::view_3d::View3D;
use egui::{Color32, RichText};

/// 动画状态
#[derive(Clone, Debug)]
struct Animation {
    moves: Vec<Move>,
    current_index: usize,
    timer: f32,
    speed: f32, // 秒/步
}

impl Animation {
    fn is_running(&self) -> bool {
        self.current_index < self.moves.len()
    }

    fn tick(&mut self, dt: f32) -> Option<Move> {
        if !self.is_running() {
            return None;
        }
        self.timer += dt;
        if self.timer >= self.speed {
            self.timer -= self.speed;
            let mv = self.moves[self.current_index];
            self.current_index += 1;
            Some(mv)
        } else {
            None
        }
    }
}

pub struct CubeApp {
    cube: Cube,
    history: Vec<Cube>, // 用于撤销
    redo_stack: Vec<Cube>,

    // 渲染器
    net_renderer: NetRenderer,
    view_3d: View3D,

    // 动画
    animation: Option<Animation>,

    // UI 状态
    formula_input: String,
    scramble_count: usize,
    status_message: String,
    show_3d: bool,
    anim_speed: f32,
}

impl Default for CubeApp {
    fn default() -> Self {
        Self {
            cube: Cube::solved(),
            history: vec![Cube::solved()],
            redo_stack: Vec::new(),
            net_renderer: NetRenderer::default(),
            view_3d: View3D::default(),
            animation: None,
            formula_input: String::new(),
            scramble_count: 20,
            status_message: "魔方已重置".to_string(),
            show_3d: true,
            anim_speed: 0.3,
        }
    }
}

impl CubeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 加载系统中文字体
        load_chinese_font(&cc.egui_ctx);
        Self::default()
    }

    fn push_history(&mut self) {
        self.history.push(self.cube.clone());
        self.redo_stack.clear();
    }

    fn apply_move(&mut self, mv: Move) {
        self.push_history();
        self.cube.apply_move(&mv);
        self.status_message = format!("执行: {}", mv.notation());
    }

    fn apply_moves(&mut self, moves: Vec<Move>) {
        if moves.is_empty() {
            return;
        }
        self.push_history();
        self.cube.apply_moves(&moves);
        let notations: Vec<String> = moves.iter().map(|m| m.notation()).collect();
        self.status_message = format!("执行序列: {}", notations.join(" "));
    }

    fn undo(&mut self) {
        if self.history.len() > 1 {
            let current = self.cube.clone();
            self.redo_stack.push(current);
            self.history.pop();
            self.cube = self.history.last().unwrap().clone();
            self.status_message = "撤销".to_string();
        }
    }

    fn redo(&mut self) {
        if let Some(cube) = self.redo_stack.pop() {
            self.history.push(cube.clone());
            self.cube = cube;
            self.status_message = "重做".to_string();
        }
    }

    fn reset(&mut self) {
        self.push_history();
        self.cube = Cube::solved();
        self.status_message = "魔方已重置".to_string();
    }

    fn scramble(&mut self) {
        let moves = scramble_moves(self.scramble_count);
        self.start_animation(moves);
        self.status_message = format!("打乱 {} 步", self.scramble_count);
    }

    fn start_animation(&mut self, moves: Vec<Move>) {
        self.animation = Some(Animation {
            moves,
            current_index: 0,
            timer: 0.0,
            speed: self.anim_speed,
        });
    }

    fn run_formula(&mut self) {
        match parse_moves(&self.formula_input) {
            Ok(moves) => {
                if !moves.is_empty() {
                    self.start_animation(moves);
                    self.status_message = "播放公式...".to_string();
                }
            }
            Err(e) => {
                self.status_message = format!("公式错误: {}", e);
            }
        }
    }
}

impl eframe::App for CubeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理动画
        if let Some(ref mut anim) = self.animation {
            if anim.is_running() {
                ctx.request_repaint_after(std::time::Duration::from_millis(16));
            }
        }

        // 动画 tick
        let dt = ctx.input(|i| i.stable_dt);
        if let Some(ref mut anim) = self.animation {
            if let Some(mv) = anim.tick(dt) {
                self.cube.apply_move(&mv);
            }
            if !anim.is_running() {
                self.animation = None;
                self.status_message = "动画完成".to_string();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎲 Rust 魔方");
                ui.separator();
                ui.checkbox(&mut self.show_3d, "显示 3D 视图");
            });

            ui.separator();

            // 状态信息
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.status_message).color(Color32::LIGHT_BLUE));
            });

            ui.separator();

            // 主区域：3D 视图 + 2D 展开图
            ui.horizontal(|ui| {
                // 3D 视图
                if self.show_3d {
                    ui.vertical(|ui| {
                        ui.label("3D 视图（拖拽旋转视角，点击面旋转）");
                        let clicked_face = self.view_3d.show(ui, &self.cube);
                        if let Some(face) = clicked_face {
                            if ui.input(|i| i.modifiers.shift) {
                                self.apply_move(Move::new(face, false, false));
                            } else {
                                self.apply_move(Move::new(face, true, false));
                            }
                        }
                    });
                    ui.separator();
                }

                // 2D 展开图
                ui.vertical(|ui| {
                    ui.label("2D 展开图（点击色块选择面）");
                    if let Some(click) = self.net_renderer.show(ui, &self.cube) {
                        // 点击展开图时，根据点击位置决定旋转方向
                        // 简化：点击直接顺时针旋转该面
                        self.apply_move(Move::new(click.face, true, false));
                    }
                });
            });

            ui.separator();

            // 控制面板
            ui.group(|ui| {
                ui.label("控制面板");

                // 手动旋转按钮
                ui.horizontal(|ui| {
                    ui.label("单步旋转:");
                    for face in Face::ALL {
                        if ui.button(format!("{}", face.name())).clicked() {
                            self.apply_move(Move::new(face, true, false));
                        }
                        if ui.button(format!("{}'", face.name())).clicked() {
                            self.apply_move(Move::new(face, false, false));
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("180°:");
                    for face in Face::ALL {
                        if ui.button(format!("{}2", face.name())).clicked() {
                            self.apply_move(Move::new(face, true, true));
                        }
                    }
                });

                ui.separator();

                // 公式输入
                ui.horizontal(|ui| {
                    ui.label("公式:");
                    ui.text_edit_singleline(&mut self.formula_input);
                    if ui.button("▶ 执行").clicked() {
                        self.run_formula();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("动画速度:");
                    ui.add(egui::Slider::new(&mut self.anim_speed, 0.05..=1.0).text("秒/步"));
                });

                ui.separator();

                // 功能按钮
                ui.horizontal(|ui| {
                    if ui.button("⏮ 撤销").clicked() {
                        self.undo();
                    }
                    if ui.button("⏭ 重做").clicked() {
                        self.redo();
                    }
                    if ui.button("🔄 重置").clicked() {
                        self.reset();
                    }
                    ui.separator();
                    ui.label("打乱步数:");
                    ui.add(egui::DragValue::new(&mut self.scramble_count).range(1..=100));
                    if ui.button("🔀 打乱").clicked() {
                        self.scramble();
                    }
                });
            });

            ui.separator();

            // 使用说明
            ui.collapsing("📖 使用说明", |ui| {
                ui.label("• 3D 视图: 拖拽旋转视角，点击面进行顺时针旋转，Shift+点击逆时针");
                ui.label("• 2D 展开图: 直接点击任意面执行顺时针旋转");
                ui.label("• 公式语法: U D F B L R 加 ' 表示逆时针，加 2 表示180°，如 R U R' U'");
                ui.label("• 常用公式示例:");
                ui.label("  - 顶层十字: F R U R' U' F'");
                ui.label("  - 三循环: R U R' U'");
            });
        });
    }
}

/// 尝试从系统路径加载中文字体并注入 egui
fn load_chinese_font(ctx: &egui::Context) {
    let font_paths: [&str; 8] = [
        // macOS
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        "/System/Library/Fonts/STHeiti Light.ttc",
        // Linux
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        // Windows
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simhei.ttf",
    ];

    for path in &font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            // 将中文字体作为 "chinese" 字体源插入
            fonts.font_data.insert(
                "chinese".to_owned(),
                egui::FontData::from_owned(font_data).into(),
            );
            // 将 "chinese" 添加到 Proportional 和 Monospace 的 fallback 中
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("chinese".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("chinese".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }

    // 若都未找到，打印警告但程序仍继续运行（英文可正常显示）
    eprintln!("警告: 未找到系统中文字体文件，中文可能显示为方框");
    eprintln!("请尝试安装 Noto Sans CJK 或 WenQuanYi ZenHei 字体");
}
