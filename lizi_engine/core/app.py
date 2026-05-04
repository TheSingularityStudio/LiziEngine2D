"""
LiziEngine  主应用 - 计算+标记+matplotlib
独立运行，无原项目依赖
"""
import os
import json
import numpy as np
from typing import Optional
import matplotlib.pyplot as plt

from ..compute.cpu_vector import CPUVectorCalculator
from ..markers import MarkerSystem
from ..graphics.matplotlib_render import MatplotlibRenderer

class LiziApp:
    """LiziEngine - CPU计算 + matplotlib渲染"""
    
    def __init__(self, config_path: str = None):
        # 加载配置
        self.config = self._load_config(config_path)
        
        # 计算器
        self.vector_calc = CPUVectorCalculator(
            self.config['vector_self_weight'],
            self.config['vector_neighbor_weight']
        )
        
        # 标记系统
        self.markers = MarkerSystem(self.vector_calc, 
                                  self.config['gravity'], 
                                  self.config['speed_factor'])
        
        # 渲染器
        self.renderer = MatplotlibRenderer()
        
        # 网格
        self.grid = None
        self._init_grid()
        
        print(f"LiziEngine  初始化完成: {self.config['grid_width']}x{self.config['grid_height']}")
    
    def _load_config(self, path: str) -> dict:
        """加载配置"""
        default_config = {
            'grid_width': 320, 'grid_height': 240, 'cell_size': 2.0,
            'vector_self_weight': 0.2, 'vector_neighbor_weight': 0.2,
            'vector_scale': 20.0, 'show_grid': True,
            'vector_color': [0.2, 0.6, 1.0], 'marker_color': [1.0, 0.2, 0.2],
            'gravity': 0.01, 'speed_factor': 0.95, 'compute_iterations': 1
        }
        
        if path and os.path.exists(path):
            with open(path, 'r', encoding='utf-8') as f:
                user_config = json.load(f)
                default_config.update(user_config)
        
        return default_config
    
    def _init_grid(self):
        """初始化网格"""
        self.grid = self.vector_calc.create_grid(
            self.config['grid_width'], 
            self.config['grid_height']
        )
        self.renderer.init_plot(
            self.config['grid_width'], 
            self.config['grid_height'],
            "LiziEngine  - CPU + Matplotlib"
        )
    
    def add_markers_random(self, count: int = 20):
        """随机添加标记"""
        import random
        h, w = self.grid.shape[:2]
        for _ in range(count):
            self.markers.add_marker(
                random.uniform(50, w-50),
                random.uniform(50, h-50),
                mag=random.uniform(0.5, 2.0)
            )
        print(f"添加 {count} 个随机标记")
    
    def step(self):
        """单步模拟：场更新 -> 标记影响场 -> 标记更新 -> 渲染"""
        # 场扩散
        for _ in range(self.config['compute_iterations']):
            self.vector_calc.update_grid(self.grid)
        
        # 标记影响场
        self.markers.apply_to_field(self.grid)
        
        # 标记跟随场
        self.markers.update(self.grid)
        
        # 渲染
        self.renderer.render(
            self.grid, self.markers.markers,
            vector_scale=self.config['vector_scale'],
            show_grid=self.config['show_grid'],
            vector_color=tuple(self.config['vector_color']),
            marker_color=tuple(self.config['marker_color'])
        )
    
    def run(self, steps: int = 1000, delay: float = 0.05):
        """运行模拟"""
        print(f"开始运行 {steps} 步 (按Ctrl+C停止)...")
        try:
            for i in range(steps):
                self.step()
                if i % 20 == 0:
                    print(f"Step {i}/{steps}, markers: {len(self.markers.markers)}")
                plt.pause(delay)
        except KeyboardInterrupt:
            print("\n模拟停止")
        finally:
            self.close()
    
    def close(self):
        """清理"""
        self.renderer.close()
        plt.ioff()

# 便捷运行
def run_demo(config_path: str = "lizi_engine/config.json"):
    app = LiziApp(config_path)
    app.add_markers_random(30)
    app.run(steps=2000, delay=0.03)
