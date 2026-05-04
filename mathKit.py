# 一些常用的数学工具函数
import math
import numpy as np

# 双线插值，输入网格对象及坐标，返回插值结果
def chazhi(grid, coord: list[float, float]):
    return grid.interp(coord[0], coord[1])