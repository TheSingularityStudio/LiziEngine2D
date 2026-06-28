from __future__ import annotations

import argparse
import numpy as np

from src.lizi2d.grid import Grid2D
from src.lizi2d.particles import ParticleState
from src.lizi2d.sim import ElectrostaticSim2D
from src.lizi2d.poisson_fft import solve_poisson_via_discrete_greens_function_kernel, compute_e_from_potential_periodic
from src.lizi2d.interp import gather_field_to_particles_bilinear


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--nx", type=int, default=64)
    ap.add_argument("--ny", type=int, default=64)
    ap.add_argument("--dx", type=float, default=1.0)
    ap.add_argument("--dy", type=float, default=1.0)
    ap.add_argument("--charge_x", type=float, default=None)
    ap.add_argument("--charge_y", type=float, default=None)
    ap.add_argument("--eps", type=float, default=1e-12)
    args = ap.parse_args()

    grid = Grid2D(nx=args.nx, ny=args.ny, dx=args.dx, dy=args.dy)

    # 1 个单位电荷粒子（q=1）
    N = 1
    particles = ParticleState.zeros(N)
    if args.charge_x is None:
        particles.x[0] = 0.5 * grid.nx * grid.dx
    else:
        particles.x[0] = args.charge_x
    if args.charge_y is None:
        particles.y[0] = 0.5 * grid.ny * grid.dy
    else:
        particles.y[0] = args.charge_y

    # 只计算一次电场（此验证不需要时间步进）
    sim = ElectrostaticSim2D(grid, particles, eps_poisson=args.eps)
    sim.compute_fields()

    Ex = sim.Ex
    Ey = sim.Ey

    # 在电荷周围的若干“环形采样点”上验证：
    # 1) 电场方向是否与径向方向一致
    # 2) 由于周期域破坏了严格的 1/r 形式，这里不做严格幅值对比
    #    仅检查方向一致性与局部合理性
    cx = particles.x[0] / grid.dx
    cy = particles.y[0] / grid.dy

    sample_r = [5, 8, 12, 16]  # 采样半径（网格单元数）
    sample_angles = np.linspace(0, 2 * np.pi, 16, endpoint=False)

    def periodic_delta(a: float, b: float, period: float) -> float:
        d = a - b
        # 将差值环绕到 [-period/2, period/2)
        d = (d + 0.5 * period) % period - 0.5 * period
        return d

    lx = grid.nx
    ly = grid.ny

    # 在粒子“无电荷”的查询点处采样 E：
    # 采用与模拟相同的双线性 gather
    errors: list[float] = []
    for r in sample_r:
        for th in sample_angles:
            gx = cx + (r * np.cos(th))
            gy = cy + (r * np.sin(th))

            # 坐标按周期包裹到 [0, n)
            gxw = gx % lx
            gyw = gy % ly

            qx = gxw * grid.dx
            qy = gyw * grid.dy

            qstate = ParticleState(
                x=np.array([qx], dtype=np.float64),
                y=np.array([qy], dtype=np.float64),
                vx=np.zeros(1, dtype=np.float64),
                vy=np.zeros(1, dtype=np.float64),
                fx=np.zeros(1, dtype=np.float64),
                fy=np.zeros(1, dtype=np.float64),
            )

            # 使用与 sim 相同的插值方式采样 E
            fx, fy = gather_field_to_particles_bilinear(grid, qstate, Ex, Ey)
            Exq = fx[0]
            Eyq = fy[0]

            # 计算在周期坐标系下的径向方向单位向量
            dxr = periodic_delta(gxw, cx, lx)
            dyr = periodic_delta(gyw, cy, ly)
            rhat = np.array([dxr, dyr], dtype=np.float64)
            nr = np.linalg.norm(rhat)
            if nr < 1e-9:
                continue
            rhat /= nr

            Evec = np.array([Exq, Eyq], dtype=np.float64)
            En = np.linalg.norm(Evec)
            if En < 1e-12:
                continue

            cosang = float(np.dot(Evec / En, rhat))
            errors.append(1.0 - cosang)

    err = float(np.mean(errors)) if errors else 0.0
    print(f"[validate_single_charge] 平均方向误差(1-cos)= {err:.6e}")

    # 启发式阈值：
    # 如果离散/插值/梯度实现合理，方向误差应当较小。
    # 由于是周期域 FFT-based Poisson，允许存在一定偏差。
    if err > 2e-1:
        raise SystemExit(2)

    print("OK：单点电荷方向一致性验证通过")


if __name__ == "__main__":
    main()
