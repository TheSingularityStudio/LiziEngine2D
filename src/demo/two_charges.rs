use crate::core::grid::Grid2D;
use crate::core::particles::ParticleState;
use crate::core::sim::ElectrostaticSim2D;
use crate::visual::minifb_renderer::MinifbRenderer;
use crate::visual::window::VisualWindow;

/// Demo：双电荷叠加可视化（动态交互版）
///
/// 在网格上放置两个电荷，显示叠加 V 热力图和粒子位置。
/// 支持鼠标拖动粒子、Space 暂停/继续。
/// 使用 minifb 轻量窗口渲染。
pub fn run(nx: usize, ny: usize, dx: f64, dy: f64, eps: f64, opposite_sign: bool) {
    let grid = Grid2D::new(nx, ny, dx, dy);
    let lx = grid.lx();
    let ly = grid.ly();
    
    // 创建两个粒子的模拟器
    let mut particles = ParticleState::zeros(2, Some(0));
    particles.x[0] = 0.25 * lx;
    particles.y[0] = 0.5 * ly;
    particles.x[1] = 0.75 * lx;
    particles.y[1] = 0.5 * ly;
    
    // 注意：当前系统假设所有粒子带单位正电荷
    // opposite_sign 参数暂时不影响物理计算，只影响标题
    // 要真正支持异号电荷，需要扩展 ParticleState 添加电荷属性
    
    let mut sim = ElectrostaticSim2D::new(grid, particles, eps);
    
    let title = if opposite_sign {
        "LiziEngine2D - Two Charges (Opposite Sign) Demo"
    } else {
        "LiziEngine2D - Two Charges (Same Sign) Demo"
    };
    
    let mut renderer = MinifbRenderer::new(title, 512, 512);
    
    println!("Two Charges Demo started. Press ESC to close, Space to pause/resume.");
    println!("You can drag particles with the mouse.");
    
    // 时间步长
    let dt = 0.05;
    
    loop {
        // 只在未暂停时执行模拟步骤
        if !renderer.is_paused() {
            sim.step(dt);
        }
        
        let snapshot = sim.get_state_snapshot();
        if !renderer.update(&snapshot) {
            println!("Demo window closed.");
            return;
        }
        
        // 处理拖动事件：获取待处理的粒子位置更新并应用到模拟器
        let updates = renderer.get_pending_particle_updates();
        for (idx, x, y) in updates {
            sim.set_particle_position(idx, x, y);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
