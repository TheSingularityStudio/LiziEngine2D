from __future__ import annotations

import numpy as np

from .grid import Grid2D
from .particles import ParticleState
from .scatter import scatter_unit_charges_to_grid
from .poisson_fft import solve_poisson_via_discrete_greens_function_kernel, compute_e_from_potential_periodic
from .interp import gather_field_to_particles_bilinear
from .integrator import step_half_implicit_euler


class ElectrostaticSim2D:
    """
    2D 静电（电场-粒子）CPU 模拟器（PIC 风格实现；单位电荷、单位质量）。

    每个时间步的计算流程：
      1) 将粒子散射到网格上，得到离散电荷密度 rho
      2) 在周期边界条件下求解 Poisson：通过 FFT 求得电势 V
      3) 在网格上计算电场：E = -∇V
      4) 将网格电场通过 gather 插值回粒子位置，得到粒子受力
         （由于 q=1，故 F = E）
      5) 使用半隐式欧拉对粒子积分更新（速度/位置）
    """

    def __init__(
        self,
        grid: Grid2D,
        particles: ParticleState,
        *,
        eps_poisson: float = 1e-12,
    ):
        """
        参数：
        - grid：计算网格
        - particles：粒子状态（位置、速度、力）
        - eps_poisson：Poisson 求解时的数值阈值（用于避免 k=0 除零）
        """
        self.grid = grid
        self.particles = particles
        self.eps_poisson = eps_poisson

        self.rho: np.ndarray | None = None
        self.V: np.ndarray | None = None
        self.Ex: np.ndarray | None = None
        self.Ey: np.ndarray | None = None

    def compute_fields(self) -> None:
        self.rho = scatter_unit_charges_to_grid(self.grid, self.particles)
        self.V = solve_poisson_via_discrete_greens_function_kernel(
            self.rho, self.grid.dx, self.grid.dy, eps=self.eps_poisson
        )
        self.Ex, self.Ey = compute_e_from_potential_periodic(self.V, self.grid.dx, self.grid.dy)

        # 将网格电场（对应 E）gather 回粒子，得到粒子受力：
        # 由于 q=1 且 m=1，故 F = E
        fx, fy = gather_field_to_particles_bilinear(self.grid, self.particles, self.Ex, self.Ey)
        self.particles.fx = fx
        self.particles.fy = fy

    def step(self, dt: float) -> None:
        self.compute_fields()
        self.particles = step_half_implicit_euler(self.grid, self.particles, dt)

    def run(self, dt: float, steps: int, *, record_every: int = 1) -> dict[str, np.ndarray]:
        frames: list[np.ndarray] = []
        for s in range(steps):
            self.step(dt)
            if (s % record_every) == 0:
                # 记录（snapshots）粒子位置，用于后续可视化/调试
                frames.append(np.stack([self.particles.x, self.particles.y], axis=1))
        return {"positions": np.array(frames, dtype=np.float64)}

    def get_state_snapshot(self) -> dict[str, np.ndarray]:
        """
        获取当前仿真状态快照（用于可视化/调试）。
        要求：compute_fields() 至少执行过一次，确保 V/Ex/Ey 存在。
        """
        if self.V is None or self.Ex is None or self.Ey is None:
            # 若用户直接在首次 step 前调用，则强制计算一次
            self.compute_fields()

        return {
            "x": self.particles.x.copy(),
            "y": self.particles.y.copy(),
            "V": self.V.copy(),   # (nx, ny)
            "Ex": self.Ex.copy(), # (nx, ny)
            "Ey": self.Ey.copy(), # (nx, ny)
        }
