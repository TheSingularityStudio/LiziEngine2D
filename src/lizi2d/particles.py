from __future__ import annotations

from dataclasses import dataclass
import numpy as np


@dataclass
class ParticleState:
    """
    粒子状态数据结构。

    x/y: 位置（连续世界坐标，周期边界内）
    vx/vy: 速度
    fx/fy: 受力（由电场计算得到）
    """
    x: np.ndarray  # shape (N,)
    y: np.ndarray  # shape (N,)
    vx: np.ndarray  # shape (N,)
    vy: np.ndarray  # shape (N,)
    fx: np.ndarray  # shape (N,)
    fy: np.ndarray  # shape (N,)

    @staticmethod
    def zeros(n: int, seed: int | None = None) -> "ParticleState":
        """
        创建 n 个粒子：位置初始化为随机值，速度/力初始化为 0。
        """
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
        """深拷贝一份粒子状态。"""
        return ParticleState(
            x=self.x.copy(),
            y=self.y.copy(),
            vx=self.vx.copy(),
            vy=self.vy.copy(),
            fx=self.fx.copy(),
            fy=self.fy.copy(),
        )

    def as_dict(self) -> dict[str, np.ndarray]:
        """导出为 dict，便于调试/序列化。"""
        return {"x": self.x, "y": self.y, "vx": self.vx, "vy": self.vy, "fx": self.fx, "fy": self.fy}
