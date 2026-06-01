use crate::cube::moves::{parse_moves, scramble_moves, Face, Move};
use crate::cube::Cube;
use crate::renderer::view_2d::NetRenderer;
use crate::renderer::view_3d::{RotationAnim, View3D};
use egui::{Color32, RichText};

/// 动画状态 —— 支持单步旋转插值动画
#[derive(Clone, Debug)]
struct Animation {
    moves: Vec<Move>,       // 待执行的 move 队列
    current: usize,         // 当前正在播放第几个
    progress: f32,          // 当前旋转动画进度 0.0 ~ 1.0
    speed: f32,             // 每秒进度（如 4.0 表示 1/4 秒完成一步）
}

impl Animation {
    fn is_running(&self) -> bool {
        self.current < self.moves.len()
    }

    /// 每帧更新，返回是否需要持续重绘，以及当前旋转状态
    fn tick(&mut self, dt: f32) -> (bool, Option<(Move, f32)>) {
        if !self.is_running() {
            return (false, None);
        }
        self.progress += dt * self.speed;
        if self.progress >= 1.0 {
            // 当前 move 动画完成，返回 1.0 让上层 apply_move
            let mv = self.moves[self.current];
            self.progress = 1.0;
            return (true, Some((mv, 1.0)));
        }
        (true, Some((self.moves[self.current], self.progress)))
    }

    /// 当前 move 动画已应用后，推进到下一个
    fn advance(&mut self) {
        self.current += 1;
        self.progress = 0.0;
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

    /// 排队执行一个 Move（带动画）
    fn enqueue_move(&mut self, mv: Move) {
        self.push_history();
        self.start_animation(vec![mv]);
        self.status_message = format!("执行: {}", mv.notation());
    }

    /// 排队执行多个 Move（带动画）
    fn enqueue_moves(&mut self, moves: Vec<Move>) {
        if moves.is_empty() {
            return;
        }
        self.push_history();
        self.start_animation(moves);
        self.status_message = "播放动画序列...".to_string();
    }

    fn undo(&mut self) {
        self.cancel_animation();
        if self.history.len() > 1 {
            let current = self.cube.clone();
            self.redo_stack.push(current);
            self.history.pop();
            self.cube = self.history.last().unwrap().clone();
            self.status_message = "撤销".to_string();
        }
    }

    fn redo(&mut self) {
        self.cancel_animation();
        if let Some(cube) = self.redo_stack.pop() {
            self.history.push(cube.clone());
            self.cube = cube;
            self.status_message = "重做".to_string();
        }
    }

    fn reset(&mut self) {
        self.cancel_animation();
        self.push_history();
        self.cube = Cube::solved();
        self.status_message = "魔方已重置".to_string();
    }

    fn scramble(&mut self) {
        let moves = scramble_moves(self.scramble_count);
        self.push_history();
        self.start_animation(moves);
        self.status_message = format!("打乱 {} 步", self.scramble_count);
    }

    fn start_animation(&mut self, moves: Vec<Move>) {
        // speed = 1.0 / duration，duration 是当前 anim_speed（秒/步）
        let speed = 1.0 / self.anim_speed.max(0.02);
        self.animation = Some(Animation {
            moves,
            current: 0,
            progress: 0.0,
            speed,
        });
    }

    fn cancel_animation(&mut self) {
        self.animation = None;
    }

    fn run_formula(&mut self) {
        match parse_moves(&self.formula_input) {
            Ok(moves) => {
                if !moves.is_empty() {
                    self.enqueue_moves(moves);
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
        let dt = ctx.input(|i| i.stable_dt);

        // ── 动画 tick ──
        // 当前旋转动画状态（用于 3D 渲染插值）
        let mut current_rotation: Option<RotationAnim> = None;

        if let Some(ref mut anim) = self.animation {
            let (need_repaint, rot_state) = anim.tick(dt);
            if need_repaint {
                ctx.request_repaint_after(std::time::Duration::from_millis(8));
            }
            if let Some((mv, progress)) = rot_state {
                let target = if mv.double {
                    std::f32::consts::PI
                } else {
                    std::f32::consts::FRAC_PI_2
                };
                // 顺时针 = 负角度（右手定则）
                let sign = if mv.clockwise { -1.0 } else { 1.0 };
                current_rotation = Some(RotationAnim {
                    face: mv.face,
                    angle: progress * target * sign,
                });

                // 动画完成时真正提交魔方状态变更
                if progress >= 1.0 {
                    self.cube.apply_move(&mv);
                    anim.advance();
                    if anim.is_running() {
                        self.status_message =
                            format!("执行: {}", anim.moves[anim.current].notation());
                    } else {
                        self.animation = None;
                        self.status_message = "动画完成".to_string();
                    }
                }
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
                        let clicked_face = self.view_3d.show(ui, &self.cube, current_rotation);
                        // 动画播放期间禁用点击操作
                        if self.animation.is_none() {
                            if let Some(face) = clicked_face {
                                if ui.input(|i| i.modifiers.shift) {
                                    self.enqueue_move(Move::new(face, false, false));
                                } else {
                                    self.enqueue_move(Move::new(face, true, false));
                                }
                            }
                        }
                    });
                    ui.separator();
                }

                // 2D 展开图
                ui.vertical(|ui| {
                    ui.label("2D 展开图（点击色块选择面）");
                    if self.animation.is_none() {
                        if let Some(click) = self.net_renderer.show(ui, &self.cube) {
                            self.enqueue_move(Move::new(click.face, true, false));
                        }
                    } else {
                        // 动画期间只显示，不响应点击
                        self.net_renderer.show(ui, &self.cube);
                    }
                });
            });

            ui.separator();

            // 控制面板
            ui.group(|ui| {
                ui.label("控制面板");

                // 动画播放期间禁用按钮，避免状态混乱
                let busy = self.animation.is_some();

                // 手动旋转按钮
                ui.horizontal(|ui| {
                    ui.label("单步旋转:");
                    for face in Face::ALL {
                        let enabled = !busy;
                        if ui
                            .add_enabled(enabled, egui::Button::new(face.name()))
                            .clicked()
                        {
                            self.enqueue_move(Move::new(face, true, false));
                        }
                        if ui
                            .add_enabled(enabled, egui::Button::new(format!("{}'", face.name())))
                            .clicked()
                        {
                            self.enqueue_move(Move::new(face, false, false));
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("180°:");
                    for face in Face::ALL {
                        if ui
                            .add_enabled(!busy, egui::Button::new(format!("{}2", face.name())))
                            .clicked()
                        {
                            self.enqueue_move(Move::new(face, true, true));
                        }
                    }
                });

                ui.separator();

                // 公式输入
                ui.horizontal(|ui| {
                    ui.label("公式:");
                    ui.add_enabled(!busy, egui::TextEdit::singleline(&mut self.formula_input));
                    if ui.add_enabled(!busy, egui::Button::new("▶ 执行")).clicked() {
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
                    if ui.add_enabled(!busy, egui::Button::new("🔀 打乱")).clicked() {
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
