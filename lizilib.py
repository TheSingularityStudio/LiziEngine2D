import numpy as np

# 二维网格对象，用零向量填充
def empty_grid(nx: int, ny: int):
    return np.zeros((nx, ny, 2))

# 定义粒子对象
class lizi:
    def __init__(self, zuobiao: list[float, float], dianhe: int, sudu: list[float, float], jiasudu: list[float, float]):
        self.zuobiao = zuobiao #坐标
        self.dianhe = dianhe #电荷
        self.sudu = sudu #速度
        self.jiasudu = jiasudu #加速度
        
    def update(self, dt: float):
        # 更新速度和位置
        self.sudu += self.jiasudu * dt
        self.zuobiao += self.sudu * dt
