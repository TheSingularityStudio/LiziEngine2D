/// 工具模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolMode {
    DragParticle,   // 拖动粒子
    PlaceParticle,  // 放置粒子
    DeleteParticle, // 删除粒子
}

impl ToolMode {
    pub fn all() -> [ToolMode; 3] {
        [
            ToolMode::DragParticle,
            ToolMode::PlaceParticle,
            ToolMode::DeleteParticle,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ToolMode::DragParticle => "拖动",
            ToolMode::PlaceParticle => "放置",
            ToolMode::DeleteParticle => "删除",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ToolMode::DragParticle => "\u{1F5B1}",  // 🖱
            ToolMode::PlaceParticle => "+",
            ToolMode::DeleteParticle => "-",
        }
    }
}

/// 放置粒子的参数
#[derive(Debug, Clone)]
pub struct PlaceParticleParams {
    pub charge: f64, // 电荷量
    pub fixed: bool, // 是否固定粒子（速度为0）
}

impl Default for PlaceParticleParams {
    fn default() -> Self {
        Self {
            charge: 1.0,
            fixed: false,
        }
    }
}

/// 交互状态
#[derive(Debug, Clone)]
pub struct InteractionState {
    /// 当前选中的工具模式
    pub tool_mode: ToolMode,
    /// 放置粒子参数
    pub place_params: PlaceParticleParams,
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
            tool_mode: ToolMode::DragParticle,
            place_params: PlaceParticleParams::default(),
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