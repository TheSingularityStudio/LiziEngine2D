use std::cell::Cell;
use std::sync::Arc;
use eframe::egui;
use egui::ColorImage;
use egui::TextureHandle;
use egui::load::SizedTexture;

use crate::gui::interaction::InteractionState;
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
                .with_inner_size([800.0, 700.0]),
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

/// 渲染模拟界面（top panel + central panel）
/// 返回 false 表示用户点击了"返回"按钮
fn render_simulation_panels(ctx: &egui::Context, state: &mut SimulationState) -> bool {
    let back_flag = Cell::new(false);
    let variant = state.variant;
    let sim = &mut state.sim;
    let paused = &mut state.paused;
    let step_count = &mut state.step_count;
    let v_min = &mut state.v_min;
    let v_max = &mut state.v_max;
    let interaction = &mut state.interaction;

    // 顶部控制面板
    egui::TopBottomPanel::top("sim_controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // 返回按钮（使用 back_flag Cell 标记）
            if ui.button("← 返回").clicked() {
                back_flag.set(true);
                return;
            }

            ui.separator();

            // Play/Pause 按钮
            if *paused {
                if ui.button("▶ Play").clicked() {
                    *paused = false;
                }
            } else {
                if ui.button("⏸ Pause").clicked() {
                    *paused = true;
                }
            }

            // Step 按钮（单步执行）
            if ui.button("⏭ Step").clicked() {
                *paused = true;
                let dt = variant.config().dt;
                sim.step(dt);
                *step_count += 1;
            }

            // Reset 按钮
            if ui.button("⟳ Reset").clicked() {
                let new_sim = variant.create_sim();
                *sim = new_sim;
                *step_count = 0;
                *paused = false;
                *v_min = 0.0;
                *v_max = 1.0;
                *interaction = InteractionState::new();
            }

            ui.separator();

            // 显示模拟信息
            ui.label(format!("预设: {}", variant.display_name()));
            ui.label(format!("步数: {}", step_count));
            if let Some(v) = sim.v.as_ref() {
                let actual_min = v.iter().cloned().fold(f64::MAX, f64::min);
                let actual_max = v.iter().cloned().fold(f64::MIN, f64::max);
                ui.label(format!("V: [{:.2e}, {:.2e}]", actual_min, actual_max));
            }
            ui.label(format!("粒子数: {}", sim.particles.len()));
        });
    });

    if back_flag.get() {
        return false;
    }

    // 中央画布（热力图 + 粒子）
    egui::CentralPanel::default().show(ctx, |ui| {
        // 确保场已计算
        if sim.v.is_none() || sim.ex.is_none() || sim.ey.is_none() {
            sim.compute_fields();
        }

        let snapshot = sim.get_state_snapshot();
        let (nx, ny) = snapshot.v.dim();

        // 更新 V 范围（平滑过渡）
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

        // 构建像素数据
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

        // 复用缓存的纹理句柄，避免每帧创建新纹理
        let texture = state.heatmap_texture.get_or_insert_with(|| {
            ctx.load_texture("heatmap", color_image.clone(), egui::TextureOptions::NEAREST)
        });
        // 更新纹理内容（只更新像素数据，不重新分配 GPU 对象）
        texture.set(color_image, egui::TextureOptions::NEAREST);

        // 居中放置正方形图像区域
        let avail = ui.available_rect_before_wrap();
        let max_edge = avail.size().x.min(avail.size().y);
        let center = avail.center();
        let image_rect = egui::Rect::from_center_size(center, egui::vec2(max_edge, max_edge));

        // 绘制热力图并记录实际渲染区域
        let response = ui.put(
            image_rect,
            egui::Image::from_texture(SizedTexture::from(&*texture))
                .fit_to_exact_size(egui::vec2(max_edge, max_edge)),
        );
        let texture_rect = response.rect;
        let painter = ui.painter();
        let particle_count = snapshot.x.len();
        let lx = if snapshot.lx <= 0.0 { 1.0 } else { snapshot.lx };
        let ly = if snapshot.ly <= 0.0 { 1.0 } else { snapshot.ly };

        for p in 0..particle_count {
            // 世界坐标转归一化 [0,1]，偏移 0.5 像素对齐热力图像素中心
            let nx_p = (((snapshot.x[p] / lx) * nx as f64 + 0.5) / nx as f64).clamp(0.0, 1.0);
            let ny_p = (((snapshot.y[p] / ly) * ny as f64 + 0.5) / ny as f64).clamp(0.0, 1.0);

            // 映射到屏幕坐标（热力图区域）
            let sx = texture_rect.left() + nx_p as f32 * texture_rect.width();
            let sy = texture_rect.bottom() - ny_p as f32 * texture_rect.height();

            let color = if snapshot.q[p] < 0.0 {
                egui::Color32::CYAN
            } else {
                egui::Color32::WHITE
            };

            // 绘制粒子圆点
            painter.circle_filled(
                egui::pos2(sx, sy),
                3.0,
                color,
            );
        }

        // 处理鼠标交互
        handle_mouse_interaction(
            ui,
            sim,
            interaction,
            texture_rect,
            nx,
            ny,
            lx,
            ly,
        );
    });

    // 非暂停时自动执行模拟步进
    if !*paused {
        let dt = variant.config().dt;
        sim.step(dt);
        *step_count += 1;
        ctx.request_repaint();
    }

    true
}

/// 处理鼠标拖动粒子交互
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

    let Some(pos) = mouse_pos else { return };
    if !texture_rect.contains(pos) {
        // 鼠标移出画布时取消拖动
        interaction.dragging = false;
        interaction.dragged_particle_index = None;
        return;
    }

    // 归一化鼠标位置到 [0,1]（纹理坐标系）
    // tex_u: left=0.0, right=1.0
    // tex_v: top=1.0, bottom=0.0（对应纹理坐标 Y 轴向上）
    let tex_u = ((pos.x - texture_rect.left()) / texture_rect.width()).clamp(0.0f32, 1.0f32);
    let tex_v = ((texture_rect.bottom() - pos.y) / texture_rect.height()).clamp(0.0f32, 1.0f32);

    // 鼠标在热力图像素中心的视觉坐标 → 逆变换回世界坐标
    // 与粒子渲染的 ((x/lx * nx + 0.5) / nx) 互为逆运算
    let inv_nx = grid_nx as f64;
    let inv_ny = grid_ny as f64;
    let tex_u_f64 = tex_u as f64;
    let tex_v_f64 = tex_v as f64;
    let world_x = ((tex_u_f64 * inv_nx - 0.5) / inv_nx).clamp(0.0, 1.0) * lx;
    let world_y = ((tex_v_f64 * inv_ny - 0.5) / inv_ny).clamp(0.0, 1.0) * ly;

    if mouse_down {
        if !interaction.dragging {
            // 尝试选择最近的粒子（在视觉坐标空间中比较，即带半像素偏移的归一化坐标）
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

        // 正在拖动：更新粒子位置
        if let Some(idx) = interaction.dragged_particle_index {
            sim.particles.x[idx] = world_x;
            sim.particles.y[idx] = world_y;
            sim.particles.vx[idx] = 0.0;
            sim.particles.vy[idx] = 0.0;
        }
    } else {
        // 鼠标释放
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