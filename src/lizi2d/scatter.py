from __future__ import annotations

import numpy as np

from .grid import Grid2D
from .particles import ParticleState


def scatter_unit_charges_to_grid(
    grid: Grid2D,
    particles: ParticleState,
) -> np.ndarray:
    """
    Create charge density rho on grid nodes using bilinear weights.

    Assumptions:
    - Each particle has charge q=1.
    - Periodic wrapping is used.
    - rho returned has shape (nx, ny).

    Note:
    - Depending on your desired normalization of the discrete Poisson equation,
      you may want to divide by cell area or multiply by constants later.
    """
    nx, ny = grid.shape
    rho = np.zeros((nx, ny), dtype=np.float64)

    # Continuous grid indices
    gx = particles.x / grid.dx
    gy = particles.y / grid.dy

    # Periodic wrap of continuous indices
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

    # Bilinear scatter (gather/scatter consistent with Grid2D.bilinear_weights)
    # rho[i0, j0] += wx0*wy0
    np.add.at(rho, (i0, j0), wx0 * wy0)
    np.add.at(rho, (i1, j0), wx1 * wy0)
    np.add.at(rho, (i0, j1), wx0 * wy1)
    np.add.at(rho, (i1, j1), wx1 * wy1)

    return rho
