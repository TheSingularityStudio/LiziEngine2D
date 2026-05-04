"""
CPU向量场计算模块 - 纯numpy实现，无外部依赖
"""
import numpy as np
from typing import Tuple, List

class CPUVectorCalculator:
    """CPU向量场计算器 - 纯numpy向量化实现"""
    
    def __init__(self, self_weight: float = 0.2, neighbor_weight: float = 0.2):
        self.self_weight = self_weight
        self.neighbor_weight = neighbor_weight

    def create_grid(self, width: int, height: int, default: Tuple[float, float] = (0.0, 0.0)) -> np.ndarray:
        """创建向量网格"""
        grid = np.zeros((height, width, 2), dtype=np.float32)
        if default != (0.0, 0.0):
            grid[..., 0] = default[0]
            grid[..., 1] = default[1]
        return grid

    def update_grid(self, grid: np.ndarray) -> np.ndarray:
        """高效更新整个网格 - 向量化相邻求和"""
        if grid is None:
            return grid
        
        h, w = grid.shape[:2]
        
        # pad处理边界
        padded = np.pad(grid, ((1,1), (1,1), (0,0)), mode='edge')
        
        # 4方向邻居
        up = padded[2:, 1:-1] * self.neighbor_weight
        down = padded[:-2, 1:-1] * self.neighbor_weight
        left = padded[1:-1, :-2] * self.neighbor_weight  
        right = padded[1:-1, 2:] * self.neighbor_weight
        
        result = up + down + left + right + grid * self.self_weight
        grid[:] = result
        return grid

    def fit_vector(self, grid: np.ndarray, x: float, y: float) -> Tuple[float, float]:
        """双线性插值拟合向量"""
        h, w = grid.shape[:2]
        x = np.clip(x, 0, w-1)
        y = np.clip(y, 0, h-1)
        
        x0, y0 = int(np.floor(x)), int(np.floor(y))
        x1, y1 = min(x0+1, w-1), min(y0+1, h-1)
        
        wx, wy = x - x0, y - y0
        w00, w01, w10, w11 = (1-wx)*(1-wy), wx*(1-wy), (1-wx)*wy, wx*wy
        
        v00 = grid[y0, x0]
        v01 = grid[y0, x1]
        v10 = grid[y1, x0]
        v11 = grid[y1, x1]
        
        vx = w00*v00[0] + w01*v01[0] + w10*v10[0] + w11*v11[0]
        vy = w00*v00[1] + w01*v01[1] + w10*v10[1] + w11*v11[1]
        
        return vx, vy

    def fit_vectors_batch(self, grid: np.ndarray, positions: List[Tuple[float, float]]) -> List[Tuple[float, float]]:
        """批量插值"""
        return [self.fit_vector(grid, x, y) for x, y in positions]

    def create_tiny_vectors(self, grid: np.ndarray, positions: List[Tuple[float, float, float]]) -> None:
        """批量创建微小向量影响 (markers -> field)"""
        for x, y, mag in positions:
            for dy in [-1, 0, 1]:
                for dx in [-1, 0, 1]:
                    if abs(dx) + abs(dy) == 1:  # 邻居
                        self.add_vector(grid, x+dx, y+dy, dx*mag, dy*mag)

    def add_vector(self, grid: np.ndarray, x: float, y: float, vx: float, vy: float) -> None:
        """双线性添加向量"""
        h, w = grid.shape[:2]
        x = np.clip(x, 0, w-1)
        y = np.clip(y, 0, h-1)
        
        x0, y0 = int(np.floor(x)), int(np.floor(y))
        x1, y1 = min(x0+1, w-1), min(y0+1, h-1)
        
        wx, wy = x - x0, y - y0
        w00, w01, w10, w11 = (1-wx)*(1-wy), wx*(1-wy), (1-wx)*wy, wx*wy
        
        grid[y0, x0] += [w00 * vx, w00 * vy]
        grid[y0, x1] += [w01 * vx, w01 * vy]
        grid[y1, x0] += [w10 * vx, w10 * vy]
        grid[y1, x1] += [w11 * vx, w11 * vy]
