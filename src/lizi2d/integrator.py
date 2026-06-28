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
    Semi-implicit Euler (a.k.a. symplectic Euler):
      v_{n+1} = v_n + a_n * dt
      x_{n+1} = x_n + v_{n+1} * dt

    Assumes particles.fx/fy already hold force at current step.
    With q=1 and m=1, a = F.
    """
    particles.vx = particles.vx + particles.fx * dt
    particles.vy = particles.vy + particles.fy * dt

    particles.x = particles.x + particles.vx * dt
    particles.y = particles.y + particles.vy * dt

    # periodic wrap
    xw, yw = grid.periodic_wrap(0.0, 0.0)  # just to compute lx/ly via Grid2D internals
    # compute directly
    lx = grid.nx * grid.dx
    ly = grid.ny * grid.dy
    particles.x = np.mod(particles.x, lx)
    particles.y = np.mod(particles.y, ly)

    return particles
