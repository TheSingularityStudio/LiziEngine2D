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
from lizi2d.visualize_tkinter import TkViewerConfig, run_tkinter_viewer  # type: ignore[import-not-found]


def make_sim(
    *,
    nx: int = 64,
    ny: int = 64,
    Lx: float = 1.0,
    Ly: float = 1.0,
    particle_count: int = 80,
) -> ElectrostaticSim2D:
    dx = Lx / nx
    dy = Ly / ny
    grid = Grid2D(nx=nx, ny=ny, dx=dx, dy=dy)

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

    return ElectrostaticSim2D(grid=grid, particles=particles, eps_poisson=1e-12)


def main():
    sim = make_sim(nx=64, ny=64, Lx=1.0, Ly=1.0, particle_count=80)

    cfg = TkViewerConfig(
        dt=0.02,
        steps_per_frame=1,
        show_v=True,
        show_particles=True,
        show_e=True,
        e_stride=6,
        e_scale=0.25,
        v_percentile_lo=1.0,
        v_percentile_hi=99.0,
        interval_ms=25,
    )

    run_tkinter_viewer(sim, config=cfg)


if __name__ == "__main__":
    main()
