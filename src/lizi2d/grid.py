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
        Wrap continuous coordinates (x, y) to periodic domain [0, Lx) x [0, Ly).
        """
        lx = self.nx * self.dx
        ly = self.ny * self.dy
        return (x % lx, y % ly)

    def world_to_grid(self, x: float, y: float) -> tuple[float, float]:
        """
        Convert world coords (x, y) to grid continuous indices (gx, gy).
        gx in [0, nx), gy in [0, ny) under periodic wrapping.
        """
        xw, yw = self.periodic_wrap(x, y)
        gx = xw / self.dx
        gy = yw / self.dy
        return gx, gy

    def grid_to_world(self, i: int, j: int) -> tuple[float, float]:
        """
        Convert integer grid indices to world coords at the grid node.
        """
        x = i * self.dx
        y = j * self.dy
        return x, y

    def bilinear_weights(self, gx: float, gy: float) -> tuple[int, int, int, int, float, float, float, float]:
        """
        For continuous indices gx,gy, return:
          i0,i1,j0,j1 and weights wx0,wx1, wy0,wy1 for bilinear interpolation.

        Assumes periodic wrap: gx in [0,nx), gy in [0,ny).
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
