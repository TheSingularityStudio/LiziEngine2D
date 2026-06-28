from __future__ import annotations

import tkinter as tk
from dataclasses import dataclass

import numpy as np

# -------- Font fix (避免中文 glyph 缺失 warning) --------
# Tkinter/Matplotlib 默认可能使用 DejaVu Sans（不含中文 glyph）。
# 在 Windows 上优先选择可用的中文字体：如 Microsoft YaHei / SimHei。
import matplotlib
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.figure import Figure
from matplotlib import font_manager as _fm

def _try_set_cn_font():
    candidates = [
        "Microsoft YaHei",  # Windows
        "Microsoft JhengHei",
        "SimHei",           # 黑体
        "PingFang SC",      # macOS
        "Noto Sans CJK SC", # 常见开源
        "Source Han Sans SC",
    ]
    available = {f.name for f in _fm.fontManager.ttflist}

    for name in candidates:
        if name in available:
            matplotlib.rcParams["font.family"] = ["sans-serif"]
            matplotlib.rcParams["font.sans-serif"] = [name]
            return name

    # 找不到中文字体时不强制修改（避免引入其它副作用）
    return None

_TRIED_FONT = _try_set_cn_font()


@dataclass
class TkViewerConfig:
    dt: float = 0.02
    steps_per_frame: int = 1

    show_v: bool = True
    show_particles: bool = True
    show_e: bool = True

    e_stride: int = 6
    e_scale: float = 0.25  # 与 visualize_matplotlib_2d 保持语义一致

    # 先只支持 percentile（实现简单且够用）
    v_percentile_lo: float = 1.0
    v_percentile_hi: float = 99.0

    # 画面尺寸
    figsize: tuple[float, float] = (8, 6)

    # GUI 刷新（ms），真正“物理步数”由 steps_per_frame 决定
    interval_ms: int = 25


def _safe_percentile_vmin_vmax(V: np.ndarray, lo: float, hi: float) -> tuple[float, float]:
    vmin = float(np.percentile(V, lo))
    vmax = float(np.percentile(V, hi))
    if vmin == vmax:
        vmax = vmin + 1e-12
    return vmin, vmax


class TkinterElectrostaticViewer2D:
    """
    Tkinter + matplotlib 嵌入式渲染窗口（CPU 仿真）。

    实现方式：
    - UI 线程驱动 after(interval_ms, tick)
    - tick 内执行 steps_per_frame 次 sim.step(dt)
    - 读 sim.get_state_snapshot() 更新热力图/散点/箭头
    """

    def __init__(self, master: tk.Tk, sim, *, config: TkViewerConfig):
        self.master = master
        self.sim = sim
        self.cfg = config

        self.grid = sim.grid
        self.nx, self.ny = self.grid.nx, self.grid.ny
        self.Lx = self.nx * self.grid.dx
        self.Ly = self.ny * self.grid.dy

        # ---------- Tk variables ----------
        self.var_play = tk.BooleanVar(value=True)

        self.var_dt = tk.DoubleVar(value=float(self.cfg.dt))
        self.var_steps_per_frame = tk.IntVar(value=int(self.cfg.steps_per_frame))

        self.var_show_v = tk.BooleanVar(value=bool(self.cfg.show_v))
        self.var_show_particles = tk.BooleanVar(value=bool(self.cfg.show_particles))
        self.var_show_e = tk.BooleanVar(value=bool(self.cfg.show_e))

        self.var_e_stride = tk.IntVar(value=int(self.cfg.e_stride))
        self.var_e_scale = tk.DoubleVar(value=float(self.cfg.e_scale))

        self.var_v_lo = tk.DoubleVar(value=float(self.cfg.v_percentile_lo))
        self.var_v_hi = tk.DoubleVar(value=float(self.cfg.v_percentile_hi))

        # ---------- matplotlib figure ----------
        self.fig = Figure(figsize=self.cfg.figsize, dpi=100)
        self.ax = self.fig.add_subplot(111)
        self.ax.set_aspect("equal", adjustable="box")
        self.ax.set_xlabel("x")
        self.ax.set_ylabel("y")

        # 初始化 snapshot（确保 sim 里 V/Ex/Ey 存在）
        snap = self.sim.get_state_snapshot()
        x = snap["x"]
        y = snap["y"]
        V = snap["V"]
        Ex = snap["Ex"]
        Ey = snap["Ey"]

        extent = (0.0, self.Lx, 0.0, self.Ly)
        vmin, vmax = _safe_percentile_vmin_vmax(V, self.cfg.v_percentile_lo, self.cfg.v_percentile_hi)

        # artists
        self._im = None
        self._scat = None
        self._quiv = None
        self._cbar = None

        # V heatmap
        if self.var_show_v.get():
            self._im = self.ax.imshow(
                V.T,
                origin="lower",
                extent=extent,
                cmap="RdBu_r",
                vmin=vmin,
                vmax=vmax,
                interpolation="bilinear",
            )
            self._cbar = self.fig.colorbar(self._im, ax=self.ax, fraction=0.046, pad=0.04)
            self._cbar.set_label("V（电势）")

        # particles
        if self.var_show_particles.get():
            self._scat = self.ax.scatter(x, y, s=18, c="k", alpha=0.85)

        # quiver
        if self.var_show_e.get():
            self._quiv = self._create_quiver(Ex, Ey)

        # title / status
        self._title = self.ax.set_title("2D Electrostatic PIC (Tkinter MVP)")
        self._status_text = self.ax.text(0.01, 0.99, "", transform=self.ax.transAxes, va="top")

        # ---------- canvas ----------
        self.canvas = FigureCanvasTkAgg(self.fig, master=self.master)
        self.canvas_widget = self.canvas.get_tk_widget()
        self.canvas_widget.pack(side=tk.TOP, fill=tk.BOTH, expand=1)

        # ---------- controls ----------
        self._build_controls()

        # loop
        self._running = True
        self._tick()

    def _build_controls(self) -> None:
        panel = tk.Frame(self.master)
        panel.pack(side=tk.BOTTOM, fill=tk.X)

        btn = tk.Button(panel, text="Pause" if self.var_play.get() else "Play", command=self._toggle_play)
        btn.pack(side=tk.LEFT, padx=4, pady=4)
        self._play_btn = btn

        # helper: labeled slider
        def mk_labeled_scale(label: str, var, from_, to, resolution, pack_side=tk.LEFT):
            fr = tk.Frame(panel)
            fr.pack(side=pack_side, padx=6)
            tk.Label(fr, text=label).pack(side=tk.TOP)
            scale = tk.Scale(
                fr,
                variable=var,
                from_=from_,
                to=to,
                orient=tk.HORIZONTAL,
                resolution=resolution,
                length=180,
            )
            scale.pack(side=tk.TOP)

        mk_labeled_scale("dt", self.var_dt, 0.001, 0.1, 0.001)
        mk_labeled_scale("steps", self.var_steps_per_frame, 1, 10, 1)

        # show switches
        def mk_check(label: str, var):
            cb = tk.Checkbutton(panel, text=label, variable=var, command=self._sync_visibility)
            cb.pack(side=tk.LEFT, padx=6)

        mk_check("Show V", self.var_show_v)
        mk_check("Particles", self.var_show_particles)
        mk_check("E field", self.var_show_e)

        mk_labeled_scale("e_stride", self.var_e_stride, 1, 16, 1)
        mk_labeled_scale("e_scale", self.var_e_scale, 0.05, 1.0, 0.05)

        # percentile lo/hi
        fr2 = tk.Frame(panel)
        fr2.pack(side=tk.LEFT, padx=8)
        tk.Label(fr2, text="v[%]").pack(side=tk.TOP)

        s_lo = tk.Scale(fr2, variable=self.var_v_lo, from_=0.0, to=20.0, resolution=0.5, orient=tk.HORIZONTAL, length=140)
        s_hi = tk.Scale(fr2, variable=self.var_v_hi, from_=80.0, to=100.0, resolution=0.5, orient=tk.HORIZONTAL, length=140)

        tk.Label(fr2, text="lo").pack(side=tk.LEFT)
        s_lo.pack(side=tk.TOP)
        tk.Label(fr2, text="hi").pack(side=tk.LEFT)
        s_hi.pack(side=tk.TOP)

        # reset (简单：暂停 + 继续跑；demo 里通过脚本重建 sim 更合理)
        reset_btn = tk.Button(panel, text="Pause&ResetHint", command=self._reset_hint)
        reset_btn.pack(side=tk.RIGHT, padx=6)

    def _reset_hint(self) -> None:
        # MVP：不重建 sim（需要 sim factory 才能正确 reset）
        # 这里只是提供可用按钮，不做“错误的伪重置”。
        self.var_play.set(False)
        self._play_btn.config(text="Play")

    def _toggle_play(self):
        now = not self.var_play.get()
        self.var_play.set(now)
        self._play_btn.config(text="Pause" if now else "Play")

    def _sync_visibility(self) -> None:
        show_v = bool(self.var_show_v.get())
        show_p = bool(self.var_show_particles.get())
        show_e = bool(self.var_show_e.get())

        if self._im is not None:
            self._im.set_visible(show_v)

        if self._scat is not None:
            self._scat.set_visible(show_p)

        if self._quiv is not None:
            self._quiv.set_visible(show_e)

        self.canvas.draw_idle()

    def _create_quiver(self, Ex: np.ndarray, Ey: np.ndarray):
        xs = np.linspace(0.0, self.Lx, self.grid.nx, endpoint=False)
        ys = np.linspace(0.0, self.Ly, self.grid.ny, endpoint=False)
        X, Y = np.meshgrid(xs, ys, indexing="ij")

        stride = max(1, int(self.var_e_stride.get()))
        Ex_s = Ex[::stride, ::stride]
        Ey_s = Ey[::stride, ::stride]
        X_s = X[::stride, ::stride]
        Y_s = Y[::stride, ::stride]

        scale = 1.0 / max(float(self.var_e_scale.get()), 1e-12)

        quiv = self.ax.quiver(
            X_s,
            Y_s,
            Ex_s,
            Ey_s,
            angles="xy",
            scale_units="xy",
            scale=scale,
            width=0.003,
            color="tab:green",
        )
        return quiv

    def _update_quiver(self, Ex: np.ndarray, Ey: np) -> None:
        # MVP：当 stride/scale 或 show_e 状态变化时直接重建 quiver，确保维度匹配稳定
        if self._quiv is not None:
            self._quiv.remove()
            self._quiv = None

        if bool(self.var_show_e.get()):
            self._quiv = self._create_quiver(Ex, Ey)
            self._sync_visibility()

    def _tick(self):
        if not self._running:
            return

        if self.var_play.get():
            dt = float(self.var_dt.get())
            k = int(self.var_steps_per_frame.get())
            for _ in range(max(1, k)):
                self.sim.step(dt)

        snap = self.sim.get_state_snapshot()
        x = snap["x"]
        y = snap["y"]
        V = snap["V"]
        Ex = snap["Ex"]
        Ey = snap["Ey"]

        # V
        if self._im is not None and self.var_show_v.get():
            vmin, vmax = _safe_percentile_vmin_vmax(V, float(self.var_v_lo.get()), float(self.var_v_hi.get()))
            self._im.set_data(V.T)
            self._im.set_clim(vmin=vmin, vmax=vmax)

        # particles
        if self.var_show_particles.get():
            if self._scat is None:
                self._scat = self.ax.scatter(x, y, s=18, c="k", alpha=0.85)
            else:
                self._scat.set_offsets(np.stack([x, y], axis=1))
        else:
            if self._scat is not None:
                self._scat.set_visible(False)

        # E field
        if self.var_show_e.get():
            self._update_quiver(Ex, Ey)
        else:
            if self._quiv is not None:
                self._quiv.set_visible(False)

        # status
        Vmin = float(np.min(V))
        Vmax = float(np.max(V))
        Emax = float(max(np.max(np.abs(Ex)), np.max(np.abs(Ey))))
        self._status_text.set_text(
            f"V: [{Vmin:.3g}, {Vmax:.3g}]  max|E|: {Emax:.3g}\n"
            f"dt={self.var_dt.get():.4f} steps={self.var_steps_per_frame.get()}  "
            f"stride={self.var_e_stride.get()} scale={self.var_e_scale.get():.3g}"
        )

        self.canvas.draw_idle()
        self.master.after(int(self.cfg.interval_ms), self._tick)

    def close(self):
        self._running = False
        try:
            self.master.destroy()
        except Exception:
            pass


def run_tkinter_viewer(sim, *, config: TkViewerConfig | None = None):
    if config is None:
        config = TkViewerConfig()

    root = tk.Tk()
    root.title("LiziEngine2D - Tkinter Viewer")

    viewer = TkinterElectrostaticViewer2D(root, sim, config=config)

    def on_close():
        viewer.close()

    root.protocol("WM_DELETE_WINDOW", on_close)
    root.mainloop()
