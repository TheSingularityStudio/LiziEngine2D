import lizilib

if __name__ == '__main__':
    # 创建一个10x10的二维电场
    E = lizilib.empty_grid(10, 10)
    # 创建一个粒子
    lizi1 = lizilib.lizi(coord=(5, 5), q=1, v=(0, 0), a=(0, 0))
    