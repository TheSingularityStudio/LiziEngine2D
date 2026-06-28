#pragma once

#include <cstdint>
#include <string>
#include <vector>

struct FrameSnapshot {
    uint32_t nx = 0;
    uint32_t ny = 0;
    uint32_t n_particles = 0;

    // 粒子：x[N], y[N]
    std::vector<float> px;
    std::vector<float> py;

    // 电势：V[nx*ny]
    std::vector<float> V;

    void clear() {
        nx = 0;
        ny = 0;
        n_particles = 0;
        px.clear();
        py.clear();
        V.clear();
    }
};

class FrameFileIO {
public:
    // 约定二进制文件布局（little-endian）：
    // uint32 nx, uint32 ny, uint32 N
    // float32 x[N], float32 y[N]
    // float32 V[nx*ny]
    bool load_latest(const std::string& path, FrameSnapshot& out);
};
