use minifb::{Key, Window, WindowOptions, Scale, ScaleMode, MouseButton};
use ndarray::Array2;
use crate::core::sim::StateSnapshot;
use crate::core::interaction::InteractionState;
use crate::visual::colors::{heatmap_rgb, pack_rgb};
use crate::visual::window::VisualWindow;

/// 交互事件：表示用户拖动粒子完成后的信息
#[derive(Debug, Clone)]
pub struct InteractionEvent {
    pub particle_index: usize,
    pub x: f64,
    pub y: f64,
}

/// 基于 minifb 的轻量窗口渲染器
///
/// 将仿真状态（V 热力图 + 粒子叠加）渲染到窗口中。
/// 支持 ESC 退出、Space 暂停/继续、鼠标拖动粒子。
pub struct MinifbRenderer {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    paused: bool,
    v_min: f64,
    v_max: f64,
    /// 交互状态
    interaction: InteractionState,
    /// 待处理的粒子位置更新（用于持久化拖动结果）
    pending_updates: Vec<(usize, f64, f64)>,
}

impl MinifbRenderer {
    /// 创建新的 minifb 窗口
    ///
    /// * `title` - 窗口标题
    /// * `width` - 窗口宽度（像素）
    /// * `height` - 窗口高度（像素）
    pub fn new(title: &str, width: usize, height: usize) -> Self {
        let mut window = Window::new(
            title,
            width,
            height,
            WindowOptions {
                scale: Scale::X2,
                scale_mode: ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            },
        )
        .expect("无法创建 minifb 窗口");

        #[allow(deprecated)]
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        let buffer = vec![0u32; width * height];

        Self {
            window,
            buffer,
            width,
            height,
            paused: false,
            v_min: 0.0,
            v_max: 1.0,
            interaction: InteractionState::new(),
            pending_updates: Vec::new(),
        }
    }

    /// 将 V 网格绘制为热力图到帧缓冲
    fn render_heatmap(&mut self, v: &Array2<f64>) {
        let (nx, ny) = v.dim();

        // 计算 V 的 min/max（每帧更新）
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for val in v.iter() {
            if *val < min { min = *val; }
            if *val > max { max = *val; }
        }
        // 平滑过渡 min/max
        self.v_min = self.v_min * 0.9 + min * 0.1;
        self.v_max = self.v_max * 0.9 + max * 0.1;
        if (self.v_max - self.v_min).abs() < 1e-12 {
            self.v_max = self.v_min + 1.0;
        }

        // 将 nx×ny 网格拉伸到 width×height
        let scale_x = self.width as f64 / nx as f64;
        let scale_y = self.height as f64 / ny as f64;

        for py in 0..self.height {
            let gy = (py as f64 / scale_y) as usize;
            let gy = gy.min(ny - 1);
            for px in 0..self.width {
                let gx = (px as f64 / scale_x) as usize;
                let gx = gx.min(nx - 1);
                let val = v[[gx, gy]];
                let (r, g, b) = heatmap_rgb(val, self.v_min, self.v_max);
                self.buffer[py * self.width + px] = pack_rgb(r, g, b);
            }
        }
    }

}

impl MinifbRenderer {
    /// 处理鼠标交互
    fn handle_mouse_interaction(&mut self, snapshot: &mut StateSnapshot) {
        // 获取鼠标位置（像素坐标）
        let (mouse_px, mouse_py) = match self.window.get_mouse_pos(minifb::MouseMode::Clamp) {
            Some((x, y)) => (x as f64, y as f64),
            None => return,
        };

        // 转换为归一化坐标 [0, 1]
        let mouse_nx = mouse_px / self.width as f64;
        let mouse_ny = mouse_py / self.height as f64;

        // 更新交互状态的鼠标位置（归一化坐标）
        self.interaction.update_mouse_position(mouse_nx, mouse_ny);

        // 将归一化鼠标坐标转换为世界坐标
        let lx = snapshot.lx;
        let ly = snapshot.ly;
        let lx = if lx <= 0.0 { 1.0 } else { lx };
        let ly = if ly <= 0.0 { 1.0 } else { ly };
        let mouse_world_x = mouse_nx * lx;
        let mouse_world_y = mouse_ny * ly;

        // 处理鼠标左键
        if self.window.get_mouse_down(MouseButton::Left) {
            if !self.interaction.is_dragging() {
                // 尝试选择粒子（使用归一化坐标比较）
                // 需要将粒子世界坐标转换为归一化坐标进行比较
                let norm_x: Vec<f64> = snapshot.x.iter().map(|&x| x / lx).collect();
                let norm_y: Vec<f64> = snapshot.y.iter().map(|&y| y / ly).collect();
                if let Some((idx, _dist)) = self.interaction.find_nearest_particle(
                    &norm_x,
                    &norm_y,
                ) {
                    self.interaction.start_drag(idx);
                }
            }

            // 如果正在拖动，更新粒子位置（使用世界坐标）
            if let Some(idx) = self.interaction.dragged_particle() {
                snapshot.x[idx] = mouse_world_x;
                snapshot.y[idx] = mouse_world_y;
                // 重置被拖动粒子的速度
                snapshot.vx[idx] = 0.0;
                snapshot.vy[idx] = 0.0;
            }
        } else {
            // 鼠标释放，停止拖动，并记录位置更新
            if self.interaction.is_dragging() {
                if let Some(idx) = self.interaction.dragged_particle() {
                    // 记录最终位置到待处理更新列表
                    self.pending_updates.push((idx, snapshot.x[idx], snapshot.y[idx]));
                }
                self.interaction.stop_drag();
            }
        }
    }

    /// 渲染粒子，高亮显示被拖动的粒子
    fn render_particles_with_highlight(
        &mut self,
        x: &ndarray::Array1<f64>,
        y: &ndarray::Array1<f64>,
        lx: f64,
        ly: f64,
    ) {
        let dot_radius: isize = 2;
        let dragged_idx = self.interaction.dragged_particle();

        // 防止除以零
        let lx = if lx <= 0.0 { 1.0 } else { lx };
        let ly = if ly <= 0.0 { 1.0 } else { ly };

        for p in 0..x.len() {
            // 将世界坐标归一化到 [0, 1]，然后转换为像素坐标
            let nx = x[p] / lx;
            let ny = y[p] / ly;
            let px = ((nx * self.width as f64) as usize).min(self.width - 1);
            let py = ((ny * self.height as f64) as usize).min(self.height - 1);

            // 选择颜色：被拖动的粒子为黄色，其他为白色
            let dot_color = if dragged_idx == Some(p) {
                0x00FFFF00u32 // 黄色
            } else {
                0x00FFFFFFu32 // 白色
            };

            // 如果被拖动，绘制稍大的圆点
            let radius = if dragged_idx == Some(p) {
                dot_radius + 1
            } else {
                dot_radius
            };

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        let sx = (px as isize + dx).max(0).min(self.width as isize - 1) as usize;
                        let sy = (py as isize + dy).max(0).min(self.height as isize - 1) as usize;
                        self.buffer[sy * self.width + sx] = dot_color;
                    }
                }
            }
        }
    }
}

impl MinifbRenderer {
    /// 获取当前暂停状态
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 获取当前正在拖动的粒子索引
    pub fn dragged_particle_index(&self) -> Option<usize> {
        self.interaction.dragged_particle()
    }

    /// 检查是否正在拖动粒子
    pub fn is_dragging(&self) -> bool {
        self.interaction.is_dragging()
    }

    /// 获取拖动粒子的新位置（如果正在拖动）
    /// 返回 (粒子索引, 新x, 新y)
    pub fn get_drag_position(&self, snapshot: &StateSnapshot) -> Option<(usize, f64, f64)> {
        if let Some(idx) = self.interaction.dragged_particle() {
            return Some((idx, snapshot.x[idx], snapshot.y[idx]));
        }
        None
    }

    /// 获取并清除待处理的粒子位置更新
    /// 返回需要更新的粒子位置和速度信息
    pub fn get_pending_particle_updates(&mut self) -> Vec<(usize, f64, f64)> {
        std::mem::take(&mut self.pending_updates)
    }
}

impl VisualWindow for MinifbRenderer {
    fn update(&mut self, snapshot: &StateSnapshot) -> bool {
        // 处理键盘输入
        if self.window.is_key_down(Key::Escape) {
            return false;
        }
        if self.window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            self.paused = !self.paused;
        }

        // 克隆 snapshot 以便修改（用于交互）
        let mut mutable_snapshot = snapshot.clone();

        if !self.paused {
            // 渲染 V 热力图
            self.render_heatmap(&mutable_snapshot.v);

            // 处理鼠标交互
            self.handle_mouse_interaction(&mut mutable_snapshot);

            // 叠加粒子（使用高亮渲染）
            self.render_particles_with_highlight(
                &mutable_snapshot.x,
                &mutable_snapshot.y,
                mutable_snapshot.lx,
                mutable_snapshot.ly,
            );
        } else {
            // 暂停时也允许拖动粒子
            self.handle_mouse_interaction(&mut mutable_snapshot);
            self.render_particles_with_highlight(
                &mutable_snapshot.x,
                &mutable_snapshot.y,
                mutable_snapshot.lx,
                mutable_snapshot.ly,
            );
        }

        // 更新窗口
        self.window
            .update_with_buffer(&self.buffer, self.width, self.height)
            .expect("minifb 窗口更新失败");

        self.window.is_open()
    }

    fn should_close(&self) -> bool {
        !self.window.is_open()
    }
}
