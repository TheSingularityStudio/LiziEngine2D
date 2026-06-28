# LiziEngine2D - 2D 电静场 CPU 模拟器（Python）

## Step 1：项目脚手架（scaffold）
- [x] 创建 `requirements.txt`
- [ ] 创建 `pyproject.toml`（或保持仅 requirements）
- [x] 创建 `src/` 包与基础模块

## Step 2：核心仿真实现
- [x] 实现网格 + 坐标映射
- [x] 实现粒子状态（位置、速度）
- [x] 实现 scatter：粒子 -> 网格电荷密度（ρ），使用双线性加权
- [x] 实现 Poisson 求解：离散格林函数核 + **FFT 的循环卷积**
- [x] 实现梯度：E = -∇V（由网格电势得到）
- [x] 实现 gather：由插值后的电场得到粒子受力（与 scatter 插值保持一致）
- [x] 实现半隐式欧拉（half-implicit Euler）积分器

## Step 3：验证 / 测试（critical-path）
- [x] `scripts/validate_single_charge.py`（单电荷：对称性/方向检查）
- [x] `scripts/validate_two_charges.py`（双电荷：叠加原理/方向检查）
- [x] `scripts/validate_random.py`（随机初始条件：数值稳定性）

## Step 4：运行与报告
- [x] 执行三个验证脚本
  - [x] `scripts/validate_single_charge.py`
  - [x] `scripts/validate_two_charges.py`
  - [x] `scripts/validate_random.py`
- [ ] 如误差/稳定性不达标，调整常数或离散方式（网格/时间步等）

## Step 5：可视化渲染框架（最小可用版本 MVP）
- [ ] 明确“物理-渲染”数据契约（推荐：每帧输出 float32 的 V/Ex/Ey 与粒子 positions）
- [ ] MVP1：只画粒子
  - [ ] 粒子坐标（周期域的 wrap 后显示）正确
  - [ ] GUI：Play/Pause、Step、Reset
- [ ] MVP2：加入电势 V 热力图
  - [ ] GUI：ShowV、colormap 范围（Auto/Fixed/百分位裁剪）
- [ ] MVP3：加入电场 E（箭头稀疏采样）
  - [ ] GUI：ShowE、stride、arrow length 缩放因子
- [ ] 调试叠加（可选，但建议先做）
  - [ ] 网格点/采样点显示开关
  - [ ] 关键统计：V 的 min/max/mean、E 的 max_abs（用于标定缩放）

## Step 6：GUI 与数据传输（跨语言/跨线程）
- [ ] 选择渲染实现路线：OpenGL + ImGui（你已选择：b + a + c）
- [ ] 定义跨模块接口：
  - Python 侧：在 `step()` 后生成“帧快照”(V/Ex/Ey/particles)
  - 渲染侧：只读上一帧快照（双缓冲/三缓冲，避免读写冲突）
- [ ] 暂定 MVP 的数据交换方式（先可用，再优化性能）
  - [ ] 先用拷贝/简单绑定（易实现）
  - [ ] 通过后再升级共享内存/零拷贝（ring buffer）

## Step 7：可视化一致性与数值对齐测试
- [ ] 用同一 seed 场景渲染 V/E：
  - [ ] 渲染端 V/E 的统计量与 Python 端一致（允许浮点误差）
- [ ] 验证周期域显示正确性
  - [ ] 粒子 wrap 显示无明显跳变或“跨域拉伸”
- [ ] 对照验证（可选）
  - [ ] 单点电荷：等值线近似圆对称、E 箭头径向
  - [ ] 两点电荷：叠加方向正确
