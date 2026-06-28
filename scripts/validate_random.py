from __future__ import annotations

import argparse
import numpy as np

from src.lizi2d.grid import Grid2D
from src.lizi2d.particles import ParticleState
from src.lizi2d.sim import ElectrostaticSim2D


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--nx", type=int, default=64)
    ap.add_argument("--ny", type=int, default=64)
    ap.add_argument("--dx", type=float, default=1.0)
    ap.add_argument("--dy", type=float, default=1.0)
    ap.add_argument("--N", type=int, default=200)
    ap.add_argument("--steps", type=int, default=20)
    ap.add_argument("--dt", type=float, default=0.05)
    ap.add_argument("--eps", type=float, default=1e-12)
    ap.add_argument("--seed", type=int, default=0)
    args = ap.parse_args()

    grid = Grid2D(nx=args.nx, ny=args.ny, dx=args.dx, dy=args.dy)

    particles = ParticleState.zeros(args.N, seed=args.seed)
    # 使用较小的随机初速度来触发积分过程（避免全零速度导致的“表面正确”）
    rng = np.random.default_rng(args.seed + 123)
    particles.vx = (rng.random(args.N) - 0.5) * 0.02
    particles.vy = (rng.random(args.N) - 0.5) * 0.02

    sim = ElectrostaticSim2D(grid, particles, eps_poisson=args.eps)

    max_speed_hist: list[float] = []
    for _ in range(args.steps):
        sim.step(args.dt)
        speed = np.sqrt(sim.particles.vx**2 + sim.particles.vy**2)
        max_speed_hist.append(float(np.max(speed)))

    max_speed = max(max_speed_hist)
    print(f"[validate_random] 多步最大速度 = {max_speed:.6e}")

    # 启发式稳定性阈值（后续可按需求调节）
    if max_speed > 50.0:
        raise SystemExit(2)

    print("OK：随机初始条件下的数值稳定性验证通过")


if __name__ == "__main__":
    main()
