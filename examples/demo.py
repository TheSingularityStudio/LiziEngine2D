#!/usr/bin/env python3
"""
LiziEngine2D 演示 - 独立运行
只需 numpy + matplotlib
"""
import sys
import os

# Add project root to Python path for local package imports
project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, project_root)

from lizi_engine.core.app import run_demo

if __name__ == "__main__":
    print("=== LiziEngine2D 演示 ===")
    print("CPU计算 + 标记系统 + matplotlib渲染")
    print("按 Ctrl+C 停止\n")
    
    config_path = "lizi_engine/config.json"
    run_demo(config_path)
