# LiziEngine2D
LiziEngine2D - 2D 电静场（CPU）模拟器（Python）

- 使用 PIC 风格流水线：
  - 粒子散射（双线性）→ 网格离散电荷密度 ρ
  - 周期边界下 Poisson 求解（FFT）→ 电势 V
  - E = -∇V 求电场（中心差分）
  - 网格电场（双线性）→ 粒子受力（gather）
  - 半隐式欧拉积分更新粒子运动
- 目前主要提供 critical-path 验证脚本：
  - `scripts/validate_single_charge.py`
  - `scripts/validate_two_charges.py`
  - `scripts/validate_random.py`
