# LiziEngine2D
LiziEngine2D - 2D 电静场（CPU）模拟器（Python）

- 使用 PIC 风格流水线：
  - 粒子散射（双线性）→ 网格离散电荷密度 ρ
  - 周期边界下 Poisson 求解（FFT）→ 电势 V
  - E = -∇V 求电场（中心差分）
  - 网格电场（双线性）→ 粒子受力（gather）
  - 半隐式欧拉积分更新粒子运动

# 单点电荷可视化（minifb 窗口，ESC 退出）
cargo run -- demo single-charge

# 双电荷可视化（同号/异号）
cargo run -- demo two-charges --opposite-sign

# 随机粒子动画（200步模拟，实时显示）
cargo run -- demo random-particles --steps 200 --n 200 --dt 0.05

# 原有验证命令保持不变
cargo run -- validate-single-charge
cargo run -- validate-two-charges
cargo run -- validate-random