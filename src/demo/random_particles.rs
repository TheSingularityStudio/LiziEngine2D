use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::Rng;

use crate::core::grid::Grid2D;
use crate::core::particles::ParticleState;
use crate::core::sim::ElectrostaticSim2D;
use crate::visual::{MinifbRenderer, VisualWindow};

/// Demo：随机粒子模拟动画
///
/// 200 个随机初始化的粒子在静电场中运动，
/// 使用 minifb 窗口实时显示 V 热力图和粒子位置动画。
pub fn run(nx: usize, ny: usize, dx: f64, dy: f64, n: usize, _steps: usize, dt: f64, eps: f64, seed: u64) {
    let grid = Grid2D::new(nx, ny, dx, dy);
    let lx = grid.lx();
    let ly = grid.ly();

    // 随机初始化粒子位置和速度
    let mut particles = ParticleState::zeros(n, Some(seed));
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(123));
    for i in 0..n {
        particles.x[i] = rng.gen::<f64>() * lx;
        particles.y[i] = rng.gen::<f64>() * ly;
        particles.vx[i] = (rng.gen::<f64>() - 0.5) * 0.02;
        particles.vy[i] = (rng.gen::<f64>() - 0.5) * 0.02;
    }

    let mut sim = ElectrostaticSim2D::new(grid, particles, eps);

    let mut renderer = MinifbRenderer::new(
        "LiziEngine2D - Random Particles Animation",
        512,
        512,
    );

    println!("Random Particles Demo started. Press ESC to close.");
    println!("Running continuously until the window is closed; dt={}", dt);

    let mut step_count = 0usize;
    loop {
        sim.step(dt);

        let snapshot = sim.get_state_snapshot();
        if !renderer.update(&snapshot) {
            println!("Window closed by user at step {}.", step_count);
            return;
        }

        step_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}