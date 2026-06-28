from __future__ import annotations

import numpy as np

from .grid import Grid2D
from .particles import ParticleState


def step_half_implicit_euler(
    grid: Grid2D,
    particles: ParticleState,
    dt: float,
) -> ParticleState:
    """
    半隐式欧拉（semi-implicit Euler / symplectic Euler）时间积分：

      v_{n+1} = v_n + a_n * dt
      x_{n+1} = x_n + v_{n+1} * dt

    前提：
    - particles.fx/fy 已经在当前步计算好（即当前步的电场给出的受力）
    - 由于本原型采用 q=1、m=1，故加速度 a = F

    返回：
    - 更新后的 particles（位置做周期包裹）
    """
    particles.vx = particles.vx + particles.fx * dt
    particles.vy = particles.vy + particles.fy * dt

    particles.x = particles.x + particles.vx * dt
    particles.y = particles.y + particles.vy * dt

    # 周期包裹
    lx = grid.nx * grid.dx
    ly = grid.ny * grid.dy
    particles.x = np.mod(particles.x, lx)
    particles.y = np.mod(particles.y, ly)

    return particles
