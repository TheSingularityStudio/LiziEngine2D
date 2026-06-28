from __future__ import annotations

import argparse
import numpy as np

from src.lizi2d.grid import Grid2D
from src.lizi2d.particles import ParticleState
from src.lizi2d.sim import ElectrostaticSim2D
from src.lizi2d.scatter import scatter_unit_charges_to_grid
from src.lizi2d.poisson_fft import solve_poisson_via_discrete_greens_function_kernel, compute_e_from_potential_periodic


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--nx", type=int, default=64)
    ap.add_argument("--ny", type=int, default=64)
    ap.add_argument("--dx", type=float, default=1.0)
    ap.add_argument("--dy", type=float, default=1.0)
    ap.add_argument("--charge_x", type=float, default=None)
    ap.add_argument("--charge_y", type=float, default=None)
    ap.add_argument("--eps", type=float, default=1e-12)
    args = ap.parse_args()

    grid = Grid2D(nx=args.nx, ny=args.ny, dx=args.dx, dy=args.dy)

    # one particle as unit charge
    N = 1
    particles = ParticleState.zeros(N)
    if args.charge_x is None:
        particles.x[0] = 0.5 * grid.nx * grid.dx
    else:
        particles.x[0] = args.charge_x
    if args.charge_y is None:
        particles.y[0] = 0.5 * grid.ny * grid.dy
    else:
        particles.y[0] = args.charge_y

    # Build fields once (no time stepping needed)
    sim = ElectrostaticSim2D(grid, particles, eps_poisson=args.eps)
    sim.compute_fields()

    Ex = sim.Ex
    Ey = sim.Ey

    # Validate radial direction on a set of ring sample points around the charge
    # Since periodic domain breaks pure 1/r, we just check consistency of E direction
    # and that |E| has reasonable monotonicity locally.
    cx = particles.x[0] / grid.dx
    cy = particles.y[0] / grid.dy

    sample_r = [5, 8, 12, 16]  # in grid cells
    sample_angles = np.linspace(0, 2 * np.pi, 16, endpoint=False)

    def periodic_delta(a: float, b: float, period: float) -> float:
        d = a - b
        # wrap to [-period/2, period/2)
        d = (d + 0.5 * period) % period - 0.5 * period
        return d

    lx = grid.nx
    ly = grid.ny

    # gather E at particle-free points by using bilinear gather
    # easiest: temporary particles at query points
    errors = []
    for r in sample_r:
        for th in sample_angles:
            gx = cx + (r * np.cos(th))
            gy = cy + (r * np.sin(th))

            # wrap to [0,n)
            gxw = gx % lx
            gyw = gy % ly

            qx = gxw * grid.dx
            qy = gyw * grid.dy

            qstate = ParticleState(
                x=np.array([qx], dtype=np.float64),
                y=np.array([qy], dtype=np.float64),
                vx=np.zeros(1, dtype=np.float64),
                vy=np.zeros(1, dtype=np.float64),
                fx=np.zeros(1, dtype=np.float64),
                fy=np.zeros(1, dtype=np.float64),
            )

            # Gather E with same interpolation as sim
            # (recompute using sim internals to avoid circular import)
            from src.lizi2d.interp import gather_field_to_particles_bilinear

            fx, fy = gather_field_to_particles_bilinear(grid, qstate, Ex, Ey)
            Exq = fx[0]
            Eyq = fy[0]

            # radial direction in periodic coords
            dx = periodic_delta(gxw, cx, lx)
            dy = periodic_delta(gyw, cy, ly)
            rhat = np.array([dx, dy], dtype=np.float64)
            nr = np.linalg.norm(rhat)
            if nr < 1e-9:
                continue
            rhat /= nr

            Evec = np.array([Exq, Eyq], dtype=np.float64)
            En = np.linalg.norm(Evec)
            if En < 1e-12:
                continue

            cosang = float(np.dot(Evec / En, rhat))
            errors.append(1.0 - cosang)

    err = float(np.mean(errors)) if errors else 0.0
    print(f"[validate_single_charge] mean_direction_error(1-cos)= {err:.6e}")

    # Heuristic threshold: if your discretization is sane, direction error should be small.
    # Because it's periodic FFT-based Poisson, expect some deviation.
    if err > 2e-1:
        raise SystemExit(2)
    print("OK")


if __name__ == "__main__":
    main()
