use crate::core::grid::Grid2D;
use crate::core::particles::ParticleState;
use crate::core::sim::ElectrostaticSim2D;
use crate::visual::{MinifbRenderer, VisualWindow};

/// Demo：双电荷叠加可视化
///
/// 在网格左半部和右半部各放置一个单位电荷，显示叠加 V 热力图和粒子位置。
/// 使用 minifb 轻量窗口渲染。
pub fn run(nx: usize, ny: usize, dx: f64, dy: f64, eps: f64, opposite_sign: bool) {
    let grid = Grid2D::new(nx, ny, dx, dy);
    let lx = grid.lx();
    let ly = grid.ly();

    // 使用两个正电荷，通过 scatter 叠加
    let mut particles_pos = ParticleState::zeros(1, Some(0));
    particles_pos.x[0] = 0.25 * lx;
    particles_pos.y[0] = 0.5 * ly;

    let mut particles_neg = ParticleState::zeros(1, Some(0));
    particles_neg.x[0] = 0.75 * lx;
    particles_neg.y[0] = 0.5 * ly;

    let mut sim1 = ElectrostaticSim2D::new(grid.clone(), particles_pos, eps);
    sim1.compute_fields();
    let snap1 = sim1.get_state_snapshot();

    let mut sim2 = ElectrostaticSim2D::new(grid.clone(), particles_neg, eps);
    sim2.compute_fields();
    let snap2 = sim2.get_state_snapshot();

    // 叠加两个快照 - 使用 owned Array2 避免临时值问题
    let neg_v = -&snap2.v;
    let neg_ex = -&snap2.ex;
    let neg_ey = -&snap2.ey;

    let combined_v = if opposite_sign {
        &snap1.v + &neg_v
    } else {
        &snap1.v + &snap2.v
    };
    let combined_ex = if opposite_sign {
        &snap1.ex + &neg_ex
    } else {
        &snap1.ex + &snap2.ex
    };
    let combined_ey = if opposite_sign {
        &snap1.ey + &neg_ey
    } else {
        &snap1.ey + &snap2.ey
    };

    let combined_x_data: Vec<f64> = snap1.x.iter().chain(snap2.x.iter()).copied().collect();
    let combined_y_data: Vec<f64> = snap1.y.iter().chain(snap2.y.iter()).copied().collect();
    let combined_vx_data: Vec<f64> = snap1.vx.iter().chain(snap2.vx.iter()).copied().collect();
    let combined_vy_data: Vec<f64> = snap1.vy.iter().chain(snap2.vy.iter()).copied().collect();

    let combined_snapshot = crate::core::sim::StateSnapshot {
        x: ndarray::Array1::from_vec(combined_x_data),
        y: ndarray::Array1::from_vec(combined_y_data),
        vx: ndarray::Array1::from_vec(combined_vx_data),
        vy: ndarray::Array1::from_vec(combined_vy_data),
        v: combined_v,
        ex: combined_ex,
        ey: combined_ey,
    };

    let title = if opposite_sign {
        "LiziEngine2D - Two Charges (Opposite Sign) Demo"
    } else {
        "LiziEngine2D - Two Charges (Same Sign) Demo"
    };

    let mut renderer = MinifbRenderer::new(title, 512, 512);

    println!("Two Charges Demo started. Press ESC to close.");

    let mut frame_count = 0usize;
    loop {
        if !renderer.update(&combined_snapshot) {
            println!("Demo window closed at frame {}.", frame_count);
            return;
        }

        frame_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}