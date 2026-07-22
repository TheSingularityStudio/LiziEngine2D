use eframe::egui;
use egui::ColorImage;
use ndarray::Array2;
use std::sync::{Arc, Mutex};

use crate::core::sim::StateSnapshot;
use crate::visual::colors::heatmap_rgb;
use crate::visual::window::VisualWindow;

/// Egui 交互式可视化窗口
///
/// 提供完整的 GUI 控件：
/// - Play/Pause/Step 按钮
/// - V 热力图渲染
pub struct EguiApp {
    snapshot: Arc<Mutex<Option<StateSnapshot>>>,
    should_close: Arc<Mutex<bool>>,
    paused: Arc<Mutex<bool>>,
    step_requested: Arc<Mutex<bool>>,
}

impl EguiApp {
    /// 创建新的 egui 窗口（在后台线程中启动 eframe）
    pub fn new(title: &str, width: usize, height: usize) -> Self {
        let snapshot = Arc::new(Mutex::new(None::<StateSnapshot>));
        let should_close = Arc::new(Mutex::new(false));
        let paused = Arc::new(Mutex::new(false));
        let step_requested = Arc::new(Mutex::new(false));

        let snapshot_clone = snapshot.clone();
        let should_close_clone = should_close.clone();
        let paused_clone = paused.clone();
        let step_requested_clone = step_requested.clone();

        let title_owned = title.to_string();

        // 在独立线程中启动 egui 窗口
        std::thread::spawn(move || {
            let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([width as f32, height as f32]),
                ..Default::default()
            };

            let result = eframe::run_native(
                &title_owned,
                native_options,
                Box::new(move |_cc| {
                    Ok(Box::new(EguiAppState {
                        snapshot: snapshot_clone.clone(),
                        should_close: should_close_clone.clone(),
                        paused: paused_clone.clone(),
                        step_requested: step_requested_clone.clone(),
                        v_min: 0.0,
                        v_max: 1.0,
                    }))
                }),
            );
            if let Err(e) = result {
                eprintln!("Egui 窗口错误: {}", e);
            }
        });

        Self {
            snapshot,
            should_close,
            paused,
            step_requested,
        }
    }

    /// 检查是否请求了步进
    pub fn is_step_requested(&self) -> bool {
        let mut req = self.step_requested.lock().unwrap();
        if *req {
            *req = false;
            true
        } else {
            false
        }
    }
}

impl VisualWindow for EguiApp {
    fn render(&mut self, snapshot: &StateSnapshot) -> bool {
        // 更新快照
        if let Ok(mut snap) = self.snapshot.lock() {
            *snap = Some(snapshot.clone());
        }

        // 检查是否应关闭
        if *self.should_close.lock().unwrap() {
            return false;
        }

        // 如果暂停且未请求步进，保持
        if self.is_paused() && !self.is_step_requested() {
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        true
    }

    fn should_close(&self) -> bool {
        *self.should_close.lock().unwrap()
    }

    fn is_paused(&self) -> bool {
        *self.paused.lock().unwrap()
    }
}

/// egui 应用状态
struct EguiAppState {
    snapshot: Arc<Mutex<Option<StateSnapshot>>>,
    should_close: Arc<Mutex<bool>>,
    paused: Arc<Mutex<bool>>,
    step_requested: Arc<Mutex<bool>>,
    v_min: f64,
    v_max: f64,
}

impl EguiAppState {
    /// 将 V 网格渲染为热力图纹理
    fn render_heatmap_texture(&self, v: &Array2<f64>) -> ColorImage {
        let (nx, ny) = v.dim();

        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for val in v.iter() {
            if *val < min {
                min = *val;
            }
            if *val > max {
                max = *val;
            }
        }

        let v_min = self.v_min * 0.9 + min * 0.1;
        let v_max = self.v_max * 0.9 + max * 0.1;
        let v_min = if (v_max - v_min).abs() < 1e-12 {
            v_max - 1.0
        } else {
            v_min
        };

        let mut pixels = Vec::with_capacity(nx * ny);
        for j in (0..ny).rev() {
            for i in 0..nx {
                let val = v[[i, j]];
                let (r, g, b) = heatmap_rgb(val, v_min, v_max);
                pixels.push(egui::Color32::from_rgb(r, g, b));
            }
        }

        ColorImage {
            size: [nx, ny],
            pixels,
        }
    }
}

impl eframe::App for EguiAppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let snapshot_guard = self.snapshot.lock().unwrap();
        let snapshot = snapshot_guard.as_ref();

        // 顶部控制面板
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if *self.paused.lock().unwrap() {
                    if ui.button("▶ Play").clicked() {
                        *self.paused.lock().unwrap() = false;
                    }
                } else {
                    if ui.button("⏸ Pause").clicked() {
                        *self.paused.lock().unwrap() = true;
                    }
                }

                if ui.button("⏭ Step").clicked() {
                    *self.paused.lock().unwrap() = true;
                    *self.step_requested.lock().unwrap() = true;
                }

                ui.separator();

                if let Some(snap) = snapshot {
                    let mut v_min = f64::MAX;
                    let mut v_max = f64::MIN;
                    for val in snap.v.iter() {
                        if *val < v_min { v_min = *val; }
                        if *val > v_max { v_max = *val; }
                    }
                    ui.label(format!(
                        "V: [{:.2e}, {:.2e}] | {} particles",
                        v_min, v_max, snap.x.len()
                    ));
                } else {
                    ui.label("Waiting for data...");
                }
            });
        });

        // 中央画布
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(snap) = snapshot {
                let color_image = self.render_heatmap_texture(&snap.v);
                let texture = ctx.load_texture(
                    "heatmap",
                    color_image,
                    egui::TextureOptions::NEAREST,
                );

                let available = ui.available_size();
                let aspect = snap.v.dim().0 as f32 / snap.v.dim().1 as f32;
                let (w, h) = if available.x / available.y > aspect {
                    (available.y * aspect, available.y)
                } else {
                    (available.x, available.x / aspect)
                };

                ui.centered_and_justified(|ui| {
                    ui.add(
                        egui::Image::from_texture(
                            egui::load::SizedTexture::from(&texture),
                        )
                        .max_size(egui::vec2(w, h)),
                    );
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.heading("Waiting for simulation data...");
                });
            }
        });

        // 检查关闭
        if ctx.input(|i| i.viewport().close_requested()) {
            *self.should_close.lock().unwrap() = true;
        }

        ctx.request_repaint();
    }
}