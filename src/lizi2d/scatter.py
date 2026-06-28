from __future__ import annotations

import numpy as np

from .grid import Grid2D
from .particles import ParticleState


def scatter_unit_charges_to_grid(
    grid: Grid2D,
    particles: ParticleState,
) -> np.ndarray:
    """
    将单位电荷粒子散射（scatter）到网格节点上，生成离散电荷密度 rho。

    使用方法：
    - 双线性加权（bilinear weights），保证与后续从网格到粒子的双线性 gather 一致
    - 周期性边界：超出边界的粒子坐标进行周期包裹

    返回：
    - rho: shape = (nx, ny)，每个粒子 q=1 的贡献按权重分摊到四个邻近格点
    """
    nx, ny = grid.shape
    rho = np.zeros((nx, ny), dtype=np.float64)

    # 连续网格索引
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

    # 双线性权重
    wx0 = 1.0 - fx
    wx1 = fx
    wy0 = 1.0 - fy
    wy1 = fy

    # 双线性散射：每个粒子贡献分摊到 (i0,j0),(i1,j0),(i0,j1),(i1,j1)
    np.add.at(rho, (i0, j0), wx0 * wy0)
    np.add.at(rho, (i1, j0), wx1 * wy0)
    np.add.at(rho, (i0, j1), wx0 * wy1)
    np.add.at(rho, (i1, j1), wx1 * wy1)

    return rho
