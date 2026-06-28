from __future__ import annotations

import os
import sys

import numpy as np

# 让脚本在任意工作目录下也能 import 到 src/lizi2d
PROJECT_ROOT = os.path.dirname(os.path.dirname(__file__))
SRC_ROOT = os.path.join(PROJECT_ROOT, "src")
if SRC_ROOT not in sys.path:
    sys.path.insert(0, SRC_ROOT)

from lizi2d.grid import Grid2D  # type: ignore[import-not-found]
from lizi2d.particles import ParticleState  # type: ignore[import-not-found]
from lizi2d.sim import ElectrostaticSim2D  # type: ignore[import-not-found]
from lizi2d.visualize_matplotlib import visualize_matplotlib_2d  # type: ignore[import-not-found]


def main():
    # 网格与物理参数（周期域）
    nx, ny = 64, 64
    Lx, Ly = 1.0, 1.0
    dx = Lx / nx
    dy = Ly / ny

    grid = Grid2D(nx=nx, ny=ny, dx=dx, dy=dy)

    # 粒子初始条件：少量单位电荷粒子（q=1）
    particle_count = 80
    rng = np.random.default_rng(42)

    x = rng.uniform(0.0, Lx, size=(particle_count,))
    y = rng.uniform(0.0, Ly, size=(particle_count,))
    vx = rng.uniform(-0.1, 0.1, size=(particle_count,))
    vy = rng.uniform(-0.1, 0.1, size=(particle_count,))

    zeros = np.zeros_like(x, dtype=np.float64)
    particles = ParticleState(
        x=x.astype(np.float64),
        y=y.astype(np.float64),
        vx=vx.astype(np.float64),
        vy=vy.astype(np.float64),
        fx=zeros.copy(),
        fy=zeros.copy(),
    )

    # 创建仿真器
    sim = ElectrostaticSim2D(grid=grid, particles=particles, eps_poisson=1e-12)

    # 可视化（最小可用版本）
    visualize_matplotlib_2d(
        sim,
        dt=0.02,
        steps_per_frame=1,
        frames=300,
        interval_ms=25,
        show_particles=True,
        show_v=True,
        show_e=True,
        e_stride=6,
        e_scale=0.25,
        v_range_mode="percentile",
    )


if __name__ == "__main__":
    main()
