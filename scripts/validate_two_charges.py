from __future__ import annotations

import argparse
import numpy as np

from src.lizi2d.grid import Grid2D
from src.lizi2d.particles import ParticleState
from src.lizi2d.sim import ElectrostaticSim2D
from src.lizi2d.interp import gather_field_to_particles_bilinear


def compute_E_for_charges(grid: Grid2D, charges: list[tuple[float, float, float]], *, eps: float) -> tuple[np.ndarray, np.ndarray]:
    particles = ParticleState.zeros(len(charges))
    for i, (x, y, _q) in enumerate(charges):
        particles.x[i] = x
        particles.y[i] = y

    # Unit-magnitude charges only in current engine; emulate negative by two runs and subtraction
    # For now, assume q is either +1 or -1.
    # We'll do:
    #   V = V(+1 set) - V(-1 set)
    pos = [(x, y, q) for (x, y, q) in charges if q > 0]
    neg = [(x, y, q) for (x, y, q) in charges if q < 0]

    def run_set(points: list[tuple[float, float, float]]) -> tuple[np.ndarray, np.ndarray]:
        if not points:
            return np.zeros(grid.shape), np.zeros(grid.shape)
        p = ParticleState.zeros(len(points))
        for j, (x, y, _q) in enumerate(points):
            p.x[j] = x
            p.y[j] = y
        sim = ElectrostaticSim2D(grid, p, eps_poisson=eps)
        sim.compute_fields()
        return sim.Ex, sim.Ey

    Ex_pos, Ey_pos = run_set(pos)
    Ex_neg, Ey_neg = run_set(neg)

    Ex = Ex_pos - Ex_neg
    Ey = Ey_pos - Ey_neg
    return Ex, Ey


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--nx", type=int, default=64)
    ap.add_argument("--ny", type=int, default=64)
    ap.add_argument("--dx", type=float, default=1.0)
    ap.add_argument("--dy", type=float, default=1.0)
    ap.add_argument("--eps", type=float, default=1e-12)
    args = ap.parse_args()

    grid = Grid2D(nx=args.nx, ny=args.ny, dx=args.dx, dy=args.dy)

    # Two +1 charges at fixed positions; compare with superposition:
    # E_total = E1 + E2
    x1, y1 = 0.25 * grid.nx * grid.dx, 0.5 * grid.ny * grid.dy
    x2, y2 = 0.75 * grid.nx * grid.dx, 0.5 * grid.ny * grid.dy

    Ex_total, Ey_total = compute_E_for_charges(
        grid, [(x1, y1, +1.0), (x2, y2, +1.0)], eps=args.eps
    )
    Ex1, Ey1 = compute_E_for_charges(grid, [(x1, y1, +1.0)], eps=args.eps)
    Ex2, Ey2 = compute_E_for_charges(grid, [(x2, y2, +1.0)], eps=args.eps)

    # Sample points and compare
    Nq = 200
    rng = np.random.default_rng(0)
    xs = rng.random(Nq) * grid.nx * grid.dx
    ys = rng.random(Nq) * grid.ny * grid.dy

    qstate = ParticleState.zeros(Nq)
    qstate.x = xs
    qstate.y = ys

    Exq_total, Eyq_total = gather_field_to_particles_bilinear(grid, qstate, Ex_total, Ey_total)
    Exq_1, Eyq_1 = gather_field_to_particles_bilinear(grid, qstate, Ex1, Ey1)
    Exq_2, Eyq_2 = gather_field_to_particles_bilinear(grid, qstate, Ex2, Ey2)

    Ex_pred = Exq_1 + Exq_2
    Ey_pred = Eyq_1 + Eyq_2

    # Relative L2 error
    denom = np.linalg.norm(Exq_total) + np.linalg.norm(Eyq_total) + 1e-15
    num = np.linalg.norm(Ex_pred - Exq_total) + np.linalg.norm(Ey_pred - Eyq_total)
    rel = float(num / denom)

    print(f"[validate_two_charges] relative_L2_error={rel:.6e}")
    if rel > 5e-2:
        raise SystemExit(2)
    print("OK")


if __name__ == "__main__":
    main()
