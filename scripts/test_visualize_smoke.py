from __future__ import annotations

import os
import sys

# 让脚本在任意工作目录下也能 import 到 src/lizi2d
PROJECT_ROOT = os.path.dirname(os.path.dirname(__file__))
SRC_ROOT = os.path.join(PROJECT_ROOT, "src")
if SRC_ROOT not in sys.path:
    sys.path.insert(0, SRC_ROOT)

import numpy as np

# Headless：避免无 GUI 环境阻塞
import matplotlib
matplotlib.use("Agg")  # noqa: E402

import matplotlib.pyplot as plt  # noqa: E402

from lizi2d.grid import Grid2D  # noqa: E402
from lizi2d.particles import ParticleState  # noqa: E402
from lizi2d.sim import ElectrostaticSim2D  # noqa: E402
from lizi2d.visualize_matplotlib import visualize_matplotlib_2d  # noqa: E402


def main():
    # 阶段性 smoke：重点覆盖 visualize_matplotlib_2d 的控制分支与参数路径
    nx, ny = 32, 32
    Lx, Ly = 1.0, 1.0
    dx = Lx / nx
    dy = Ly / ny
    grid = Grid2D(nx=nx, ny=ny, dx=dx, dy=dy)

    n = 25
    rng = np.random.default_rng(0)
    x = rng.uniform(0.0, Lx, size=(n,)).astype(np.float64)
    y = rng.uniform(0.0, Ly, size=(n,)).astype(np.float64)
    vx = rng.normal(0.0, 0.01, size=(n,)).astype(np.float64)
    vy = rng.normal(0.0, 0.01, size=(n,)).astype(np.float64)

    particles = ParticleState(
        x=x,
        y=y,
        vx=vx,
        vy=vy,
        fx=np.zeros_like(x),
        fy=np.zeros_like(y),
    )

    sim = ElectrostaticSim2D(grid=grid, particles=particles, eps_poisson=1e-12)

    # 覆盖 plt.show / plt.pause 让它不阻塞
    plt_show_orig = plt.show
    plt_pause_orig = plt.pause
    plt.show = lambda *args, **kwargs: None  # type: ignore[assignment]
    plt.pause = lambda *args, **kwargs: None  # type: ignore[assignment]

    try:
        visualize_matplotlib_2d(
            sim,
            dt=0.01,
            steps_per_frame=1,
            frames=1,
            interval_ms=1,
            show_particles=True,
            show_v=True,
            show_e=True,
            e_stride=4,
            e_scale=0.25,
            v_range_mode="percentile",
        )

        visualize_matplotlib_2d(
            sim,
            dt=0.01,
            steps_per_frame=1,
            frames=1,
            interval_ms=1,
            show_particles=False,
            show_v=True,
            show_e=False,
            v_range_mode="auto",
        )

        visualize_matplotlib_2d(
            sim,
            dt=0.01,
            steps_per_frame=1,
            frames=1,
            interval_ms=1,
            show_particles=True,
            show_v=False,
            show_e=True,
            e_stride=8,
            e_scale=0.1,
            v_range_mode="fixed",
            vmin=-1.0,
            vmax=1.0,
        )

        print("visualize_matplotlib_2d smoke test: OK")
    finally:
        plt.show = plt_show_orig  # type: ignore[assignment]
        plt.pause = plt_pause_orig  # type: ignore[assignment]


if __name__ == "__main__":
    main()
