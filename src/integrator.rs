use crate::grid::Grid2D;
use crate::particles::ParticleState;

/// 半隐式欧拉时间积分：
///
///   v_{n+1} = v_n + a_n * dt
///   x_{n+1} = x_n + v_{n+1} * dt
///
/// 由于 q=1、m=1，加速度 a = F
///
/// 返回：更新后的 particles（位置做周期包裹）
pub fn step_half_implicit_euler(
    grid: &Grid2D,
    particles: &mut ParticleState,
    dt: f64,
) {
    let lx = grid.lx();
    let ly = grid.ly();

    for p in 0..particles.len() {
        // v += a * dt (a = F 因为 q=1, m=1)
        particles.vx[p] += particles.fx[p] * dt;
        particles.vy[p] += particles.fy[p] * dt;

        // x += v * dt
        particles.x[p] += particles.vx[p] * dt;
        particles.y[p] += particles.vy[p] * dt;

        // 周期包裹
        particles.x[p] = ((particles.x[p] % lx) + lx) % lx;
        particles.y[p] = ((particles.y[p] % ly) + ly) % ly;
    }
}