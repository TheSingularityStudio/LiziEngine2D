use ndarray::Array1;
use ndarray::Array2;

use crate::core::grid::Grid2D;
use crate::core::particles::ParticleState;

/// 将定义在网格节点上的电场矢量场 (Ex, Ey) 通过双线性插值采样到粒子位置
///
/// 返回: (fx, fy) — shape 均为 (N,) 的粒子受力分量
pub fn gather_field_to_particles_bilinear(
    grid: &Grid2D,
    particles: &ParticleState,
    ex: &Array2<f64>,
    ey: &Array2<f64>,
) -> (Array1<f64>, Array1<f64>) {
    let (nx, ny) = grid.shape();
    let n = particles.len();
    let lx = grid.lx();
    let ly = grid.ly();

    let mut fx = Array1::zeros(n);
    let mut fy = Array1::zeros(n);

    for p in 0..n {
        // 周期包裹
        let xw = ((particles.x[p] % lx) + lx) % lx;
        let yw = ((particles.y[p] % ly) + ly) % ly;
        let gx = xw / grid.dx;
        let gy = yw / grid.dy;

        let i0 = (gx.floor() as i64).rem_euclid(nx as i64) as usize;
        let j0 = (gy.floor() as i64).rem_euclid(ny as i64) as usize;
        let i1 = (i0 + 1) % nx;
        let j1 = (j0 + 1) % ny;

        let ftx = gx - gx.floor();
        let fty = gy - gy.floor();

        let wx0 = 1.0 - ftx;
        let wx1 = ftx;
        let wy0 = 1.0 - fty;
        let wy1 = fty;

        // 双线性插值
        fx[p] = ex[[i0, j0]] * wx0 * wy0
            + ex[[i1, j0]] * wx1 * wy0
            + ex[[i0, j1]] * wx0 * wy1
            + ex[[i1, j1]] * wx1 * wy1;

        fy[p] = ey[[i0, j0]] * wx0 * wy0
            + ey[[i1, j0]] * wx1 * wy0
            + ey[[i0, j1]] * wx0 * wy1
            + ey[[i1, j1]] * wx1 * wy1;
    }

    (fx, fy)
}