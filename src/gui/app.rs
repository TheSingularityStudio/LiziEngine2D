use std::sync::Arc;
use eframe::egui;
use egui::ColorImage;
use egui::TextureHandle;
use egui::load::SizedTexture;
use egui::menu;

use crate::gui::interaction::{InteractionState, ToolMode};
use crate::core::sim::ElectrostaticSim2D;
use crate::presets::PresetVariant;
use crate::visual::colors::heatmap_rgb;

/// 尝试加载中文字体，返回是否成功加载
fn load_chinese_fonts(fonts: &mut egui::FontDefinitions) -> bool {
    // 尝试多个常见中文字体路径
    let font_candidates = [
        "C:\\Windows\\Fonts\\msyh.ttc",    // 微软雅黑
        "C:\\Windows\\Fonts\\simhei.ttf",   // 黑体
        "C:\\Windows\\Fonts\\simsun.ttc",   // 宋体
        "C:\\Windows\\Fonts\\yahei.ttf",    // 微软雅黑（变体）
        "C:\\Windows\\Fonts\\msyhbd.ttc",   // 微软雅黑粗体
    ];

    for path in &font_candidates {
        if let Ok(data) = std::fs::read(path) {
            let name = format!("chinese_{}", fonts.font_data.len());
            fonts.font_data.insert(
                name.clone(),
                Arc::new(egui::FontData::from_owned(data)),
            );
            // 添加到 Proportional 字体栈顶部
            if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                proportional.insert(0, name.clone());
            }
            if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                monospace.insert(0, name);
            }
            return true;
        }
    }
    false
}

/// 模拟运行状态
struct SimulationState {
    variant: PresetVariant,
    sim: ElectrostaticSim2D,
    paused: bool,
    step_count: usize,
    /// 平滑后的 V 范围
    v_min: f64,
    v_max: f64,
    /// 交互状态
    interaction: InteractionState,
    /// 缓存的纹理句柄（避免每帧重新创建）
    heatmap_texture: Option<TextureHandle>,
    /// 面板可见性
    show_left_panel: bool,
    show_right_panel: bool,
    /// 显示热力图
    show_heatmap: bool,
    /// 弹窗状态
    show_about_dialog: bool,
    show_shortcuts_dialog: bool,
}

/// LiziEngine2D 主 GUI 应用
pub struct LiziApp {
    /// None = 预设选择界面, Some = 模拟界面
    state: Option<SimulationState>,
}

impl Default for LiziApp {
    fn default() -> Self {
        Self { state: None }
    }
}

impl LiziApp {
    /// 启动 GUI 应用
    pub fn run() {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1100.0, 700.0]),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "LiziEngine2D - 静电 PIC 模拟器",
            native_options,
            Box::new(|cc| {
                // 配置中文字体支持（动态查找系统字体）
                let mut fonts = egui::FontDefinitions::default();
                if !load_chinese_fonts(&mut fonts) {
                    eprintln!("警告: 未找到中文字体，中文可能显示为方框");
                }
                cc.egui_ctx.set_fonts(fonts);
                Ok(Box::new(LiziApp::default()))
            }),
        );
    }

    /// 渲染预设选择界面
    fn render_preset_selection(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.heading("LiziEngine2D");
                ui.label("静电 PIC 模拟器");
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(20.0);

                ui.label("选择一个预设场景开始模拟：");
                ui.add_space(10.0);

                for variant in PresetVariant::all() {
                    let name = variant.display_name();
                    let desc = variant.description();

                    let frame = egui::Frame::group(ui.style());
                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());
                            if ui.button(name).clicked() {
                                let sim = variant.create_sim();
                                self.state = Some(SimulationState {
                                    variant: *variant,
                                    sim,
                                    paused: false,
                                    step_count: 0,
                                    v_min: 0.0,
                                    v_max: 1.0,
                                    interaction: InteractionState::new(),
                                    heatmap_texture: None,
                                    show_left_panel: true,
                                    show_right_panel: true,
                                    show_heatmap: true,
                                    show_about_dialog: false,
                                    show_shortcuts_dialog: false,
                                });
                            }
                            ui.add_space(10.0);
                            ui.label(desc);
                        });
                    });
                    ui.add_space(5.0);
                }
            });
        });
    }
}

/// 渲染菜单栏
fn render_menu_bar(ctx: &egui::Context, state: &mut SimulationState) -> bool {
    let mut back_requested = false;

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        menu::bar(ui, |ui| {
            // ---- 文件菜单 ----
            ui.menu_button("文件", |ui| {
                if ui.button("返回预设选择").clicked() {
                    back_requested = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("退出").clicked() {
                    std::process::exit(0);
                }
            });

            // ---- 选项菜单 ----
            ui.menu_button("选项", |ui| {
                let mut show_left = state.show_left_panel;
                if ui.checkbox(&mut show_left, "显示工具面板").changed() {
                    state.show_left_panel = show_left;
                }
                let mut show_right = state.show_right_panel;
                if ui.checkbox(&mut show_right, "显示参数面板").changed() {
                    state.show_right_panel = show_right;
                }
                ui.separator();
                let mut show_heatmap = state.show_heatmap;
                if ui.checkbox(&mut show_heatmap, "显示热力图").changed() {
                    state.show_heatmap = show_heatmap;
                }
            });

            // ---- 帮助菜单 ----
            ui.menu_button("帮助", |ui| {
                if ui.button("关于 LiziEngine2D").clicked() {
                    state.show_about_dialog = true;
                    ui.close_menu();
                }
                if ui.button("快捷键说明").clicked() {
                    state.show_shortcuts_dialog = true;
                    ui.close_menu();
                }
            });

            // ---- 右侧控制按钮（在菜单栏同一行） ----
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 模拟信息
                ui.label(format!("预设: {}", state.variant.display_name()));
                ui.label(format!("步数: {}", state.step_count));
                if let Some(v) = state.sim.v.as_ref() {
                    let actual_min = v.iter().cloned().fold(f64::MAX, f64::min);
                    let actual_max = v.iter().cloned().fold(f64::MIN, f64::max);
                    ui.label(format!("V: [{:.2e}, {:.2e}]", actual_min, actual_max));
                }
                ui.label(format!("粒子数: {}", state.sim.particles.len()));
                ui.separator();

                if ui.button("⟳ Reset").clicked() {
                    let new_sim = state.variant.create_sim();
                    state.sim = new_sim;
                    state.step_count = 0;
                    state.paused = false;
                    state.v_min = 0.0;
                    state.v_max = 1.0;
                    state.interaction = InteractionState::new();
                }
                if ui.button("⏭ Step").clicked() {
                    state.paused = true;
                    let dt = state.variant.config().dt;
                    state.sim.step(dt);
                    state.step_count += 1;
                }
                if state.paused {
                    if ui.button("▶ Play").clicked() {
                        state.paused = false;
                    }
                } else {
                    if ui.button("⏸ Pause").clicked() {
                        state.paused = true;
                    }
                }

                if ui.button("← 返回").clicked() {
                    back_requested = true;
                }
            });
        });
    });

    back_requested
}

/// 渲染对话框
fn render_dialogs(ctx: &egui::Context, state: &mut SimulationState) {
    // 关于对话框
    if state.show_about_dialog {
        egui::Window::new("关于 LiziEngine2D")
            .open(&mut state.show_about_dialog)
            .resizable(false)
            .default_size([420.0, 280.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("LiziEngine2D");
                    ui.label("版本 0.1.0");
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label("二维静电 PIC (Particle-in-Cell) 模拟器");
                    ui.label("使用 Rust + egui 实现");
                    ui.add_space(8.0);
                    ui.hyperlink_to("GitHub 仓库", "https://github.com/TheSingularityStudio/LiziEngine2D");
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label("技术栈：");
                    ui.label("  • eframe/egui — GUI 框架");
                    ui.label("  • ndarray — 数值计算");
                    ui.label("  • ndrustfft — FFT Poisson 求解器");
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label("许可证：MIT");
                });
            });
    }

    // 快捷键说明对话框
    if state.show_shortcuts_dialog {
        egui::Window::new("快捷键说明")
            .open(&mut state.show_shortcuts_dialog)
            .resizable(false)
            .default_size([380.0, 300.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("工具模式");
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label("左侧面板选择三种工具：");
                    ui.label("  • 拖动粒子 — 点击选中粒子并拖拽移动");
                    ui.label("  • 放置粒子 — 点击画布空白处创建新粒子");
                    ui.label("  • 删除粒子 — 点击粒子将其删除");
                    ui.add_space(8.0);

                    ui.heading("画布操作");
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label("  • 鼠标左键 — 根据当前工具执行操作");
                    ui.label("  • 鼠标拖拽 — 在\"拖动粒子\"模式下移动粒子");
                    ui.label("  • 面板显示/隐藏 — 在\"选项\"菜单中控制");
                    ui.add_space(8.0);

                    ui.heading("模拟控制");
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label("  • ▶ Play — 启动自动步进模拟");
                    ui.label("  • ⏸ Pause — 暂停模拟");
                    ui.label("  • ⏭ Step — 单步执行一个时间步");
                    ui.label("  • ⟳ Reset — 重置到初始状态");
                    ui.label("  • ← 返回 — 返回预设选择界面");
                    ui.add_space(8.0);

                    ui.heading("菜单栏");
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label("  • 文件 → 返回预设选择 / 退出");
                    ui.label("  • 选项 → 显示/隐藏面板和热力图");
                    ui.label("  • 帮助 → 关于 / 快捷键说明");
                });
            });
    }
}

/// 渲染左侧工具选择面板
fn render_left_panel(ctx: &egui::Context, state: &mut SimulationState) {
    if !state.show_left_panel { return; }
    let interaction = &mut state.interaction;
    egui::SidePanel::left("tool_panel")
        .resizable(false)
        .default_width(140.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(8.0);
                ui.heading("工具");
                ui.separator();
                ui.add_space(4.0);

                for tool in ToolMode::all() {
                    let is_selected = interaction.tool_mode == tool;
                    let text = tool.display_name();

                    let button = if is_selected {
                        egui::Button::new(text)
                            .fill(ui.style().visuals.selection.bg_fill)
                            .min_size(egui::vec2(120.0, 32.0))
                    } else {
                        egui::Button::new(text)
                            .min_size(egui::vec2(120.0, 32.0))
                    };

                    if ui.add(button).clicked() {
                        interaction.tool_mode = tool;
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label("快捷操作提示：");
                ui.label("拖拽可选择粒子");
                ui.label("点击画布执行操作");
            });
        });
}

/// 渲染右侧参数调整面板
fn render_right_panel(ctx: &egui::Context, state: &mut SimulationState) {
    if !state.show_right_panel { return; }
    let interaction = &mut state.interaction;
    egui::SidePanel::right("param_panel")
        .resizable(false)
        .default_width(180.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(8.0);
                ui.heading("参数");
                ui.separator();
                ui.add_space(8.0);

                // 显示当前工具
                ui.horizontal(|ui| {
                    ui.label("当前工具：");
                    ui.label(interaction.tool_mode.display_name());
                });

                ui.add_space(8.0);

                // 根据工具模式显示不同参数
                match interaction.tool_mode {
                    ToolMode::DragParticle => {
                        ui.label("拖动粒子");
                        ui.add_space(4.0);
                        ui.label("拖拽粒子改变其位置。");
                        ui.label("选中时粒子速度归零。");

                        ui.add_space(8.0);
                        ui.label("选择半径：");
                        ui.add(egui::Slider::new(&mut interaction.selection_radius, 0.01..=0.20)
                            .text("归一化")
                            .step_by(0.005));
                    }
                    ToolMode::PlaceParticle => {
                        ui.label("放置粒子参数：");
                        ui.add_space(4.0);

                        // 电荷量
                        ui.horizontal(|ui| {
                            ui.set_min_width(60.0);
                            ui.label("电荷量：");
                        });
                        ui.add(egui::Slider::new(&mut interaction.place_params.charge, -10.0..=10.0)
                            .text("q")
                            .step_by(0.1));

                        // 固定粒子选项
                        ui.add_space(4.0);
                        ui.checkbox(&mut interaction.place_params.fixed, "固定粒子（速度=0）");

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);
                        ui.label("在画布上点击放置粒子。");
                        if interaction.place_params.fixed {
                            ui.label("固定粒子不会移动。");
                        }
                    }
                    ToolMode::DeleteParticle => {
                        ui.label("删除粒子");
                        ui.add_space(4.0);
                        ui.label("点击粒子将其删除。");

                        ui.add_space(8.0);
                        ui.label("选择半径：");
                        ui.add(egui::Slider::new(&mut interaction.selection_radius, 0.01..=0.20)
                            .text("归一化")
                            .step_by(0.005));
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // 模拟参数显示
                ui.label("模拟状态：");

                let dt = state.variant.config().dt;
                ui.label(format!("dt = {:.2e}", dt));
                if state.paused {
                    ui.label("⏸ 已暂停");
                } else {
                    ui.label("▶ 运行中");
                }
            });
        });
}

/// 渲染模拟界面（菜单栏 + 侧边栏 + 中央画布）
/// 返回 false 表示用户点击了"返回"按钮
fn render_simulation_panels(ctx: &egui::Context, state: &mut SimulationState) -> bool {
    // 先渲染菜单栏（返回菜单栏中的返回请求）
    let back_flag = render_menu_bar(ctx, state);

    // 渲染对话框
    render_dialogs(ctx, state);

    // 渲染左侧工具面板
    render_left_panel(ctx, state);

    // 渲染右侧参数面板
    render_right_panel(ctx, state);

    // 渲染中央画布
    render_central_canvas(ctx, state);

    // 非暂停时自动执行模拟步进
    if !state.paused {
        let dt = state.variant.config().dt;
        state.sim.step(dt);
        state.step_count += 1;
        ctx.request_repaint();
    }

    !back_flag
}

/// 渲染中央画布（热力图 + 粒子）
fn render_central_canvas(ctx: &egui::Context, state: &mut SimulationState) {
    let sim = &mut state.sim;
    let v_min = &mut state.v_min;
    let v_max = &mut state.v_max;
    let interaction = &mut state.interaction;

    egui::CentralPanel::default().show(ctx, |ui| {
        // 确保场已计算
        if sim.v.is_none() || sim.ex.is_none() || sim.ey.is_none() {
            sim.compute_fields();
        }

        let snapshot = sim.get_state_snapshot();
        let (nx, ny) = snapshot.v.dim();

        // 更新 V 范围（平滑过渡）— 无论是否显示热力图都计算，因为粒子交互需要
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for val in snapshot.v.iter() {
            if *val < min {
                min = *val;
            }
            if *val > max {
                max = *val;
            }
        }
        *v_min = *v_min * 0.9 + min * 0.1;
        *v_max = *v_max * 0.9 + max * 0.1;
        if (*v_max - *v_min).abs() < 1e-12 {
            *v_max = *v_min + 1.0;
        }

        // 居中放置正方形图像区域
        let avail = ui.available_rect_before_wrap();
        let max_edge = avail.size().x.min(avail.size().y);
        let center = avail.center();
        let image_rect = egui::Rect::from_center_size(center, egui::vec2(max_edge, max_edge));

        let texture_rect: egui::Rect;

        if state.show_heatmap {
            // 构建热力图像素数据
            let mut pixels = Vec::with_capacity(nx * ny);
            for j in (0..ny).rev() {
                for i in 0..nx {
                    let val = snapshot.v[[i, j]];
                    let (r, g, b) = heatmap_rgb(val, *v_min, *v_max);
                    pixels.push(egui::Color32::from_rgb(r, g, b));
                }
            }

            let color_image = ColorImage {
                size: [nx, ny],
                pixels,
            };

            // 复用缓存的纹理句柄
            let texture = state.heatmap_texture.get_or_insert_with(|| {
                ctx.load_texture("heatmap", color_image.clone(), egui::TextureOptions::NEAREST)
            });
            texture.set(color_image, egui::TextureOptions::NEAREST);

            // 绘制热力图并记录实际渲染区域
            let response = ui.put(
                image_rect,
                egui::Image::from_texture(SizedTexture::from(&*texture))
                    .fit_to_exact_size(egui::vec2(max_edge, max_edge)),
            );
            texture_rect = response.rect;
        } else {
            // 不显示热力图：在画布区域绘制纯色背景
            texture_rect = image_rect;
            let bg_color = ui.style().visuals.panel_fill;
            ui.painter().rect_filled(image_rect, 0.0, bg_color);
        }

        // 绘制粒子
        let painter = ui.painter();
        let particle_count = snapshot.x.len();
        let lx = if snapshot.lx <= 0.0 { 1.0 } else { snapshot.lx };
        let ly = if snapshot.ly <= 0.0 { 1.0 } else { snapshot.ly };

        for p in 0..particle_count {
            let nx_p = (((snapshot.x[p] / lx) * nx as f64 + 0.5) / nx as f64).clamp(0.0, 1.0);
            let ny_p = (((snapshot.y[p] / ly) * ny as f64 + 0.5) / ny as f64).clamp(0.0, 1.0);

            let sx = texture_rect.left() + nx_p as f32 * texture_rect.width();
            let sy = texture_rect.bottom() - ny_p as f32 * texture_rect.height();

            let color = if snapshot.q[p] < 0.0 {
                egui::Color32::CYAN
            } else {
                egui::Color32::WHITE
            };

            painter.circle_filled(egui::pos2(sx, sy), 3.0, color);
        }

        // 处理鼠标交互
        handle_mouse_interaction(
            ui, sim, interaction, texture_rect, nx, ny, lx, ly,
        );
    });
}

/// 根据当前工具模式处理鼠标交互
fn handle_mouse_interaction(
    ui: &egui::Ui,
    sim: &mut ElectrostaticSim2D,
    interaction: &mut InteractionState,
    texture_rect: egui::Rect,
    grid_nx: usize,
    grid_ny: usize,
    lx: f64,
    ly: f64,
) {
    // 检查鼠标是否在画布区域内
    let mouse_pos = ui.input(|i| i.pointer.hover_pos());
    let mouse_down = ui.input(|i| i.pointer.any_down());
    let mouse_clicked = ui.input(|i| i.pointer.any_click());

    let Some(pos) = mouse_pos else { return };
    if !texture_rect.contains(pos) {
        // 鼠标移出画布时取消任何交互
        interaction.dragging = false;
        interaction.dragged_particle_index = None;
        return;
    }

    // 归一化鼠标位置到 [0,1]（纹理坐标系）
    let tex_u = ((pos.x - texture_rect.left()) / texture_rect.width()).clamp(0.0f32, 1.0f32);
    let tex_v = ((texture_rect.bottom() - pos.y) / texture_rect.height()).clamp(0.0f32, 1.0f32);

    // 鼠标在热力图像素中心的视觉坐标 → 逆变换回世界坐标
    let inv_nx = grid_nx as f64;
    let inv_ny = grid_ny as f64;
    let tex_u_f64 = tex_u as f64;
    let tex_v_f64 = tex_v as f64;
    let world_x = ((tex_u_f64 * inv_nx - 0.5) / inv_nx).clamp(0.0, 1.0) * lx;
    let world_y = ((tex_v_f64 * inv_ny - 0.5) / inv_ny).clamp(0.0, 1.0) * ly;

    match interaction.tool_mode {
        ToolMode::DragParticle => {
            handle_drag_interaction(
                ui, sim, interaction, texture_rect, grid_nx, grid_ny, lx, ly,
                pos, tex_u, tex_v, tex_u_f64, tex_v_f64, world_x, world_y,
                mouse_down,
            );
            // 拖动粒子时标记场已过期
            if interaction.dragging && mouse_down {
                sim.v = None;
                sim.ex = None;
                sim.ey = None;
            }
        }
        ToolMode::PlaceParticle => {
            if mouse_clicked {
                let charge = interaction.place_params.charge;
                sim.particles.add_particle(world_x, world_y, charge, 0.0, 0.0);
                sim.v = None;
                sim.ex = None;
                sim.ey = None;
                ui.ctx().request_repaint();
            }
        }
        ToolMode::DeleteParticle => {
            if mouse_clicked {
                let particle_visual_u: Vec<f64> = sim.particles.x.iter()
                    .map(|&x| (((x / lx) * inv_nx + 0.5) / inv_nx).clamp(0.0, 1.0))
                    .collect();
                let particle_visual_v: Vec<f64> = sim.particles.y.iter()
                    .map(|&y| (((y / ly) * inv_ny + 0.5) / inv_ny).clamp(0.0, 1.0))
                    .collect();

                let mut min_dist = f64::MAX;
                let mut min_index = None;
                for i in 0..sim.particles.len() {
                    let dx = particle_visual_u[i] - tex_u as f64;
                    let dy = particle_visual_v[i] - tex_v as f64;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < min_dist {
                        min_dist = dist;
                        min_index = Some(i);
                    }
                }
                if let Some(idx) = min_index {
                    if min_dist <= interaction.selection_radius {
                        sim.particles.remove_particle(idx);
                        sim.v = None;
                        sim.ex = None;
                        sim.ey = None;
                        ui.ctx().request_repaint();
                    }
                }
            }
        }
    }

    if !mouse_down {
        interaction.dragging = false;
        interaction.dragged_particle_index = None;
    }
}

/// 处理拖动粒子交互
fn handle_drag_interaction(
    _ui: &egui::Ui,
    sim: &mut ElectrostaticSim2D,
    interaction: &mut InteractionState,
    _texture_rect: egui::Rect,
    grid_nx: usize,
    grid_ny: usize,
    lx: f64,
    ly: f64,
    _pos: egui::Pos2,
    tex_u: f32,
    tex_v: f32,
    _tex_u_f64: f64,
    _tex_v_f64: f64,
    world_x: f64,
    world_y: f64,
    mouse_down: bool,
) {
    let inv_nx = grid_nx as f64;
    let inv_ny = grid_ny as f64;

    if mouse_down {
        if !interaction.dragging {
            let particle_visual_u: Vec<f64> = sim.particles.x.iter()
                .map(|&x| (((x / lx) * inv_nx + 0.5) / inv_nx).clamp(0.0, 1.0))
                .collect();
            let particle_visual_v: Vec<f64> = sim.particles.y.iter()
                .map(|&y| (((y / ly) * inv_ny + 0.5) / inv_ny).clamp(0.0, 1.0))
                .collect();

            let mut min_dist = f64::MAX;
            let mut min_index = None;
            for i in 0..sim.particles.len() {
                let dx = particle_visual_u[i] - tex_u as f64;
                let dy = particle_visual_v[i] - tex_v as f64;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < min_dist {
                    min_dist = dist;
                    min_index = Some(i);
                }
            }
            if let Some(idx) = min_index {
                if min_dist <= interaction.selection_radius {
                    interaction.dragging = true;
                    interaction.dragged_particle_index = Some(idx);
                }
            }
        }

        if let Some(idx) = interaction.dragged_particle_index {
            sim.particles.x[idx] = world_x;
            sim.particles.y[idx] = world_y;
            sim.particles.vx[idx] = 0.0;
            sim.particles.vy[idx] = 0.0;
        }
    } else {
        interaction.dragging = false;
        interaction.dragged_particle_index = None;
    }
}

impl eframe::App for LiziApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(ref mut state) = self.state {
            let running = render_simulation_panels(ctx, state);
            if !running {
                self.state = None;
            } else if !state.paused {
                ctx.request_repaint();
            }
        } else {
            self.render_preset_selection(ctx);
        }
    }
}