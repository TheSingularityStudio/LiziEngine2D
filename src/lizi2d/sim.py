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
    PIC-like electrostatic simulator (unit charges, unit mass).
    Pipeline per step:
      1) scatter particles -> rho grid
      2) solve Poisson (periodic) -> V grid via FFT spectral method
      3) compute E = -grad(V) on grid
      4) gather E to particle positions -> particle forces (q=1 => F=E)
      5) integrate particles with semi-implicit Euler
    """

    def __init__(
        self,
        grid: Grid2D,
        particles: ParticleState,
        *,
        eps_poisson: float = 1e-12,
    ):
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

        # Gather forces to particles: F = E (q=1)
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
                # snapshot particle positions
                frames.append(np.stack([self.particles.x, self.particles.y], axis=1))
        return {"positions": np.array(frames, dtype=np.float64)}
