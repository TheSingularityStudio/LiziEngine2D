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
    将定义在网格节点上的电场矢量场（Ex, Ey）通过双线性插值（gather）
    采样到粒子位置，得到粒子的受力分量（q=1 时 F=E）。

    前提/假设：
    - 粒子坐标 (x,y) 在周期域内（越界后自动按周期包裹）
    - Ex/Ey 的 shape 为 (nx, ny)
    - 插值权重体系与 scatter_unit_charges_to_grid 保持一致（双线性权重）

    返回：
      fx, fy：shape = (N,) 的粒子电场采样值
    """
    nx, ny = grid.shape

    # 周期性包裹粒子坐标
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

    # 双线性插值
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
