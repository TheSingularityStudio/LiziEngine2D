"""
标记系统 - 管理粒子标记，使用向量场驱动运动
纯Python + numpy，无外部GUI依赖
"""
from typing import List, Dict, Tuple
import numpy as np
from .cpu_vector import CPUVectorCalculator

class MarkerSystem:
    """标记系统 - 向量场驱动的粒子模拟"""
    
    def __init__(self, vector_calc: CPUVectorCalculator, gravity: float = 0.01, speed_factor: float = 0.95):
        self.vector_calc = vector_calc
        self.gravity = gravity
        self.speed_factor = speed_factor
        self.markers: List[Dict[str, float]] = []
    
    def add_marker(self, x: float, y: float, mag: float = 1.0) -> None:
        """添加标记"""
        self.markers.append({
            'x': float(x), 'y': float(y), 
            'vx': 0.0, 'vy': 0.0, 'mag': float(mag)
        })
    
    def clear(self) -> None:
        """清空标记"""
        self.markers.clear()
    
    def get_markers(self) -> List[Tuple[float, float, float]]:
        """获取标记位置和幅值 (x,y,mag)"""
        return [(m['x'], m['y'], m['mag']) for m in self.markers]
    
    def update(self, grid: np.ndarray, dt: float = 1.0) -> None:
        """更新所有标记：场驱动 + 物理"""
        if not self.markers or grid is None:
            return
        
        h, w = grid.shape[:2]
        
        # 批量获取场值
        positions = [(m['x'], m['y']) for m in self.markers]
        field_vectors = self.vector_calc.fit_vectors_batch(grid, positions)
        
        # 更新每个标记
        new_markers = []
        for i, marker in enumerate(self.markers):
            fx, fy = field_vectors[i]
            
            # 加速度 = 场 / mag
            marker['vx'] += fx / marker['mag']
            marker['vy'] += fy / marker['mag'] + self.gravity
            
            # 速度限制
            speed = (marker['vx']**2 + marker['vy']**2)**0.5
            if speed > 1.0:  # cell units/sec
                scale = 1.0 / speed
                marker['vx'] *= scale
                marker['vy'] *= scale
            
            # 位置更新 + 阻尼
            marker['x'] = np.clip(marker['x'] + marker['vx'] * dt, 0, w-1)
            marker['y'] = np.clip(marker['y'] + marker['vy'] * dt, 0, h-1)
            marker['vx'] *= self.speed_factor
            marker['vy'] *= self.speed_factor
            
            new_markers.append(marker)
        
        self.markers = new_markers
    
    def apply_to_field(self, grid: np.ndarray) -> None:
        """标记影响场：每个标记创建tiny vectors"""
        if not self.markers:
            return
        positions = self.get_markers()
        self.vector_calc.create_tiny_vectors(grid, positions)
