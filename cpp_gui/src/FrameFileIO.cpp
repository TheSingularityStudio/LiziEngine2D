#include "FrameFileIO.h"

#include <fstream>
#include <iostream>
#include <vector>
#include <string>

static bool read_exact(std::ifstream&amp; ifs, void* dst, size_t bytes) {
    if (bytes == 0) return true;
    ifs.read(reinterpret_cast<char*>(dst), static_cast<std::streamsize>(bytes));
    return static_cast<size_t>(ifs.gcount()) == bytes;
}

bool FrameFileIO::load_latest(const std::string&amp; path, FrameSnapshot&amp; out) {
    std::ifstream ifs(path, std::ios::binary);
    if (!ifs.is_open()) return false;

    uint32_t nx = 0, ny = 0;
    uint32_t n_particles = 0;

    // Header: nx, ny, N (uint32 little-endian)
    if (!read_exact(ifs, &amp;nx, sizeof(uint32_t))) return false;
    if (!read_exact(ifs, &amp;ny, sizeof(uint32_t))) return false;
    if (!read_exact(ifs, &amp;n_particles, sizeof(uint32_t))) return false;

    out.nx = nx;
    out.ny = ny;
    out.n_particles = n_particles;

    out.px.resize(n_particles);
    out.py.resize(n_particles);
    out.V.resize(static_cast<size_t>(nx) * static_cast<size_t>(ny));

    // Particles: x[N], y[N] float32
    if (!read_exact(ifs, out.px.data(), sizeof(float) * static_cast<size_t>(n_particles))) return false;
    if (!read_exact(ifs, out.py.data(), sizeof(float) * static_cast<size_t>(n_particles))) return false;

    // V: nx*ny float32
    if (!read_exact(ifs, out.V.data(), sizeof(float) * static_cast<size_t>(nx) * static_cast<size_t>(ny))) return false;

    return true;
}
