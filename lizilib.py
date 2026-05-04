import numpy as np
import mathKit as mk

# 二维网格对象，用零向量填充
def empty_grid(nx: int, ny: int):
    return np.zeros((nx, ny, 2))

# 定义粒子对象
class lizi:
    def __init__(self, coord: list[float, float], q: int, v: list[float, float] = (0,0), a: list[float, float]=(0,0)):
        self.coord = coord #坐标
        self.q = q #电荷
        self.v = v #速度
        self.a = a #加速度
        
    def update(self, dt: float = 1.0):
        # 更新速度和位置
        self.v += self.a * dt
        self.coord += self.v * dt

    def force(self, E: np.ndarray):
        # 计算电场力
        fx, fy = self.q * mk.chazhi(E, self.coord)
        return list[fx, fy]

    def chendian(self, E: np.ndarray):
        # 沉淀电荷，记录电荷量和位置