use crate::core::grid::Grid2D;
use crate::core::particles::ParticleState;
use crate::core::sim::ElectrostaticSim2D;
use crate::visual::minifb_renderer::MinifbRenderer;

/// Demo：单点电荷可视化
///
/// 在网格中央放置一个单位电荷，显示 V 热力图和粒子位置。
/// 使用 minifb 轻量窗口渲染。
pub fn run(nx: usize, ny: usize, dx: f64, dy: f64, charge_x: Option<f64>, charge_y: Option<f64>, eps: f64) {
    let grid = Grid2D::new(nx, ny, dx, dy);
    let lx = grid.lx();
    let ly = grid.ly();

    let cx = charge_x.unwrap_or(0.5 * lx);
    let cy = charge_y.unwrap_or(0.5 * ly);

    let mut particles = ParticleState::zeros(1, Some(0));
    particles.x[0] = cx;
    particles.y[0] = cy;

    let mut sim = ElectrostaticSim2D::new(grid, particles, eps);
    sim.compute_fields();

    let snapshot = sim.get_state_snapshot();

    // 使用 minifb 窗口显示
    let mut renderer = MinifbRenderer::new(
        "LiziEngine2D - Single Charge Demo",
        512,
        512,
    );

    println!("Single Charge Demo started. Press ESC to close.");

    let mut frame_count = 0usize;
    loop {
        if !renderer.render(&snapshot) {
            println!("Demo window closed at frame {}.", frame_count);
            return;
        }

        frame_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}