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
}