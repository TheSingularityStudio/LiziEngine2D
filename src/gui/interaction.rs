use serde::{Deserialize, Serialize};

/// 工具模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolMode {
    DragParticle,   // 拖动粒子
    SpawnParticle,  // 生成粒子
    DeleteParticle, // 删除粒子
    Inspect,        // 查看（缩放、平移、显示信息）
}

impl ToolMode {
    pub fn all() -> [ToolMode; 4] {
        [
            ToolMode::DragParticle,
            ToolMode::SpawnParticle,
            ToolMode::DeleteParticle,
            ToolMode::Inspect,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ToolMode::DragParticle => "拖动",
            ToolMode::SpawnParticle => "生成",
            ToolMode::DeleteParticle => "删除",
            ToolMode::Inspect => "查看",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ToolMode::DragParticle => "✋",
            ToolMode::SpawnParticle => "✨",
            ToolMode::DeleteParticle => "❌",
            ToolMode::Inspect => "🔍",
        }
    }
}

/// 生成粒子的参数
#[derive(Debug, Clone)]
pub struct SpawnParticleParams {
    pub charge: f64, // 电荷量
    pub mass: f64,   // 质量
}

impl Default for SpawnParticleParams {
    fn default() -> Self {
        Self {
            charge: 1.0,
            mass: 1.0,
        }
    }
}

/// 生成清单中的单个条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnmentEntry {
    pub charge: f64,
    pub mass: f64,
}

impl Default for SpawnmentEntry {
    fn default() -> Self {
        Self {
            charge: 1.0,
            mass: 1.0,
        }
    }
}

/// 粒子排列方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArrangeMode {
    Stack,      // 堆叠（所有粒子在同一位置）
    Horizontal, // 水平排列
    Vertical,   // 垂直排列
    Grid,       // 网格排列
}

impl ArrangeMode {
    pub fn all() -> [ArrangeMode; 4] {
        [ArrangeMode::Stack, ArrangeMode::Horizontal, ArrangeMode::Vertical, ArrangeMode::Grid]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ArrangeMode::Stack => "堆叠",
            ArrangeMode::Horizontal => "水平排列",
            ArrangeMode::Vertical => "垂直排列",
            ArrangeMode::Grid => "网格排列",
        }
    }
}

/// 单个清单的 JSON 数据格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnmentListData {
    pub entries: Vec<SpawnmentEntry>,
    pub spacing: f64,
    pub arrange_mode: ArrangeMode,
}

/// 命名的生成清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedSpawnmentList {
    pub name: String,
    pub entries: Vec<SpawnmentEntry>,
    pub spacing: f64,
    pub arrange_mode: ArrangeMode,
}

impl Default for NamedSpawnmentList {
    fn default() -> Self {
        Self {
            name: "默认".to_string(),
            entries: vec![SpawnmentEntry::default()],
            spacing: 0.03,
            arrange_mode: ArrangeMode::Stack,
        }
    }
}

impl NamedSpawnmentList {
    /// 导出为 JSON 字符串（单个清单格式，兼容旧版）
    pub fn export_to_json(&self) -> Result<String, String> {
        let data = SpawnmentListData {
            entries: self.entries.clone(),
            spacing: self.spacing,
            arrange_mode: self.arrange_mode,
        };
        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("序列化失败: {}", e))
    }

    /// 从 JSON 字符串导入（单个清单格式，兼容旧版）
    pub fn import_from_json(&mut self, json_str: &str) -> Result<(), String> {
        let data: SpawnmentListData = serde_json::from_str(json_str)
            .map_err(|e| format!("反序列化失败: {}", e))?;
        self.entries = data.entries;
        self.spacing = data.spacing;
        self.arrange_mode = data.arrange_mode;
        Ok(())
    }

    /// 根据排列方式生成所有粒子的坐标偏移量（相对于点击点）
    pub fn compute_spawnment_offsets(&self) -> Vec<(f64, f64)> {
        let count = self.entries.len();
        if count == 0 {
            return Vec::new();
        }
        let spacing = self.spacing;
        match self.arrange_mode {
            ArrangeMode::Stack => {
                vec![(0.0, 0.0); count]
            }
            ArrangeMode::Horizontal => {
                let start = -(count as f64 - 1.0) / 2.0 * spacing;
                (0..count).map(|i| {
                    (start + i as f64 * spacing, 0.0)
                }).collect()
            }
            ArrangeMode::Vertical => {
                let start = -(count as f64 - 1.0) / 2.0 * spacing;
                (0..count).map(|i| {
                    (0.0, start + i as f64 * spacing)
                }).collect()
            }
            ArrangeMode::Grid => {
                let cols = (count as f64).sqrt().ceil() as usize;
                let rows = (count + cols - 1) / cols;
                let start_x = -(cols as f64 - 1.0) / 2.0 * spacing;
                let start_y = -(rows as f64 - 1.0) / 2.0 * spacing;
                (0..count).map(|i| {
                    let col = i % cols;
                    let row = i / cols;
                    (start_x + col as f64 * spacing, start_y + row as f64 * spacing)
                }).collect()
            }
        }
    }
}

/// 多个生成清单的集合（用于 JSON 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnmentListCollectionData {
    pub lists: Vec<NamedSpawnmentList>,
}

/// 多个生成清单的容器
#[derive(Debug, Clone)]
pub struct SpawnmentListCollection {
    pub lists: Vec<NamedSpawnmentList>,
    pub selected_index: usize,
}

impl SpawnmentListCollection {
    pub fn new() -> Self {
        Self {
            lists: vec![NamedSpawnmentList::default()],
            selected_index: 0,
        }
    }

    /// 获取当前选中的清单
    pub fn current_list(&self) -> Option<&NamedSpawnmentList> {
        self.lists.get(self.selected_index)
    }

    /// 获取当前选中的清单（可变引用）
    pub fn current_list_mut(&mut self) -> Option<&mut NamedSpawnmentList> {
        self.lists.get_mut(self.selected_index)
    }

    /// 选中指定索引的清单
    pub fn select_list(&mut self, index: usize) {
        if index < self.lists.len() {
            self.selected_index = index;
        }
    }

    /// 添加新清单
    pub fn add_list(&mut self, name: String) {
        self.lists.push(NamedSpawnmentList {
            name,
            entries: vec![SpawnmentEntry::default()],
            spacing: 0.03,
            arrange_mode: ArrangeMode::Stack,
        });
    }

    /// 删除指定索引的清单（如果只剩一个则不清空）
    pub fn remove_list(&mut self, index: usize) -> bool {
        if self.lists.len() <= 1 || index >= self.lists.len() {
            return false;
        }
        self.lists.remove(index);
        if self.selected_index >= self.lists.len() {
            self.selected_index = self.lists.len() - 1;
        }
        true
    }

    /// 重命名指定索引的清单
    pub fn rename_list(&mut self, index: usize, new_name: String) {
        if let Some(list) = self.lists.get_mut(index) {
            list.name = new_name;
        }
    }

    /// 导出整个集合为 JSON
    pub fn export_all_to_json(&self) -> Result<String, String> {
        let data = SpawnmentListCollectionData {
            lists: self.lists.clone(),
        };
        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("序列化失败: {}", e))
    }

    /// 从 JSON 导入整个集合
    pub fn import_all_from_json(&mut self, json_str: &str) -> Result<(), String> {
        let data: SpawnmentListCollectionData = serde_json::from_str(json_str)
            .map_err(|e| format!("反序列化失败: {}", e))?;
        if data.lists.is_empty() {
            return Err("清单集合不能为空".to_string());
        }
        self.lists = data.lists;
        self.selected_index = 0;
        Ok(())
    }
}

/// 悬停粒子信息
#[derive(Debug, Clone)]
pub struct HoveredParticleInfo {
    pub index: usize,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub q: f64,
    pub m: f64,
}

/// 交互状态
#[derive(Debug, Clone)]
pub struct InteractionState {
    /// 当前选中的工具模式
    pub tool_mode: ToolMode,
    /// 生成粒子参数（快速生成）
    pub spawn_params: SpawnParticleParams,
    /// 生成清单集合
    pub spawnment_lists: SpawnmentListCollection,
    /// 是否正在拖动粒子
    pub dragging: bool,
    /// 当前拖动的粒子索引（如果有）
    pub dragged_particle_index: Option<usize>,
    /// 选择粒子的半径（归一化坐标）
    pub selection_radius: f64,
    /// 查看工具：画布平移偏移量 (dx, dy)
    pub view_offset: (f32, f32),
    /// 查看工具：缩放倍率
    pub zoom: f32,
    /// 查看工具：是否正在平移画布
    pub panning: bool,
    /// 查看工具：上次鼠标位置（用于拖拽平移）
    pub last_pan_pos: Option<(f32, f32)>,
    /// 查看工具：悬停的粒子信息
    pub hovered_particle: Option<HoveredParticleInfo>,
    /// 拖动工具：是否启用惯性模式（施加力而非瞬移）
    pub drag_inertia_mode: bool,
    /// 惯性模式下施加力的大小系数
    pub drag_force_strength: f64,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            tool_mode: ToolMode::DragParticle,
            spawn_params: SpawnParticleParams::default(),
            spawnment_lists: SpawnmentListCollection::new(),
            dragging: false,
            dragged_particle_index: None,
            selection_radius: 0.05, // 默认选择半径为窗口尺寸的 5%
            view_offset: (0.0, 0.0),
            zoom: 1.0,
            panning: false,
            last_pan_pos: None,
            hovered_particle: None,
            drag_inertia_mode: false,
            drag_force_strength: 1.0,
        }
    }
}

impl InteractionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 重置查看工具的视图
    pub fn reset_view(&mut self) {
        self.view_offset = (0.0, 0.0);
        self.zoom = 1.0;
    }

    /// 根据当前选中清单生成所有粒子的坐标偏移量（相对于点击点）
    /// 返回 (dx, dy) 向量列表
    pub fn compute_spawnment_offsets(&self) -> Vec<(f64, f64)> {
        self.spawnment_lists.current_list()
            .map(|list| list.compute_spawnment_offsets())
            .unwrap_or_default()
    }
}