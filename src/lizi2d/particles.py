from __future__ import annotations

from dataclasses import dataclass
import numpy as np


@dataclass
class ParticleState:
    """
    Positions/velocities/forces are stored as float64 for stability in validation.
    For performance you can switch to float32 later.
    """
    x: np.ndarray  # shape (N,)
    y: np.ndarray  # shape (N,)
    vx: np.ndarray  # shape (N,)
    vy: np.ndarray  # shape (N,)
    fx: np.ndarray  # shape (N,)
    fy: np.ndarray  # shape (N,)

    @staticmethod
    def zeros(n: int, seed: int | None = None) -> "ParticleState":
        rng = np.random.default_rng(seed)
        x = rng.random(n)
        y = rng.random(n)
        return ParticleState(
            x=x.astype(np.float64),
            y=y.astype(np.float64),
            vx=np.zeros(n, dtype=np.float64),
            vy=np.zeros(n, dtype=np.float64),
            fx=np.zeros(n, dtype=np.float64),
            fy=np.zeros(n, dtype=np.float64),
        )

    def copy(self) -> "ParticleState":
        return ParticleState(
            x=self.x.copy(),
            y=self.y.copy(),
            vx=self.vx.copy(),
            vy=self.vy.copy(),
            fx=self.fx.copy(),
            fy=self.fy.copy(),
        )

    def as_dict(self) -> dict[str, np.ndarray]:
        return {"x": self.x, "y": self.y, "vx": self.vx, "vy": self.vy, "fx": self.fx, "fy": self.fy}
