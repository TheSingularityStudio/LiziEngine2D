use crate::core::grid::Grid2D;
use crate::core::particles::ParticleState;
use crate::core::sim::ElectrostaticSim2D;
use crate::core::interaction::InteractionState;
use crate::visual::minifb_renderer::MinifbRenderer;

/// Demo：双电荷叠加可视化（动态交互版）
///
/// 在网格上放置两个电荷，显示叠加 V 热力图和粒子位置。
/// 交互逻辑直接操作 sim.particles，无延迟。
/// 支持 ESC 退出、Space 暂停/继续、鼠标拖动粒子。
pub fn run(nx: usize, ny: usize, dx: f64, dy: f64, eps: f64, opposite_sign: bool) {
    let grid = Grid2D::new(nx, ny, dx, dy);
    let lx = grid.lx();
    let ly = grid.ly();
    
    // 创建两个粒子的模拟器
    let charges = if opposite_sign {
        vec![1.0, -1.0]
    } else {
        vec![1.0, 1.0]
    };
    let mut particles = ParticleState::with_charges(2, Some(0), &charges);
    particles.x[0] = 0.25 * lx;
    particles.y[0] = 0.5 * ly;
    particles.x[1] = 0.75 * lx;
    particles.y[1] = 0.5 * ly;
    
    let mut sim = ElectrostaticSim2D::new(grid, particles, eps);
    
    let title = if opposite_sign {
        "LiziEngine2D - Two Charges (Opposite Sign) Demo"
    } else {
        "LiziEngine2D - Two Charges (Same Sign) Demo"
    };
    
    let mut renderer = MinifbRenderer::new(title, 512, 512);
    let mut interaction = InteractionState::new();
    
    println!("Two Charges Demo started. Press ESC to close, Space to pause/resume.");
    println!("You can drag particles with the mouse.");
    if opposite_sign {
        println!("First particle: +1, Second particle: -1");
    }
    
    let dt = 0.05;
    
    loop {
        // 1. 处理鼠标交互（直接修改 sim.particles，即时生效）
        let _is_dragging = interaction.handle_mouse(
            renderer.window(),
            renderer.width(),
            renderer.height(),
            &mut sim,
        );

        // 2. 只在未暂停时执行模拟步骤
        if !renderer.is_paused() {
            sim.step(dt);
        }

        // 3. 获取快照并渲染
        let snapshot = sim.get_state_snapshot();
        if !renderer.render(&snapshot) {
            println!("Demo window closed.");
            return;
        }
        
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}