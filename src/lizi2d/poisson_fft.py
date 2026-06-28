from __future__ import annotations

import numpy as np


def _fft2_circular(a: np.ndarray) -> np.ndarray:
    """
    计算 2D FFT（周期域默认等价于循环卷积的频域表示）。
    """
    return np.fft.fft2(a)


def solve_poisson_via_discrete_greens_function_kernel(
    rho: np.ndarray,
    dx: float,
    dy: float,
    *,
    eps: float = 1e-12,
) -> np.ndarray:
    """
    使用 FFT 在周期域中求解离散 Poisson 方程（用于计算电势 V）。

    方程：
      ∇^2 V = -rho

    在傅里叶空间：
      V_hat(k) = rho_hat(k) / (kx^2 + ky^2)
    （注意：这里是“光谱式 Poisson 求解”，具体离散约定可能与严格有限差分特征值略有差别）

    说明/注意事项：
    - 这是一个光谱求解器：在 k 空间中使用连续拉普拉斯形式
    - 对于验证通常足够稳定（网格较细时更可靠）
    - 令 k=0 模为 0，用于固定规范：电势仅相差常数无物理意义

    返回：
      V: shape = (nx, ny)
    """
    nx, ny = rho.shape

    rho_hat = np.fft.fft2(rho)

    kx = 2.0 * np.pi * np.fft.fftfreq(nx, d=dx)  # shape (nx,)
    ky = 2.0 * np.pi * np.fft.fftfreq(ny, d=dy)  # shape (ny,)

    kx2 = kx[:, None] ** 2
    ky2 = ky[None, :] ** 2
    k2 = kx2 + ky2

    V_hat = np.zeros_like(rho_hat, dtype=np.complex128)

    # k=0 时避免除以 0（相当于固定电势常数项）
    mask = k2 > eps
    V_hat[mask] = rho_hat[mask] / (k2[mask])

    V = np.fft.ifft2(V_hat).real
    return V


def compute_e_from_potential_periodic(
    V: np.ndarray,
    dx: float,
    dy: float,
) -> tuple[np.ndarray, np.ndarray]:
    """
    在周期边界下使用中心差分计算电场：
      E = -∇V

    离散形式：
    - E_x(i,j) = -(V(i+1,j)-V(i-1,j)) / (2dx)
    - E_y(i,j) = -(V(i,j+1)-V(i,j-1)) / (2dy)

    返回：
      Ex, Ey: 均为 shape = (nx, ny)
    """
    Vx_p = np.roll(V, shift=-1, axis=0)
    Vx_m = np.roll(V, shift=1, axis=0)
    Vy_p = np.roll(V, shift=-1, axis=1)
    Vy_m = np.roll(V, shift=1, axis=1)

    Ex = -(Vx_p - Vx_m) / (2.0 * dx)
    Ey = -(Vy_p - Vy_m) / (2.0 * dy)
    return Ex, Ey
