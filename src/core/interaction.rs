/// 交互模块：处理鼠标与粒子的交互
///
/// 提供鼠标拖动粒子的功能

/// 交互状态
#[derive(Debug, Clone)]
pub struct InteractionState {
    /// 是否正在拖动粒子
    pub dragging: bool,
    /// 当前拖动的粒子索引（如果有）
    pub dragged_particle_index: Option<usize>,
    /// 鼠标在归一化坐标系中的位置 [0, 1]
    pub mouse_x: f64,
    pub mouse_y: f64,
    /// 选择粒子的半径（归一化坐标）
    pub selection_radius: f64,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            dragging: false,
            dragged_particle_index: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            selection_radius: 0.05, // 默认选择半径为窗口尺寸的 5%
        }
    }
}

impl InteractionState {
    /// 创建新的交互状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新鼠标位置（归一化坐标）
    pub fn update_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    /// 查找距离鼠标最近的粒子索引
    /// 
    /// 返回距离鼠标最近的粒子索引和距离（如果在选择半径内）
    pub fn find_nearest_particle(
        &self,
        particle_x: &[f64],
        particle_y: &[f64],
    ) -> Option<(usize, f64)> {
        let mut min_dist = f64::MAX;
        let mut min_index = None;

        for i in 0..particle_x.len() {
            let dx = particle_x[i] - self.mouse_x;
            let dy = particle_y[i] - self.mouse_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < min_dist {
                min_dist = dist;
                min_index = Some(i);
            }
        }

        match min_index {
            Some(idx) if min_dist <= self.selection_radius => Some((idx, min_dist)),
            _ => None,
        }
    }

    /// 开始拖动粒子
    pub fn start_drag(&mut self, particle_index: usize) {
        self.dragging = true;
        self.dragged_particle_index = Some(particle_index);
    }

    /// 停止拖动
    pub fn stop_drag(&mut self) {
        self.dragging = false;
        self.dragged_particle_index = None;
    }

    /// 获取当前拖动的粒子索引
    pub fn dragged_particle(&self) -> Option<usize> {
        self.dragged_particle_index
    }

    /// 检查是否在拖动
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }
}