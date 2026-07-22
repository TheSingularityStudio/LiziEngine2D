use ndarray::{Array2, Axis};
use ndrustfft::{ndfft, ndifft, FftHandler};
use num_complex::Complex64;
use num_traits::Zero;

/// 使用 FFT 在周期域中求解离散 Poisson 方程
///
/// 方程: ∇²V = -rho
/// 在傅里叶空间: V_hat(k) = rho_hat(k) / (kx² + ky²)
///
/// 参数:
/// - rho: 电荷密度, shape (nx, ny)
/// - dx, dy: 网格间距
/// - eps: k=0 模式阈值，避免除零
///
/// 返回: 电势 V, shape (nx, ny)
pub fn solve_poisson_via_discrete_greens_function_kernel(
    rho: &Array2<f64>,
    dx: f64,
    dy: f64,
    eps: f64,
) -> Array2<f64> {
    let (nx, ny) = rho.dim();

    // 转换为复数
    let mut rho_hat: Array2<Complex64> = rho.mapv(|v| Complex64::new(v, 0.0));

    // 2D FFT: 先沿 axis=0，再沿 axis=1
    let mut handler_0 = FftHandler::new(nx);
    let mut handler_1 = FftHandler::new(ny);
    let mut tmp = Array2::zeros((nx, ny));

    ndfft(&rho_hat, &mut tmp, &mut handler_0, 0);
    ndfft(&tmp, &mut rho_hat, &mut handler_1, 1);

    // 构造 k² 矩阵
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

    // V_hat = rho_hat / k² (k≠0 时), k=0 时设为 0
    for i in 0..nx {
        for j in 0..ny {
            if k2[[i, j]] > eps {
                rho_hat[[i, j]] = rho_hat[[i, j]] / k2[[i, j]];
            } else {
                rho_hat[[i, j]] = Complex64::zero();
            }
        }
    }

    // 逆 2D FFT: 先沿 axis=1，再沿 axis=0
    let mut tmp2 = Array2::zeros((nx, ny));
    let mut v_complex: Array2<Complex64> = Array2::zeros((nx, ny));

    ndifft(&rho_hat, &mut tmp2, &mut handler_1, 1);
    ndifft(&tmp2, &mut v_complex, &mut handler_0, 0);

    // 取实部
    v_complex.mapv(|c| c.re)
}

/// 在周期边界下使用中心差分计算电场
///
/// E = -∇V
/// E_x(i,j) = -(V(i+1,j) - V(i-1,j)) / (2dx)
/// E_y(i,j) = -(V(i,j+1) - V(i,j-1)) / (2dy)
///
/// 返回: (Ex, Ey)，均为 shape (nx, ny)
pub fn compute_e_from_potential_periodic(
    v: &Array2<f64>,
    dx: f64,
    dy: f64,
) -> (Array2<f64>, Array2<f64>) {
    let (nx, ny) = v.dim();

    // E_x: -(V(i+1,j) - V(i-1,j)) / (2dx)
    let vx_p = shift_array2(v, Axis(0), 1isize);
    let vx_m = shift_array2(v, Axis(0), -1isize);

    let mut ex = Array2::zeros((nx, ny));
    for i in 0..nx {
        for j in 0..ny {
            ex[[i, j]] = -(vx_p[[i, j]] - vx_m[[i, j]]) / (2.0 * dx);
        }
    }

    // E_y: -(V(i,j+1) - V(i,j-1)) / (2dy)
    let vy_p = shift_array2(v, Axis(1), 1isize);
    let vy_m = shift_array2(v, Axis(1), -1isize);

    let mut ey = Array2::zeros((nx, ny));
    for i in 0..nx {
        for j in 0..ny {
            ey[[i, j]] = -(vy_p[[i, j]] - vy_m[[i, j]]) / (2.0 * dy);
        }
    }

    (ex, ey)
}

/// 在指定轴上滚动数组（类似 np.roll）
fn shift_array2(arr: &Array2<f64>, axis: Axis, shift: isize) -> Array2<f64> {
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