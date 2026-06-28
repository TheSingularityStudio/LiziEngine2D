from __future__ import annotations

import numpy as np
import matplotlib.pyplot as plt


def _safe_vmin_vmax(V: np.ndarray, mode: str = "fixed", vmin: float | None = None, vmax: float | None = None) -> tuple[float, float]:
    if mode == "fixed":
        if vmin is None or vmax is None:
            raise ValueError("mode='fixed' 需要提供 vmin/vmax。")
        return float(vmin), float(vmax)

    if mode == "auto":
        vmin_ = float(np.min(V))
        vmax_ = float(np.max(V))
        if vmin_ == vmax_:
            vmax_ = vmin_ + 1e-12
        return vmin_, vmax_

    if mode == "percentile":
        lo = float(np.percentile(V, 1.0))
        hi = float(np.percentile(V, 99.0))
        if lo == hi:
            hi = lo + 1e-12
        return lo, hi

    raise ValueError(f"未知 v_range_mode: {mode}")


def visualize_matplotlib_2d(
    sim,
    *,
    dt: float,
    steps_per_frame: int = 1,
    frames: int = 200,
    interval_ms: int = 30,
    show_particles: bool = True,
    show_v: bool = True,
    show_e: bool = False,
    e_stride: int = 6,
    e_scale: float = 0.25,
    v_range_mode: str = "percentile",
    vmin: float | None = None,
    vmax: float | None = None,
    figsize: tuple[int, int] = (8, 6),
):
    """
    使用 matplotlib 做最小可视化（MVP）。

    预期 sim 是 ElectrostaticSim2D，至少需要：
    - sim.grid (Grid2D)
    - sim.step(dt)
    - sim.get_state_snapshot() -> {"x","y","V","Ex","Ey"}
    """
    grid = sim.grid

    fig, ax = plt.subplots(figsize=figsize)
    ax.set_aspect("equal", adjustable="box")
    ax.set_xlabel("x")
    ax.set_ylabel("y")

    # 初始化数据
    snap = sim.get_state_snapshot()
    x = snap["x"]
    y = snap["y"]
    V = snap["V"]
    Ex = snap["Ex"]
    Ey = snap["Ey"]

    # 画背景（V 热力图）
    Lx = grid.nx * grid.dx
    Ly = grid.ny * grid.dy

    extent = (0.0, Lx, 0.0, Ly)
    Vmin, Vmax = _safe_vmin_vmax(V, mode=v_range_mode, vmin=vmin, vmax=vmax)

    im = None
    if show_v:
        im = ax.imshow(
            V.T,
            origin="lower",
            extent=extent,
            cmap="RdBu_r",
            vmin=Vmin,
            vmax=Vmax,
            interpolation="bilinear",
        )
        cbar = fig.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
        cbar.set_label("V（电势）")

    # 粒子
    scat = None
    if show_particles:
        scat = ax.scatter(x, y, s=18, c="k", alpha=0.85)

    # 电场箭头
    quiv = None
    if show_e:
        xs = np.linspace(0.0, Lx, grid.nx, endpoint=False)
        ys = np.linspace(0.0, Ly, grid.ny, endpoint=False)
        X, Y = np.meshgrid(xs, ys, indexing="ij")

        Ex_s = Ex[::e_stride, ::e_stride]
        Ey_s = Ey[::e_stride, ::e_stride]
        X_s = X[::e_stride, ::e_stride]
        Y_s = Y[::e_stride, ::e_stride]

        quiv = ax.quiver(
            X_s,
            Y_s,
            Ex_s,
            Ey_s,
            angles="xy",
            scale_units="xy",
            scale=1.0 / max(e_scale, 1e-12),
            width=0.003,
            color="tab:green",
        )

    # 标题信息
    title = ax.set_title("2D Electrostatic PIC (matplotlib MVP)")

    def draw_frame(frame_idx: int):
        nonlocal im, scat, quiv

        for _ in range(steps_per_frame):
            sim.step(dt)

        snap2 = sim.get_state_snapshot()
        x2 = snap2["x"]
        y2 = snap2["y"]
        V2 = snap2["V"]
        Ex2 = snap2["Ex"]
        Ey2 = snap2["Ey"]

        if show_v and im is not None:
            im.set_data(V2.T)

        if show_particles and scat is not None:
            scat.set_offsets(np.stack([x2, y2], axis=1))

        if show_e and quiv is not None:
            xs = np.linspace(0.0, Lx, grid.nx, endpoint=False)
            ys = np.linspace(0.0, Ly, grid.ny, endpoint=False)
            X, Y = np.meshgrid(xs, ys, indexing="ij")

            Ex_s = Ex2[::e_stride, ::e_stride]
            Ey_s = Ey2[::e_stride, ::e_stride]
            X_s = X[::e_stride, ::e_stride]
            Y_s = Y[::e_stride, ::e_stride]

            # 重新绘制 quiver（matplotlib 对 set_UVC 支持更稳定）
            quiv.set_UVC(Ex_s, Ey_s)

        title.set_text(f"frame {frame_idx}/{frames}")

        return []

    # 手动用事件循环驱动（避免额外动画依赖）
    for fi in range(frames):
        draw_frame(fi)
        plt.pause(interval_ms / 1000.0)

    plt.show()
