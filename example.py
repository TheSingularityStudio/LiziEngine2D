import lizilib

if __name__ == '__main__':
    # 创建一个10x10的二维电场
    E = lizilib.empty_grid(10, 10)
    # 创建一个粒子
    lizi1 = lizilib.lizi(zuobiao=(5, 5), dianhe=1, sudu=(0, 0), jiasudu=(0, 0))
    