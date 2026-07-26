use std::fmt;
use ndarray::{Array2, Axis};
use ndrustfft::{ndfft, ndifft, FftHandler};
use num_complex::Complex64;
use num_traits::Zero;

/// Poisson 求解器类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PoissonSolverType {
    /// FFT 周期求解器（默认，最高效）
    FFTPeriodic,
    /// Jacobi 迭代求解器（支持 Dirichlet 边界）
    Jacobi,
    /// SOR 迭代求解器（支持 Dirichlet 边界，收敛更快）
    SOR,
}

impl PoissonSolverType {
    pub fn all() -> [PoissonSolverType; 3] {
        [
            PoissonSolverType::FFTPeriodic,
            PoissonSolverType::Jacobi,
            PoissonSolverType::SOR,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PoissonSolverType::FFTPeriodic => "FFT 周期",
            PoissonSolverType::Jacobi => "Jacobi 迭代",
            PoissonSolverType::SOR => "SOR 迭代",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PoissonSolverType::FFTPeriodic => "傅里叶变换求解，支持周期边界",
            PoissonSolverType::Jacobi => "迭代求解，支持 Dirichlet/Neumann 边界",
            PoissonSolverType::SOR => "超松弛迭代，收敛比 Jacobi 更快",
        }
    }
}

/// Poisson 边界条件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PoissonBoundaryType {
    /// Dirichlet 边界：V=0（导体边界）
    Dirichlet,
    /// Neumann 边界：dV/dn=0（开路，电场垂直边界为零）
    Neumann,
}

impl PoissonBoundaryType {
    pub fn all() -> [PoissonBoundaryType; 2] {
        [PoissonBoundaryType::Dirichlet, PoissonBoundaryType::Neumann]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PoissonBoundaryType::Dirichlet => "Dirichlet (V=0)",
            PoissonBoundaryType::Neumann => "Neumann (dE=0)",
        }
    }
}

/// 统一的 Poisson 求解器枚举
#[derive(Debug)]
pub enum PoissonSolverEnum {
    FFTPeriodic(FftPoissonSolver),
    Jacobi(JacobiSolver),
    SOR(SorSolver),
}

impl PoissonSolverEnum {
    pub fn new(solver_type: PoissonSolverType, nx: usize, ny: usize, dx: f64, dy: f64, eps: f64) -> Self {
        match solver_type {
            PoissonSolverType::FFTPeriodic => {
                PoissonSolverEnum::FFTPeriodic(FftPoissonSolver::new(nx, ny, dx, dy, eps))
            }
            PoissonSolverType::Jacobi => {
                PoissonSolverEnum::Jacobi(JacobiSolver::new(nx, ny, dx, dy, eps))
            }
            PoissonSolverType::SOR => {
                PoissonSolverEnum::SOR(SorSolver::new(nx, ny, dx, dy, eps))
            }
        }
    }

    pub fn solve(&mut self, rho: &Array2<f64>, eps: f64, boundary: PoissonBoundaryType) -> Array2<f64> {
        match self {
            PoissonSolverEnum::FFTPeriodic(solver) => solver.solve(rho, eps),
            PoissonSolverEnum::Jacobi(solver) => solver.solve(rho, eps, boundary),
            PoissonSolverEnum::SOR(solver) => solver.solve(rho, eps, boundary),
        }
    }

    pub fn shape(&self) -> (usize, usize) {
        match self {
            PoissonSolverEnum::FFTPeriodic(s) => s.shape(),
            PoissonSolverEnum::Jacobi(s) => s.shape(),
            PoissonSolverEnum::SOR(s) => s.shape(),
        }
    }
}

// ============================================================
// 1. FFT 周期求解器（原有算法）
// ============================================================

/// 使用 FFT 的 Poisson 求解器（带缓存，避免每帧重新分配 FFT Handler）
///
/// 在周期域中求解离散 Poisson 方程: ∇²V = -rho
/// 在傅里叶空间: V_hat(k) = rho_hat(k) / (kx² + ky²)
pub struct FftPoissonSolver {
    nx: usize,
    ny: usize,
    handler_0: FftHandler<f64>,
    handler_1: FftHandler<f64>,
    k2: Array2<f64>,
}

impl FftPoissonSolver {
    pub fn new(nx: usize, ny: usize, dx: f64, dy: f64, _eps: f64) -> Self {
        let handler_0 = FftHandler::new(nx);
        let handler_1 = FftHandler::new(ny);

        let mut k2 = Array2::<f64>::zeros((nx, ny));
        for i in 0..nx {
            let freq_i = if i <= nx / 2 {
                i as f64
            } else {
                i as f64 - nx as f64
            };
            let kx = 2.0 * std::f64::consts::PI * freq_i / (nx as f64 * dx);
            for j in 0..ny {
                let freq_j = if j <= ny / 2 {
                    j as f64
                } else {
                    j as f64 - ny as f64
                };
                let ky = 2.0 * std::f64::consts::PI * freq_j / (ny as f64 * dy);
                k2[[i, j]] = kx * kx + ky * ky;
            }
        }

        Self { nx, ny, handler_0, handler_1, k2 }
    }

    /// 求解 Poisson 方程（周期边界）
    pub fn solve(&mut self, rho: &Array2<f64>, eps: f64) -> Array2<f64> {
        let mut rho_hat: Array2<Complex64> = rho.mapv(|v| Complex64::new(v, 0.0));
        let mut tmp = Array2::zeros((self.nx, self.ny));

        ndfft(&rho_hat, &mut tmp, &mut self.handler_0, 0);
        ndfft(&tmp, &mut rho_hat, &mut self.handler_1, 1);

        for i in 0..self.nx {
            for j in 0..self.ny {
                if self.k2[[i, j]] > eps {
                    rho_hat[[i, j]] = rho_hat[[i, j]] / self.k2[[i, j]];
                } else {
                    rho_hat[[i, j]] = Complex64::zero();
                }
            }
        }

        let mut tmp2 = Array2::zeros((self.nx, self.ny));
        let mut v_complex: Array2<Complex64> = Array2::zeros((self.nx, self.ny));

        ndifft(&rho_hat, &mut tmp2, &mut self.handler_1, 1);
        ndifft(&tmp2, &mut v_complex, &mut self.handler_0, 0);

        v_complex.mapv(|c| c.re)
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.nx, self.ny)
    }
}

impl fmt::Debug for FftPoissonSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FftPoissonSolver")
            .field("nx", &self.nx)
            .field("ny", &self.ny)
            .finish()
    }
}

// ============================================================
// 2. Jacobi 迭代求解器（支持非周期边界）
// ============================================================

/// Jacobi 迭代 Poisson 求解器
///
/// 迭代格式: V_new[i,j] = (V[i+1,j] + V[i-1,j] + V[i,j+1] + V[i,j-1] + rho[i,j] * h²) / 4
/// 支持 Dirichlet (V=0) 和 Neumann (dV/dn=0) 边界条件
pub struct JacobiSolver {
    nx: usize,
    ny: usize,
    dx: f64,
    dy: f64,
    max_iters: usize,
    tolerance: f64,
}

impl JacobiSolver {
    pub fn new(nx: usize, ny: usize, dx: f64, dy: f64, _eps: f64) -> Self {
        // 根据网格大小自动估算迭代次数
        let grid_size = (nx * ny) as f64;
        let max_iters = (grid_size.sqrt() * 100.0) as usize;
        Self {
            nx, ny, dx, dy,
            max_iters: max_iters.max(1000),
            tolerance: 1e-6,
        }
    }

    /// 设置最大迭代次数
    pub fn with_max_iters(mut self, iters: usize) -> Self {
        self.max_iters = iters;
        self
    }

    /// 设置收敛容差
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }

    /// 求解 Poisson 方程（支持 Dirichlet/Neumann 边界）
    pub fn solve(&mut self, rho: &Array2<f64>, _eps: f64, boundary: PoissonBoundaryType) -> Array2<f64> {
        let mut v = Array2::zeros((self.nx, self.ny));
        let mut v_new = Array2::zeros((self.nx, self.ny));
        let _h2_inv = 2.0 / (self.dx * self.dx + self.dy * self.dy);
        let dx2 = self.dx * self.dx;
        let dy2 = self.dy * self.dy;
        let coeff = 0.5 / (dx2 + dy2) * dx2 * dy2; // dx²dy²/(2(dx²+dy²))

        for iteration in 0..self.max_iters {
            // 内部点 Jacobi 迭代
            for i in 1..self.nx - 1 {
                for j in 1..self.ny - 1 {
                    v_new[[i, j]] = coeff * (
                        (v[[i+1, j]] + v[[i-1, j]]) / dx2 +
                        (v[[i, j+1]] + v[[i, j-1]]) / dy2 +
                        rho[[i, j]]
                    );
                }
            }

            // 边界条件处理
            match boundary {
                PoissonBoundaryType::Dirichlet => {
                    // 四边 V=0（已在初始化时设置，此处确保不变）
                    // 边界保持 0，无需额外操作
                }
                PoissonBoundaryType::Neumann => {
                    // dV/dn=0: 边界节点等于相邻内部节点
                    for i in 1..self.nx - 1 {
                        v_new[[i, 0]] = v_new[[i, 1]];          // 下边界
                        v_new[[i, self.ny - 1]] = v_new[[i, self.ny - 2]]; // 上边界
                    }
                    for j in 1..self.ny - 1 {
                        v_new[[0, j]] = v_new[[1, j]];          // 左边界
                        v_new[[self.nx - 1, j]] = v_new[[self.nx - 2, j]]; // 右边界
                    }
                    // 四个角点
                    v_new[[0, 0]] = (v_new[[1, 0]] + v_new[[0, 1]]) / 2.0;
                    v_new[[self.nx - 1, 0]] = (v_new[[self.nx - 2, 0]] + v_new[[self.nx - 1, 1]]) / 2.0;
                    v_new[[0, self.ny - 1]] = (v_new[[1, self.ny - 1]] + v_new[[0, self.ny - 2]]) / 2.0;
                    v_new[[self.nx - 1, self.ny - 1]] = (v_new[[self.nx - 2, self.ny - 1]] + v_new[[self.nx - 1, self.ny - 2]]) / 2.0;
                }
            }

            // 检查收敛
            let mut max_diff: f64 = 0.0;
            for i in 0..self.nx {
                for j in 0..self.ny {
                    let val_i: f64 = *v_new.get([i, j]).unwrap_or(&0.0);
                    let val_j: f64 = *v.get([i, j]).unwrap_or(&0.0);
                    let diff = f64::abs(val_i - val_j);
                    if diff > max_diff {
                        max_diff = diff;
                    }
                }
            }

            v.assign(&v_new);

            if max_diff < self.tolerance {
                // 可以打印调试信息，但为了性能静默退出
                break;
            }

            // 安全：防止无限循环
            if iteration == self.max_iters - 1 {
                eprintln!("Jacobi solver reached max iterations ({}) with residual {}", self.max_iters, max_diff);
            }
        }

        v
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.nx, self.ny)
    }
}

impl fmt::Debug for JacobiSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JacobiSolver")
            .field("nx", &self.nx)
            .field("ny", &self.ny)
            .field("max_iters", &self.max_iters)
            .field("tolerance", &self.tolerance)
            .finish()
    }
}

// ============================================================
// 3. SOR 迭代求解器（超松弛，收敛更快）
// ============================================================

/// SOR (Successive Over-Relaxation) 迭代 Poisson 求解器
///
/// 迭代格式: V_new[i,j] = (1-ω)*V[i,j] + ω/4*(V[i+1,j] + V_new[i-1,j] + V[i,j+1] + V_new[i,j-1] + rho[i,j]*h²)
/// ω 为松弛因子，1<ω<2 为超松弛
/// 支持 Dirichlet 和 Neumann 边界条件
pub struct SorSolver {
    nx: usize,
    ny: usize,
    dx: f64,
    dy: f64,
    omega: f64,
    max_iters: usize,
    tolerance: f64,
}

impl SorSolver {
    pub fn new(nx: usize, ny: usize, dx: f64, dy: f64, _eps: f64) -> Self {
        let grid_size = (nx * ny) as f64;
        let max_iters = (grid_size.sqrt() * 50.0) as usize; // SOR 收敛更快，迭代次数减半
        // 最佳松弛因子的近似：ω ≈ 2/(1+sin(π/N))
        let n = nx.max(ny) as f64;
        let omega = 2.0 / (1.0 + (std::f64::consts::PI / n).sin());
        Self {
            nx, ny, dx, dy,
            omega: omega.clamp(1.0, 1.99),
            max_iters: max_iters.max(500),
            tolerance: 1e-6,
        }
    }

    /// 设置松弛因子
    pub fn with_omega(mut self, omega: f64) -> Self {
        self.omega = omega.clamp(1.0, 1.99);
        self
    }

    /// 设置最大迭代次数
    pub fn with_max_iters(mut self, iters: usize) -> Self {
        self.max_iters = iters;
        self
    }

    /// 设置收敛容差
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }

    /// 求解 Poisson 方程
    pub fn solve(&mut self, rho: &Array2<f64>, _eps: f64, boundary: PoissonBoundaryType) -> Array2<f64> {
        let mut v = Array2::zeros((self.nx, self.ny));
        let omega = self.omega;
        let dx2 = self.dx * self.dx;
        let dy2 = self.dy * self.dy;
        let denom = 2.0 * (dx2 + dy2);
        let coeff = dx2 * dy2 / denom; // dx²dy²/(2(dx²+dy²))

        for iteration in 0..self.max_iters {
            let mut max_diff = 0.0f64;

            // SOR 迭代（直接就地更新）
            for i in 1..self.nx - 1 {
                for j in 1..self.ny - 1 {
                    let v_old = v[[i, j]];
                    let v_new = coeff * (
                        (v[[i+1, j]] + v[[i-1, j]]) / dx2 +
                        (v[[i, j+1]] + v[[i, j-1]]) / dy2 +
                        rho[[i, j]]
                    );
                    v[[i, j]] = (1.0 - omega) * v_old + omega * v_new;
                    let diff = f64::abs(v[[i, j]] - v_old);
                    if diff > max_diff {
                        max_diff = diff;
                    }
                }
            }

            // 边界条件处理
            match boundary {
                PoissonBoundaryType::Dirichlet => {
                    // V=0 边界，不需要操作
                }
                PoissonBoundaryType::Neumann => {
                    for i in 1..self.nx - 1 {
                        v[[i, 0]] = v[[i, 1]];
                        v[[i, self.ny - 1]] = v[[i, self.ny - 2]];
                    }
                    for j in 1..self.ny - 1 {
                        v[[0, j]] = v[[1, j]];
                        v[[self.nx - 1, j]] = v[[self.nx - 2, j]];
                    }
                    v[[0, 0]] = (v[[1, 0]] + v[[0, 1]]) / 2.0;
                    v[[self.nx - 1, 0]] = (v[[self.nx - 2, 0]] + v[[self.nx - 1, 1]]) / 2.0;
                    v[[0, self.ny - 1]] = (v[[1, self.ny - 1]] + v[[0, self.ny - 2]]) / 2.0;
                    v[[self.nx - 1, self.ny - 1]] = (v[[self.nx - 2, self.ny - 1]] + v[[self.nx - 1, self.ny - 2]]) / 2.0;
                }
            }

            if max_diff < self.tolerance {
                break;
            }

            if iteration == self.max_iters - 1 {
                eprintln!("SOR solver reached max iterations ({}) with residual {}", self.max_iters, max_diff);
            }
        }

        v
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.nx, self.ny)
    }
}

impl fmt::Debug for SorSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SorSolver")
            .field("nx", &self.nx)
            .field("ny", &self.ny)
            .field("omega", &self.omega)
            .field("max_iters", &self.max_iters)
            .field("tolerance", &self.tolerance)
            .finish()
    }
}

// ============================================================
// 电场计算（支持周期和非周期边界）
// ============================================================

/// 在周期边界下使用中心差分计算电场
/// E = -∇V
/// 返回: (Ex, Ey)，均为 shape (nx, ny)
pub fn compute_e_from_potential_periodic(
    v: &Array2<f64>,
    dx: f64,
    dy: f64,
) -> (Array2<f64>, Array2<f64>) {
    let vx_p = shift_array2(v, Axis(0), 1isize);
    let vx_m = shift_array2(v, Axis(0), -1isize);
    let ex = -(vx_p - vx_m) / (2.0 * dx);

    let vy_p = shift_array2(v, Axis(1), 1isize);
    let vy_m = shift_array2(v, Axis(1), -1isize);
    let ey = -(vy_p - vy_m) / (2.0 * dy);

    (ex, ey)
}

/// 在非周期边界下使用中心差分计算电场（内部点）
/// 边界点使用单侧差分
/// 返回: (Ex, Ey)，均为 shape (nx, ny)
pub fn compute_e_from_potential_nonperiodic(
    v: &Array2<f64>,
    dx: f64,
    dy: f64,
) -> (Array2<f64>, Array2<f64>) {
    let (nx, ny) = v.dim();
    let mut ex = Array2::zeros((nx, ny));
    let mut ey = Array2::zeros((nx, ny));

    // 内部点：中心差分
    for i in 1..nx - 1 {
        for j in 1..ny - 1 {
            ex[[i, j]] = -(v[[i + 1, j]] - v[[i - 1, j]]) / (2.0 * dx);
            ey[[i, j]] = -(v[[i, j + 1]] - v[[i, j - 1]]) / (2.0 * dy);
        }
    }

    // X 边界：单侧差分
    for j in 0..ny {
        ex[[0, j]] = -(v[[1, j]] - v[[0, j]]) / dx;
        ex[[nx - 1, j]] = -(v[[nx - 1, j]] - v[[nx - 2, j]]) / dx;
    }

    // Y 边界：单侧差分
    for i in 0..nx {
        ey[[i, 0]] = -(v[[i, 1]] - v[[i, 0]]) / dy;
        ey[[i, ny - 1]] = -(v[[i, ny - 1]] - v[[i, ny - 2]]) / dy;
    }

    (ex, ey)
}

/// 在指定轴上滚动数组（类似 np.roll）
pub fn shift_array2(arr: &Array2<f64>, axis: Axis, shift: isize) -> Array2<f64> {
    let dim = arr.dim();
    let mut result = Array2::zeros(dim);

    let len = match axis {
        Axis(0) => dim.0,
        Axis(1) => dim.1,
        _ => unreachable!(),
    };
    let len_i = len as isize;

    for idx in 0..len {
        let src_idx = ((idx as isize + shift).rem_euclid(len_i)) as usize;
        match axis {
            Axis(0) => {
                for j in 0..dim.1 {
                    result[[idx, j]] = arr[[src_idx, j]];
                }
            }
            Axis(1) => {
                for i in 0..dim.0 {
                    result[[i, idx]] = arr[[i, src_idx]];
                }
            }
            _ => unreachable!(),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_periodic_uniform_rho() {
        let mut solver = FftPoissonSolver::new(8, 8, 1.0, 1.0, 1e-12);
        let rho = Array2::from_elem((8, 8), 1.0);
        let v = solver.solve(&rho, 1e-12);
        for val in v.iter() {
            assert!(val.abs() < 1e-10, "Uniform rho should give V=0, got {}", val);
        }
    }

    #[test]
    fn test_fft_point_charge() {
        let mut solver = FftPoissonSolver::new(16, 16, 1.0, 1.0, 1e-12);
        let mut rho = Array2::zeros((16, 16));
        rho[[8, 8]] = 1.0;
        let v = solver.solve(&rho, 1e-12);
        assert!(v[[8, 8]] > 0.0, "V at charge should be positive");
        assert!(v[[8, 8]] > v[[4, 4]], "V should decay with distance");
    }

    #[test]
    fn test_jacobi_simple() {
        let mut solver = JacobiSolver::new(16, 16, 1.0, 1.0, 1e-12)
            .with_max_iters(5000)
            .with_tolerance(1e-5);
        let mut rho = Array2::zeros((16, 16));
        rho[[8, 8]] = 1.0;
        let v = solver.solve(&rho, 1e-12, PoissonBoundaryType::Dirichlet);
        // Point charge at center with Dirichlet BC: V should be positive at center
        assert!(v[[8, 8]] > 0.0, "V at charge should be positive, got {}", v[[8, 8]]);
    }

    #[test]
    fn test_sor_vs_jacobi() {
        let mut rho = Array2::zeros((32, 32));
        rho[[16, 16]] = 1.0;

        let mut jacobi = JacobiSolver::new(32, 32, 1.0, 1.0, 1e-12)
            .with_max_iters(10000)
            .with_tolerance(1e-6);
        let v_j = jacobi.solve(&rho, 1e-12, PoissonBoundaryType::Dirichlet);

        let mut sor = SorSolver::new(32, 32, 1.0, 1.0, 1e-12)
            .with_max_iters(5000)
            .with_tolerance(1e-6);
        let v_s = sor.solve(&rho, 1e-12, PoissonBoundaryType::Dirichlet);

        // Results should be similar
        for i in 0..32 {
            for j in 0..32 {
                let diff = (v_j[[i, j]] - v_s[[i, j]]).abs();
                assert!(diff < 0.01, "SOR and Jacobi should give similar results at [{}][{}]: diff={}", i, j, diff);
            }
        }
    }

    #[test]
    fn test_nonperiodic_e_field() {
        let nx = 8;
        let ny = 8;
        let dx = 1.0;
        let dy = 1.0;
        let mut v = Array2::zeros((nx, ny));
        // Linear potential: V = x, so Ex = -1, Ey = 0
        for i in 0..nx {
            for j in 0..ny {
                v[[i, j]] = i as f64;
            }
        }
        let (ex, ey) = compute_e_from_potential_nonperiodic(&v, dx, dy);
        for i in 0..nx {
            for j in 0..ny {
                assert!((ex[[i, j]] + 1.0).abs() < 1e-10, "Ex should be -1 at [{}][{}], got {}", i, j, ex[[i, j]]);
                assert!(ey[[i, j]].abs() < 1e-10, "Ey should be 0 at [{}][{}], got {}", i, j, ey[[i, j]]);
            }
        }
    }

    #[test]
    fn test_poisson_solver_enum_creation() {
        let mut solver = PoissonSolverEnum::new(PoissonSolverType::FFTPeriodic, 8, 8, 1.0, 1.0, 1e-12);
        let rho = Array2::zeros((8, 8));
        let v = solver.solve(&rho, 1e-12, PoissonBoundaryType::Dirichlet);
        assert_eq!(v.dim(), (8, 8));
    }
}