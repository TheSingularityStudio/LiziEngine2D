# LiziEngine2D 🧪⚡

**2D 静电 PIC (Particle-In-Cell) 模拟器** — 基于 Rust 实现，使用 CPU 并行计算电场与粒子运动的实时交互仿真。

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue)](LICENSE)

---

## 🌟 概述

LiziEngine2D 是一个二维静电场-粒子相互作用的物理模拟引擎，采用 **PIC (Particle-In-Cell)** 方法将离散的带电粒子散射到网格上，求解 Poisson 方程得到电势，再通过插值将电场力回传到粒子，从而实现高效的带电粒子系统仿真。

项目提供基于 **egui** 的实时图形界面，支持鼠标交互拖拽粒子、切换预设场景、实时显示电势热力图，直观展示静电场的物理行为。

---

## ✨ 功能特性

### PIC 物理引擎

- **PIC 流水线实现**
  - 双线性散射（Particles → Grid 电荷密度 ρ）
  - 周期边界下 Poisson 方程 FFT 求解 → 电势 V
  - 中心差分计算电场 E = −∇V
  - 双线性 Gather（Grid 电场 → Particles 受力）
  - 半隐式欧拉积分更新粒子运动
- **三种边界条件**
  - `Periodic` — 粒子从一边穿出，从另一边进入
  - `Reflective` — 粒子撞到边界后反弹
  - `Open` — 粒子移出边界即被删除
- **重力系统** — 可启用/禁用，调整大小和方向（向下/向上）
- **摩擦力系统** — 可启用/禁用，阻尼系数可调（F = -damping × v）

### 实时 GUI 交互

- 电势热力图实时渲染（伪彩色映射）
- 粒子位置实时显示（正电荷白色 / 负电荷青色）
- 四种交互工具模式：
  - **拖动粒子** — 点击选中粒子并拖拽移动，粒子速度归零
  - **生成粒子** — 在画布空白处创建自定义粒子（通过生成清单）
  - **删除粒子** — 点击粒子将其删除
  - **查看** — 滚轮缩放、拖拽平移画布，悬停查看粒子详细信息
- **粒子生成清单（Spawnment List）** — 自定义电荷量、质量、是否固定，支持 JSON 导入/导出
- **场景导入/导出** — 将当前模拟状态保存为 `.lz2d` 文件，或加载已有场景
- 可切换显示热力图、网格线

### 控制面板

- ▶️ Play / ⏸️ Pause / ⏭️ Step（单步执行）/ ⟳ Reset（重置）
- 实时显示步数、电势范围、粒子数量
- 可收起/展开的左右侧面板
- 完整的菜单栏系统（文件、选项、帮助）

### 多种预设场景

- 单点电荷、双电荷同号、双电荷异号（偶极子）、随机粒子群

### 中文字体支持

- 自动检测系统字体，支持中文界面显示

---

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Cargo（Rust 包管理器）

### 构建与运行

```bash
# 克隆仓库
git clone https://github.com/TheSingularityStudio/LiziEngine2D.git
cd LiziEngine2D

# 构建（Release 模式以获得最佳性能）
cargo build --release

# 运行 GUI 模拟器（默认启动预设选择界面）
cargo run --release
```

### 命令行参数

```bash
# 指定窗口尺寸启动 GUI
cargo run --release -- gui --width 1024 --height 768
```

> 运行后默认显示预设选择界面，选择一个场景即可进入实时仿真。

---

## 🎮 预设场景

| 场景 | 描述 | 物理意义 |
|------|------|----------|
| **单点电荷** | 网格中央生成一个单位正电荷 | 显示静电场 V 热力图，观察点电荷电势分布 |
| **双电荷（同号）** | 左右两侧生成两个同号正电荷 | 观察叠加电场，两正电荷相互排斥 |
| **双电荷（异号）** | 左右两侧生成一正一负电荷 | 偶极子电场，正负电荷相互吸引形成偶极场 |
| **随机粒子** | 200 个随机初始化的粒子 | 展示 PIC 模拟动画，粒子在静电场中运动 |

### GUI 操作指南

1. **启动程序** → 进入预设选择界面
2. **点击预设按钮** → 进入对应场景的实时模拟
3. **控制模拟**：
   - `▶️ Play` — 开始自动步进模拟
   - `⏸️ Pause` — 暂停模拟
   - `⏭️ Step` — 单步执行（每点一次前进一帧）
   - `⟳ Reset` — 重置到初始状态
   - `← 返回` — 返回预设选择界面
4. **菜单栏**：
   - **文件** → 导入场景 (`.lz2d`) / 导出场景 (`.lz2d`) / 返回预设选择 / 退出
   - **选项** → 显示/隐藏工具面板、参数面板、热力图、网格线
   - **帮助** → 关于 LiziEngine2D / 快捷键说明
5. **工具模式**（左侧面板）：
   - 选择不同工具后，在画布上执行相应操作
   - 可通过右侧面板调整选择半径、查看粒子参数等
6. **鼠标交互**：
   - 在画布上拖拽任意粒子，观察其对电场的影响
   - 查看模式下滚轮缩放、拖拽平移画布

### 四种工具模式详解

| 工具 | 图标 | 功能 |
|------|------|------|
| **拖动粒子** | ✋ | 点击选中粒子并拖拽移动，被拖拽的粒子速度归零。可通过右侧面板调整「选择半径」 |
| **生成粒子** | ✨ | 在右侧面板配置生成清单（电荷量、质量、是否固定），点击画布空白处批量生成所有清单中的粒子。清单支持 JSON 导入/导出 |
| **删除粒子** | ❌ | 点击粒子将其删除。可通过右侧面板调整「选择半径」 |
| **查看** | 🔍 | 滚轮缩放画布、拖拽平移画布。悬停粒子可查看详细信息（索引、位置、速度、电荷量、质量），粒子高亮显示 |

---

## 🏗️ 技术架构

### PIC 流水线流程

```
┌─────────────────────────────────────────────────────┐
│                  时间步循环 (dt)                      │
│                                                      │
│  粒子位置 ──→ Scatter(双线性) ──→ 网格电荷密度 ρ      │
│                                        ↓              │
│  粒子受力 ←── Gather(双线性) ←── 电场 E = −∇V       │
│                                        ↑              │
│  半隐式欧拉积分                           Poisson 求解(FFT) │
│                                        ↑              │
│  边界条件 / 速度限制                  电势 V            │
└─────────────────────────────────────────────────────┘
```

### 模块结构

```
src/
├── main.rs           # CLI 入口（clap 解析）
├── lib.rs            # 库入口，公开模块
├── core/             # 核心物理引擎
│   ├── grid.rs       # 网格定义
│   ├── particles.rs  # 粒子状态管理
│   ├── scatter.rs    # 电荷散射到网格
│   ├── poisson_fft.rs / poisson_solver.rs  # FFT Poisson 求解
│   ├── interp.rs     # 电场插值（Gather）
│   ├── integrator.rs # 半隐式欧拉积分器
│   ├── boundary.rs   # 边界条件 & 速度限制
│   └── sim.rs        # 模拟器主控（ElectrostaticSim2D）
├── gui/              # 图形界面
│   ├── app.rs        # egui 应用主体
│   └── interaction.rs # 鼠标交互状态（工具模式、生成清单等）
├── presets/          # 预设场景
│   └── config.rs     # 预设定义与构建
└── visual/           # 可视化工具
    └── colors.rs     # 热力图颜色映射
```

### 核心依赖

| 依赖 | 用途 |
|------|------|
| [ndarray](https://crates.io/crates/ndarray) | 多维数组操作（网格数据、粒子数据） |
| [ndrustfft](https://crates.io/crates/ndrustfft) | FFT Poisson 求解器 |
| [eframe/egui](https://crates.io/crates/eframe) | 即时模式 GUI 框架 |
| [clap](https://crates.io/crates/clap) | 命令行参数解析 |
| [rand](https://crates.io/crates/rand) | 随机粒子初始化 |
| [image](https://crates.io/crates/image) | 图像处理 |
| [serde](https://crates.io/crates/serde) | 序列化/反序列化（场景保存、生成清单） |
| [rfd](https://crates.io/crates/rfd) | 原生文件对话框 |
| [bincode](https://crates.io/crates/bincode) | 二进制序列化格式（场景文件 `.lz2d`） |
| [serde_json](https://crates.io/crates/serde_json) | JSON 序列化（生成清单导入/导出） |

---

## 🛠️ 项目构建

```bash
# 调试构建
cargo build

# 发布构建（推荐用于实际运行）
cargo build --release

# 运行所有测试
cargo test

# 运行验证测试（验证物理正确性）
cargo test --test validation
```

### 验证测试

项目包含三个物理验证测试，验证 PIC 引擎的正确性：

| 测试 | 描述 | 验证内容 |
|------|------|----------|
| `validate_single_charge_direction_consistency` | 单电荷方向一致性验证 | 验证单点电荷产生的电场方向是否径向向外（平均方向误差 ≤ 0.2） |
| `validate_two_charge_superposition` | 两电荷叠加验证 | 验证两个同号电荷的电场满足叠加原理（相对 L2 误差 ≤ 5%） |
| `validate_random_numerical_stability` | 随机数值稳定性验证 | 验证随机初始化的粒子在多步模拟后速度不发散（最大速度 ≤ 50.0） |

---

## 📂 场景文件格式

LiziEngine2D 使用自定义的 `.lz2d` 文件格式保存和加载场景。该格式使用 `bincode` 序列化，包含模拟器的完整状态（网格、粒子位置/速度/电荷量/质量、配置参数等）。

---

## 📜 许可证

本项目基于 [MIT License](LICENSE) 开源。

---

## 🔗 相关链接

- [GitHub 仓库](https://github.com/TheSingularityStudio/LiziEngine2D)
- [PIC 方法介绍](https://en.wikipedia.org/wiki/Particle-in-cell)
- [egui 框架文档](https://docs.rs/egui/)

---

*用 Rust 写就的粒子模拟器，探索静电场的奇妙世界。*