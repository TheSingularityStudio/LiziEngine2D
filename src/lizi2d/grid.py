from __future__ import annotations

from dataclasses import dataclass
import numpy as np


@dataclass(frozen=True)
class Grid2D:
    nx: int
    ny: int
    dx: float
    dy: float

    @property
    def shape(self) -> tuple[int, int]:
        return (self.nx, self.ny)

    def periodic_wrap(self, x: float, y: float) -> tuple[float, float]:
        """
        将连续坐标 (x, y) 进行周期性包裹到区域：
        [0, Lx) x [0, Ly)
        """
        lx = self.nx * self.dx
        ly = self.ny * self.dy
        return (x % lx, y % ly)

    def world_to_grid(self, x: float, y: float) -> tuple[float, float]:
        """
        将世界坐标 (x, y) 转换为网格连续索引 (gx, gy)。
        在周期性包裹后：
          gx 属于 [0, nx)，gy 属于 [0, ny)
        """
        xw, yw = self.periodic_wrap(x, y)
        gx = xw / self.dx
        gy = yw / self.dy
        return gx, gy

    def grid_to_world(self, i: int, j: int) -> tuple[float, float]:
        """
        将整数网格索引 (i, j) 转回世界坐标（网格点处）。
        """
        x = i * self.dx
        y = j * self.dy
        return x, y

    def bilinear_weights(
        self,
        gx: float,
        gy: float,
    ) -> tuple[int, int, int, int, float, float, float, float]:
        """
        对连续索引 gx, gy，返回双线性插值所需的：
          i0, i1, j0, j1 以及权重 wx0, wx1, wy0, wy1

        默认使用周期性包裹：gx in [0,nx), gy in [0,ny)
        """
        i0 = int(np.floor(gx)) % self.nx
        j0 = int(np.floor(gy)) % self.ny

        i1 = (i0 + 1) % self.nx
        j1 = (j0 + 1) % self.ny

        fx = gx - np.floor(gx)
        fy = gy - np.floor(gy)

        wx0 = 1.0 - fx
        wx1 = fx
        wy0 = 1.0 - fy
        wy1 = fy

        return i0, i1, j0, j1, wx0, wx1, wy0, wy1
