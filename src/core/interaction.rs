use minifb::{Window, MouseButton};
use crate::core::sim::ElectrostaticSim2D;

/// 交互状态
#[derive(Debug, Clone)]
pub struct InteractionState {
    /// 是否正在拖动粒子
    pub dragging: bool,
    /// 当前拖动的粒子索引（如果有）
    pub dragged_particle_index: Option<usize>,
    /// 选择粒子的半径（归一化坐标）
    pub selection_radius: f64,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            dragging: false,
            dragged_particle_index: None,
            selection_radius: 0.05, // 默认选择半径为窗口尺寸的 5%
        }
    }
}

impl InteractionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 处理鼠标交互，直接修改 sim 中的粒子位置
    ///
    /// 返回 true 表示有粒子被拖动中
    pub fn handle_mouse(
        &mut self,
        window: &Window,
        width: usize,
        height: usize,
        sim: &mut ElectrostaticSim2D,
    ) -> bool {
        // 获取鼠标位置（像素坐标）
        let (mouse_px, mouse_py) = match window.get_mouse_pos(minifb::MouseMode::Clamp) {
            Some((x, y)) => (x as f64, y as f64),
            None => return self.dragging,
        };

        // 转换为归一化坐标 [0, 1]
        let mouse_nx = mouse_px / width as f64;
        let mouse_ny = mouse_py / height as f64;

        // 转换为世界坐标
        let lx = sim.grid.lx();
        let ly = sim.grid.ly();
        let lx = if lx <= 0.0 { 1.0 } else { lx };
        let ly = if ly <= 0.0 { 1.0 } else { ly };
        let mouse_world_x = mouse_nx * lx;
        let mouse_world_y = mouse_ny * ly;

        // 处理鼠标左键
        if window.get_mouse_down(MouseButton::Left) {
            if !self.dragging {
                // 尝试选择粒子：粒子坐标归一化后比较
                let norm_x: Vec<f64> = sim.particles.x.iter().map(|&x| x / lx).collect();
                let norm_y: Vec<f64> = sim.particles.y.iter().map(|&y| y / ly).collect();

                let mut min_dist = f64::MAX;
                let mut min_index = None;
                for i in 0..sim.particles.len() {
                    let dx = norm_x[i] - mouse_nx;
                    let dy = norm_y[i] - mouse_ny;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < min_dist {
                        min_dist = dist;
                        min_index = Some(i);
                    }
                }
                if let Some(idx) = min_index {
                    if min_dist <= self.selection_radius {
                        self.dragging = true;
                        self.dragged_particle_index = Some(idx);
                    }
                }
            }

            // 如果正在拖动，直接修改 sim 的粒子位置和速度
            if let Some(idx) = self.dragged_particle_index {
                sim.particles.x[idx] = mouse_world_x;
                sim.particles.y[idx] = mouse_world_y;
                sim.particles.vx[idx] = 0.0;
                sim.particles.vy[idx] = 0.0;
                return true;
            }
        } else {
            // 鼠标释放
            self.dragging = false;
            self.dragged_particle_index = None;
        }

        self.dragging
    }
}