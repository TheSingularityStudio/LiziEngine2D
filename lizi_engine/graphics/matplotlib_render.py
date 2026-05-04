"""
Matplotlib渲染器 - quiver向量场 + scatter标记
非交互式，适合动画/保存
"""
import matplotlib.pyplot as plt
import numpy as np
from typing import Optional, Tuple

class MatplotlibRenderer:
    """Matplotlib向量场+标记渲染器"""
    
    def __init__(self, figsize: Tuple[float, float] = (12, 9)):
        self.figsize = figsize
        self.fig = None
        self.ax = None
        self.quiver = None
        self.scatter = None
        
    def init_plot(self, width: int, height: int, title: str = "LiziEngine "):
        """初始化matplotlib图"""
        plt.ion()  # 交互模式
        self.fig, self.ax = plt.subplots(figsize=self.figsize)
        self.ax.set_xlim(0, width)
        self.ax.set_ylim(0, height)
        self.ax.set_aspect('equal')
        self.ax.set_title(title)
        self.ax.invert_yaxis()  # matplotlib y向下
        plt.tight_layout()
        
    def render(self, grid: np.ndarray, markers: list, 
               vector_scale: float = 20.0, show_grid: bool = True,
               vector_color: Tuple[float, float, float] = (0.2, 0.6, 1.0),
               marker_color: Tuple[float, float, float] = (1.0, 0.2, 0.2)):
        """渲染场+标记"""
        if self.ax is None:
            return
            
        self.ax.clear()
        self.ax.set_xlim(0, grid.shape[1])
        self.ax.set_ylim(0, grid.shape[0])
        
        # 非零向量掩码
        mask = np.linalg.norm(grid, axis=2) > 0.01
        if np.any(mask):
            y, x = np.mgrid[0:grid.shape[0], 0:grid.shape[1]]
            u = grid[mask, 0] * vector_scale
            v = grid[mask, 1] * vector_scale  
            self.ax.quiver(x[mask], y[mask], u, v, 
                          color=vector_color, scale=1, width=0.003, alpha=0.8)
        
        # 标记
        if markers:
            xs = [m['x'] for m in markers]
            ys = [m['y'] for m in markers]
            sizes = [m['mag'] * 100 for m in markers]
            self.ax.scatter(xs, ys, c=marker_color, s=sizes, alpha=0.9, edgecolors='white', linewidth=0.5)
        
        if show_grid:
            self.ax.grid(True, alpha=0.3, color='gray')
            
        plt.pause(0.01)  # 更新显示
        
    def save_frame(self, path: str):
        """保存当前帧"""
        if self.fig:
            self.fig.savefig(path, dpi=100, bbox_inches='tight')
            
    def close(self):
        """关闭图"""
        if self.fig:
            plt.ioff()
            plt.close(self.fig)
