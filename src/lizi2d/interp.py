from __future__ import annotations

import numpy as np

from .grid import Grid2D
from .particles import ParticleState


def gather_field_to_particles_bilinear(
    grid: Grid2D,
    particles: ParticleState,
    Ex: np.ndarray,
    Ey: np.ndarray,
) -> tuple[np.ndarray, np.ndarray]:
    """
    Bilinear gather of vector field defined on grid nodes to particle positions.

    Assumptions:
    - particles.x/y in continuous world coords on periodic domain.
    - Ex,Ey are shape (nx,ny).
    - Uses same weighting scheme as scatter_unit_charges_to_grid.

    Returns:
      fx, fy arrays shape (N,)
    """
    nx, ny = grid.shape

    # Periodic wrap particle coords
    lx = nx * grid.dx
    ly = ny * grid.dy
    xw = np.mod(particles.x, lx)
    yw = np.mod(particles.y, ly)
    gx = xw / grid.dx
    gy = yw / grid.dy

    i0 = np.floor(gx).astype(np.int64) % nx
    j0 = np.floor(gy).astype(np.int64) % ny
    i1 = (i0 + 1) % nx
    j1 = (j0 + 1) % ny

    fx = gx - np.floor(gx)
    fy = gy - np.floor(gy)

    wx0 = 1.0 - fx
    wx1 = fx
    wy0 = 1.0 - fy
    wy1 = fy

    # Bilinear interpolation
    Ex_p = (
        Ex[i0, j0] * wx0 * wy0
        + Ex[i1, j0] * wx1 * wy0
        + Ex[i0, j1] * wx0 * wy1
        + Ex[i1, j1] * wx1 * wy1
    )
    Ey_p = (
        Ey[i0, j0] * wx0 * wy0
        + Ey[i1, j0] * wx1 * wy0
        + Ey[i0, j1] * wx0 * wy1
        + Ey[i1, j1] * wx1 * wy1
    )
    return Ex_p, Ey_p
