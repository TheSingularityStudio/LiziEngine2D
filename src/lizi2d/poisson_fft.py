from __future__ import annotations

import numpy as np


def _fft2_circular(a: np.ndarray) -> np.ndarray:
    return np.fft.fft2(a)


def solve_poisson_via_discrete_greens_function_kernel(
    rho: np.ndarray,
    dx: float,
    dy: float,
    *,
    eps: float = 1e-12,
) -> np.ndarray:
    """
    Solve discrete Poisson for periodic domain using FFT:

      ∇^2 V = -rho

    In Fourier space:
      V_hat(k) = rho_hat(k) / (kx^2 + ky^2)    (up to discretization conventions)

    Notes / caveats:
    - This is a spectral Poisson solver (continuous Laplacian in k-space),
      not the exact finite-difference Laplacian eigenvalues.
    - For small dx/dy this is typically acceptable for validation.
    - The k=0 mode is set to 0 to fix gauge (potential defined up to constant).

    Returns:
      V: shape (nx, ny)
    """
    nx, ny = rho.shape

    rho_hat = np.fft.fft2(rho)

    kx = 2.0 * np.pi * np.fft.fftfreq(nx, d=dx)  # shape (nx,)
    ky = 2.0 * np.pi * np.fft.fftfreq(ny, d=dy)  # shape (ny,)

    kx2 = kx[:, None] ** 2
    ky2 = ky[None, :] ** 2
    k2 = kx2 + ky2

    V_hat = np.zeros_like(rho_hat, dtype=np.complex128)

    # Avoid division by zero at k=0 (gauge)
    mask = k2 > eps
    V_hat[mask] = rho_hat[mask] / (k2[mask])

    V = np.fft.ifft2(V_hat).real
    return V


def compute_e_from_potential_periodic(V: np.ndarray, dx: float, dy: float) -> tuple[np.ndarray, np.ndarray]:
    """
    Compute E = -∇V using periodic central differences.

    E_x(i,j) = -(V(i+1,j)-V(i-1,j))/(2dx)
    E_y(i,j) = -(V(i,j+1)-V(i,j-1))/(2dy)

    Returns:
      Ex, Ey each shape (nx, ny)
    """
    Vx_p = np.roll(V, shift=-1, axis=0)
    Vx_m = np.roll(V, shift=1, axis=0)
    Vy_p = np.roll(V, shift=-1, axis=1)
    Vy_m = np.roll(V, shift=1, axis=1)

    Ex = -(Vx_p - Vx_m) / (2.0 * dx)
    Ey = -(Vy_p - Vy_m) / (2.0 * dy)
    return Ex, Ey
