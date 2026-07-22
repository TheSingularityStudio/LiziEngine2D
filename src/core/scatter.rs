use ndarray::Array2;

use crate::core::grid::Grid2D;
use crate::core::particles::ParticleState;

/// 将单位电荷粒子双线性散射到网格节点上，生成离散电荷密度 rho
///
/// 返回: rho, shape = (nx, ny)
pub fn scatter_unit_charges_to_grid(grid: &Grid2D, particles: &ParticleState) -> Array2<f64> {
    let (nx, ny) = grid.shape();
    let mut rho = Array2::zeros((nx, ny));
    let lx = grid.lx();
    let ly = grid.ly();

    for p in 0..particles.len() {
        let xw = ((particles.x[p] % lx) + lx) % lx;
        let yw = ((particles.y[p] % ly) + ly) % ly;
        let gx = xw / grid.dx;
        let gy = yw / grid.dy;

        let i0 = (gx.floor() as i64).rem_euclid(nx as i64) as usize;
        let j0 = (gy.floor() as i64).rem_euclid(ny as i64) as usize;
        let i1 = (i0 + 1) % nx;
        let j1 = (j0 + 1) % ny;

        let fx = gx - gx.floor();
        let fy = gy - gy.floor();

        let wx0 = 1.0 - fx;
        let wx1 = fx;
        let wy0 = 1.0 - fy;
        let wy1 = fy;

        rho[[i0, j0]] += wx0 * wy0;
        rho[[i1, j0]] += wx1 * wy0;
        rho[[i0, j1]] += wx0 * wy1;
        rho[[i1, j1]] += wx1 * wy1;
    }

    rho
}