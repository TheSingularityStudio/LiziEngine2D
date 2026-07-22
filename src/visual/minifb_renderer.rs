use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};
use ndarray::Array2;
use crate::core::sim::StateSnapshot;
use crate::visual::colors::{heatmap_rgb, pack_rgb};

/// 基于 minifb 的轻量窗口渲染器
///
/// 将仿真状态（V 热力图 + 粒子叠加）渲染到窗口中。
/// 支持 ESC 退出、Space 暂停/继续。
/// **不处理鼠标交互**——交互由主循环直接操作 sim。
pub struct MinifbRenderer {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    paused: bool,
    v_min: f64,
    v_max: f64,
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

    /// 渲染粒子
    fn render_particles(&mut self, snapshot: &StateSnapshot) {
        let dot_radius: isize = 2;
        let lx = if snapshot.lx <= 0.0 { 1.0 } else { snapshot.lx };
        let ly = if snapshot.ly <= 0.0 { 1.0 } else { snapshot.ly };

        for p in 0..snapshot.x.len() {
            // 将世界坐标归一化到 [0, 1]，然后转换为像素坐标
            let nx = snapshot.x[p] / lx;
            let ny = snapshot.y[p] / ly;
            let px = ((nx * self.width as f64) as usize).min(self.width - 1);
            let py = ((ny * self.height as f64) as usize).min(self.height - 1);

            // 根据电荷量选择颜色
            let dot_color = if snapshot.q[p] < 0.0 {
                0x0000FFFFu32 // 负电荷：青色
            } else {
                0x00FFFFFFu32 // 正电荷：白色
            };

            for dy in -(dot_radius)..=dot_radius {
                for dx in -(dot_radius)..=dot_radius {
                    if dx * dx + dy * dy <= dot_radius * dot_radius {
                        let sx = (px as isize + dx).max(0).min(self.width as isize - 1) as usize;
                        let sy = (py as isize + dy).max(0).min(self.height as isize - 1) as usize;
                        self.buffer[sy * self.width + sx] = dot_color;
                    }
                }
            }
        }
    }

    /// 获取窗口引用（用于交互）
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// 获取窗口宽度
    pub fn width(&self) -> usize {
        self.width
    }

    /// 获取窗口高度
    pub fn height(&self) -> usize {
        self.height
    }

    /// 获取是否暂停
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 渲染一帧：绘制热力图 + 粒子，更新窗口
    /// 返回 true 表示继续运行，false 表示用户关闭了窗口
    pub fn render(&mut self, snapshot: &StateSnapshot) -> bool {
        // 处理键盘输入
        if self.window.is_key_down(Key::Escape) {
            return false;
        }
        if self.window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            self.paused = !self.paused;
        }

        // 渲染 V 热力图
        self.render_heatmap(&snapshot.v);

        // 叠加粒子
        self.render_particles(snapshot);

        // 更新窗口
        self.window
            .update_with_buffer(&self.buffer, self.width, self.height)
            .expect("minifb 窗口更新失败");

        self.window.is_open()
    }

    /// 窗口是否已关闭
    pub fn should_close(&self) -> bool {
        !self.window.is_open()
    }
}