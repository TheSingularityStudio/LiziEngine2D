use clap::{Parser, Subcommand};
use ndarray::Array1;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

use lizi_engine_2d::grid::Grid2D;
use lizi_engine_2d::particles::ParticleState;
use lizi_engine_2d::sim::ElectrostaticSim2D;
use lizi_engine_2d::interp::gather_field_to_particles_bilinear;
#[derive(Parser)]
#[command(name = "lizi2d", about = "2D Electrostatic PIC Simulator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 单电荷方向一致性验证
    ValidateSingleCharge {
        #[arg(long, default_value = "64")]
        nx: usize,
        #[arg(long, default_value = "64")]
        ny: usize,
        #[arg(long, default_value = "1.0")]
        dx: f64,
        #[arg(long, default_value = "1.0")]
        dy: f64,
        #[arg(long)]
        charge_x: Option<f64>,
        #[arg(long)]
        charge_y: Option<f64>,
        #[arg(long, default_value = "1e-12")]
        eps: f64,
    },
    /// 两电荷叠加验证
    ValidateTwoCharges {
        #[arg(long, default_value = "64")]
        nx: usize,
        #[arg(long, default_value = "64")]
        ny: usize,
        #[arg(long, default_value = "1.0")]
        dx: f64,
        #[arg(long, default_value = "1.0")]
        dy: f64,
        #[arg(long, default_value = "1e-12")]
        eps: f64,
    },
    /// 随机初始条件数值稳定性验证
    ValidateRandom {
        #[arg(long, default_value = "64")]
        nx: usize,
        #[arg(long, default_value = "64")]
        ny: usize,
        #[arg(long, default_value = "1.0")]
        dx: f64,
        #[arg(long, default_value = "1.0")]
        dy: f64,
        #[arg(long, default_value = "200")]
        n: usize,
        #[arg(long, default_value = "20")]
        steps: usize,
        #[arg(long, default_value = "0.05")]
        dt: f64,
        #[arg(long, default_value = "1e-12")]
        eps: f64,
        #[arg(long, default_value = "0")]
        seed: u64,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::ValidateSingleCharge {
            nx,
            ny,
            dx,
            dy,
            charge_x,
            charge_y,
            eps,
        } => validate_single_charge(nx, ny, dx, dy, charge_x, charge_y, eps),
        Command::ValidateTwoCharges {
            nx,
            ny,
            dx,
            dy,
            eps,
        } => validate_two_charges(nx, ny, dx, dy, eps),
        Command::ValidateRandom {
            nx,
            ny,
            dx,
            dy,
            n,
            steps,
            dt,
            eps,
            seed,
        } => validate_random(nx, ny, dx, dy, n, steps, dt, eps, seed),
    }
}

/// ==================== 验证脚本 1：单电荷方向一致性 ====================

fn validate_single_charge(
    nx: usize,
    ny: usize,
    dx: f64,
    dy: f64,
    charge_x: Option<f64>,
    charge_y: Option<f64>,
    eps: f64,
) {
    let grid = Grid2D::new(nx, ny, dx, dy);

    let mut particles = ParticleState::zeros(1, Some(0));
    if let Some(cx) = charge_x {
        particles.x[0] = cx;
    } else {
        particles.x[0] = 0.5 * grid.lx();
    }
    if let Some(cy) = charge_y {
        particles.y[0] = cy;
    } else {
        particles.y[0] = 0.5 * grid.ly();
    }

    let mut sim = ElectrostaticSim2D::new(grid.clone(), particles, eps);
    sim.compute_fields();

    let ex = sim.ex.as_ref().unwrap();
    let ey = sim.ey.as_ref().unwrap();

    let cx = sim.particles.x[0] / grid.dx;
    let cy = sim.particles.y[0] / grid.dy;

    let sample_r = [5, 8, 12, 16];
    let sample_angles: Vec<f64> = (0..16)
        .map(|i| 2.0 * std::f64::consts::PI * i as f64 / 16.0)
        .collect();

    let lx = grid.nx as f64;
    let ly = grid.ny as f64;

    let mut errors: Vec<f64> = Vec::new();

    for &r in &sample_r {
        for &th in &sample_angles {
            let gx = cx + r as f64 * th.cos();
            let gy = cy + r as f64 * th.sin();

            // 周期包裹
            let gxw = ((gx % lx) + lx) % lx;
            let gyw = ((gy % ly) + ly) % ly;

            let qx = gxw * grid.dx;
            let qy = gyw * grid.dy;

            let mut qstate = ParticleState::zeros(1, Some(0));
            qstate.x[0] = qx;
            qstate.y[0] = qy;

            let (fx, fy) = gather_field_to_particles_bilinear(&grid, &qstate, ex, ey);
            let exq = fx[0];
            let eyq = fy[0];

            // 计算周期坐标系下的径向方向
            let dxr = periodic_delta(gxw, cx, lx);
            let dyr = periodic_delta(gyw, cy, ly);
            let nr = (dxr * dxr + dyr * dyr).sqrt();
            if nr < 1e-9 {
                continue;
            }
            let rx = dxr / nr;
            let ry = dyr / nr;

            let en = (exq * exq + eyq * eyq).sqrt();
            if en < 1e-12 {
                continue;
            }

            let cosang = (exq * rx + eyq * ry) / en;
            errors.push(1.0 - cosang);
        }
    }

    let err = if errors.is_empty() {
        0.0
    } else {
        errors.iter().sum::<f64>() / errors.len() as f64
    };

    println!("[validate_single_charge] 平均方向误差(1-cos)= {:.6e}", err);

    if err > 2e-1 {
        std::process::exit(2);
    }
    println!("OK: 单点电荷方向一致性验证通过");
}

fn periodic_delta(a: f64, b: f64, period: f64) -> f64 {
    let d = a - b;
    ((d + 0.5 * period) % period + period) % period - 0.5 * period
}

/// ==================== 验证脚本 2：两电荷叠加 ====================

fn compute_e_for_charges(
    grid: &Grid2D,
    charges: &[(f64, f64, f64)],
    eps: f64,
) -> (ndarray::Array2<f64>, ndarray::Array2<f64>) {
    let pos: Vec<(f64, f64)> = charges
        .iter()
        .filter(|(_, _, q)| *q > 0.0)
        .map(|(x, y, _)| (*x, *y))
        .collect();
    let neg: Vec<(f64, f64)> = charges
        .iter()
        .filter(|(_, _, q)| *q < 0.0)
        .map(|(x, y, _)| (*x, *y))
        .collect();

    fn run_set(grid: &Grid2D, points: &[(f64, f64)], eps: f64) -> (ndarray::Array2<f64>, ndarray::Array2<f64>) {
        if points.is_empty() {
            return (
                ndarray::Array2::zeros(grid.shape()),
                ndarray::Array2::zeros(grid.shape()),
            );
        }
        let n = points.len();
        let mut p = ParticleState::zeros(n, Some(0));
        for (j, (x, y)) in points.iter().enumerate() {
            p.x[j] = *x;
            p.y[j] = *y;
        }
        let mut sim = ElectrostaticSim2D::new(grid.clone(), p, eps);
        sim.compute_fields();
        (sim.ex.unwrap(), sim.ey.unwrap())
    }

    let (ex_pos, ey_pos) = run_set(grid, &pos, eps);
    let (ex_neg, ey_neg) = run_set(grid, &neg, eps);

    (ex_pos - ex_neg, ey_pos - ey_neg)
}

fn validate_two_charges(nx: usize, ny: usize, dx: f64, dy: f64, eps: f64) {
    let grid = Grid2D::new(nx, ny, dx, dy);

    let x1 = 0.25 * grid.lx();
    let y1 = 0.5 * grid.ly();
    let x2 = 0.75 * grid.lx();
    let y2 = 0.5 * grid.ly();

    let (ex_total, ey_total) = compute_e_for_charges(
        &grid,
        &[(x1, y1, 1.0), (x2, y2, 1.0)],
        eps,
    );
    let (ex1, ey1) = compute_e_for_charges(&grid, &[(x1, y1, 1.0)], eps);
    let (ex2, ey2) = compute_e_for_charges(&grid, &[(x2, y2, 1.0)], eps);

    let nq = 200;
    let mut rng = StdRng::seed_from_u64(0);
    let mut qstate = ParticleState::zeros(nq, Some(0));
    for i in 0..nq {
        qstate.x[i] = rng.gen::<f64>() * grid.lx();
        qstate.y[i] = rng.gen::<f64>() * grid.ly();
    }

    let (exq_total, eyq_total) =
        gather_field_to_particles_bilinear(&grid, &qstate, &ex_total, &ey_total);
    let (exq_1, eyq_1) =
        gather_field_to_particles_bilinear(&grid, &qstate, &ex1, &ey1);
    let (exq_2, eyq_2) =
        gather_field_to_particles_bilinear(&grid, &qstate, &ex2, &ey2);

    let ex_pred = &exq_1 + &exq_2;
    let ey_pred = &eyq_1 + &eyq_2;

    let denom =
        l2_norm(&exq_total) + l2_norm(&eyq_total) + 1e-15;
    let num =
        l2_norm(&(&ex_pred - &exq_total)) + l2_norm(&(&ey_pred - &eyq_total));
    let rel = num / denom;

    println!("[validate_two_charges] 相对L2误差 relative_L2_error={:.6e}", rel);
    if rel > 5e-2 {
        std::process::exit(2);
    }
    println!("OK: 两电荷叠加验证通过");
}

fn l2_norm(a: &Array1<f64>) -> f64 {
    a.iter().map(|v| v * v).sum::<f64>().sqrt()
}

/// ==================== 验证脚本 3：随机初始条件数值稳定性 ====================

fn validate_random(nx: usize, ny: usize, dx: f64, dy: f64, n: usize, steps: usize, dt: f64, eps: f64, seed: u64) {
    let grid = Grid2D::new(nx, ny, dx, dy);

    let mut particles = ParticleState::zeros(n, Some(seed));
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(123));
    for i in 0..n {
        particles.vx[i] = (rng.gen::<f64>() - 0.5) * 0.02;
        particles.vy[i] = (rng.gen::<f64>() - 0.5) * 0.02;
    }

    let mut sim = ElectrostaticSim2D::new(grid, particles, eps);

    let mut max_speed = 0.0f64;
    for _ in 0..steps {
        sim.step(dt);
        let speed_max = sim
            .particles
            .vx
            .iter()
            .zip(sim.particles.vy.iter())
            .map(|(vx, vy)| (vx * vx + vy * vy).sqrt())
            .fold(0.0f64, f64::max);
        if speed_max > max_speed {
            max_speed = speed_max;
        }
    }

    println!("[validate_random] 多步最大速度 = {:.6e}", max_speed);

    if max_speed > 50.0 {
        std::process::exit(2);
    }
    println!("OK: 随机初始条件下的数值稳定性验证通过");
}