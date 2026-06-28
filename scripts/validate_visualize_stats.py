from __future__ import annotations

import os
import sys

# 让脚本在任意工作目录下也能 import 到 src/lizi2d
PROJECT_ROOT = os.path.dirname(os.path.dirname(__file__))
SRC_ROOT = os.path.join(PROJECT_ROOT, "src")
if SRC_ROOT not in sys.path:
    sys.path.insert(0, SRC_ROOT)

import numpy as np

# matplotlib 可视化统计验证只需数值，不强依赖 GUI
import matplotlib
matplotlib.use("Agg")

from lizi2d.grid import Grid2D
from lizi2d.particles import ParticleState
from lizi2d.sim import ElectrostaticSim2D
from lizi2d.visualize_matplotlib import _safe_vmin_vmax


def main() -> None:
    # 固定 seed，确保可重复
    nx, ny = 64, 64
    Lx, Ly = 1.0, 1.0
    dx, dy = Lx / nx, Ly / ny
    grid = Grid2D(nx=nx, ny=ny, dx=dx, dy=dy)

    rng = np.random.default_rng(123)
    n = 200
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

    # 走一步，覆盖 get_state_snapshot -> compute_fields 的链路
    sim.step(0.01)
    snap = sim.get_state_snapshot()
    V = snap["V"]
    Ex = snap["Ex"]
    Ey = snap["Ey"]

    v_stats = {
        "V_min": float(np.min(V)),
        "V_max": float(np.max(V)),
        "V_mean": float(np.mean(V)),
    }
    e_stats = {
        "E_max_abs": float(np.max(np.abs(Ex))),
        "E_max_abs_y": float(np.max(np.abs(Ey))),
        "E_max_abs_mag": float(np.max(np.sqrt(Ex * Ex + Ey * Ey))),
    }

    # 验证 _safe_vmin_vmax 三种模式与返回范围合理性（不依赖 GUI ）
    vmin_auto, vmax_auto = _safe_vmin_vmax(V, mode="auto")
    vmin_pct, vmax_pct = _safe_vmin_vmax(V, mode="percentile")
    vmin_fixed, vmax_fixed = _safe_vmin_vmax(V, mode="fixed", vmin=-0.25, vmax=0.25)

    checks = [
        ("auto_range_has_width", (vmax_auto - vmin_auto) > 0),
        ("percentile_range_has_width", (vmax_pct - vmin_pct) > 0),
        ("fixed_range_exact", abs(vmin_fixed - (-0.25)) < 1e-12 and abs(vmax_fixed - 0.25) < 1e-12),
        ("percentile_within_global", vmin_pct >= v_stats["V_min"] - 1e-9 and vmax_pct <= v_stats["V_max"] + 1e-9),
    ]

    ok = all(b for _, b in checks)
    print("visualize_stats check:", "OK" if ok else "FAIL")
    for name, b in checks:
        print(f" - {name}: {b}")

    print("V stats:", v_stats)
    print("E stats:", e_stats)
    print("v_range_mode auto:", (vmin_auto, vmax_auto))
    print("v_range_mode percentile:", (vmin_pct, vmax_pct))
    print("v_range_mode fixed:", (vmin_fixed, vmax_fixed))

    if not ok:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
