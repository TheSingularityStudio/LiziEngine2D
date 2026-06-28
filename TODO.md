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
- [ ] 执行三个验证脚本
- [ ] 如误差/稳定性不达标，调整常数或离散方式（网格/时间步等）
