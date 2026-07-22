use ndarray::Array1;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

/// 粒子状态数据结构
#[derive(Debug, Clone)]
pub struct ParticleState {
    pub x: Array1<f64>,   // shape (N,)
    pub y: Array1<f64>,   // shape (N,)
    pub vx: Array1<f64>,  // shape (N,)
    pub vy: Array1<f64>,  // shape (N,)
    pub fx: Array1<f64>,  // shape (N,)
    pub fy: Array1<f64>,  // shape (N,)
    pub q: Array1<f64>,   // shape (N,) — 电荷量，默认 1.0
}

impl ParticleState {
    /// 创建 n 个粒子：位置在 [0,1) 内随机初始化，速度/力为 0，电荷为 1.0
    pub fn zeros(n: usize, seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or(0);
        let mut rng = StdRng::seed_from_u64(seed);
        let x: Array1<f64> = (0..n).map(|_| rng.gen()).collect();
        let y: Array1<f64> = (0..n).map(|_| rng.gen()).collect();
        Self {
            x,
            y,
            vx: Array1::zeros(n),
            vy: Array1::zeros(n),
            fx: Array1::zeros(n),
            fy: Array1::zeros(n),
            q: Array1::ones(n),
        }
    }

    /// 创建 n 个粒子，并指定每个粒子的电荷量
    pub fn with_charges(n: usize, seed: Option<u64>, charges: &[f64]) -> Self {
        let mut s = Self::zeros(n, seed);
        let len = charges.len().min(n);
        for i in 0..len {
            s.q[i] = charges[i];
        }
        s
    }

    pub fn len(&self) -> usize {
        self.x.len()
    }

    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// 深拷贝
    pub fn copy(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            vx: self.vx.clone(),
            vy: self.vy.clone(),
            fx: self.fx.clone(),
            fy: self.fy.clone(),
            q: self.q.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeros_creates_correct_number_of_particles() {
        let n = 10;
        let particles = ParticleState::zeros(n, Some(42));
        assert_eq!(particles.len(), n);
        assert_eq!(particles.x.len(), n);
        assert_eq!(particles.y.len(), n);
        assert_eq!(particles.vx.len(), n);
        assert_eq!(particles.vy.len(), n);
        assert_eq!(particles.fx.len(), n);
        assert_eq!(particles.fy.len(), n);
        assert_eq!(particles.q.len(), n);
    }

    #[test]
    fn test_zeros_initializes_velocity_and_force_to_zero() {
        let particles = ParticleState::zeros(5, Some(0));
        for i in 0..5 {
            assert_eq!(particles.vx[i], 0.0);
            assert_eq!(particles.vy[i], 0.0);
            assert_eq!(particles.fx[i], 0.0);
            assert_eq!(particles.fy[i], 0.0);
        }
    }

    #[test]
    fn test_zeros_initializes_charge_to_one() {
        let particles = ParticleState::zeros(5, Some(0));
        for i in 0..5 {
            assert!((particles.q[i] - 1.0).abs() < 1e-15);
        }
    }

    #[test]
    fn test_with_charges_sets_custom_charges() {
        let charges = vec![1.0, -1.0, 2.5, -0.5];
        let particles = ParticleState::with_charges(4, Some(0), &charges);
        assert!((particles.q[0] - 1.0).abs() < 1e-15);
        assert!((particles.q[1] + 1.0).abs() < 1e-15);
        assert!((particles.q[2] - 2.5).abs() < 1e-15);
        assert!((particles.q[3] + 0.5).abs() < 1e-15);
    }

    #[test]
    fn test_with_charges_truncates_to_n() {
        let charges = vec![1.0, -1.0];
        let particles = ParticleState::with_charges(1, Some(0), &charges);
        assert_eq!(particles.len(), 1);
        assert!((particles.q[0] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_copy_produces_independent_clone() {
        let mut p1 = ParticleState::zeros(3, Some(42));
        p1.x[0] = 99.0;
        p1.q[1] = -5.0;
        let p2 = p1.copy();
        // Modify original
        p1.x[0] = 0.0;
        p1.q[1] = 0.0;
        // Copy should be unchanged
        assert!((p2.x[0] - 99.0).abs() < 1e-15);
        assert!((p2.q[1] + 5.0).abs() < 1e-15);
    }

    #[test]
    fn test_is_empty() {
        let particles = ParticleState::zeros(0, Some(0));
        assert!(particles.is_empty());
        let particles = ParticleState::zeros(1, Some(0));
        assert!(!particles.is_empty());
    }

    #[test]
    fn test_seed_consistency() {
        let p1 = ParticleState::zeros(100, Some(42));
        let p2 = ParticleState::zeros(100, Some(42));
        for i in 0..100 {
            assert!((p1.x[i] - p2.x[i]).abs() < 1e-15);
            assert!((p1.y[i] - p2.y[i]).abs() < 1e-15);
        }
    }

    #[test]
    fn test_different_seed_produces_different_positions() {
        let p1 = ParticleState::zeros(100, Some(0));
        let p2 = ParticleState::zeros(100, Some(1));
        // At least one position should differ
        let all_same = p1.x.iter().zip(p2.x.iter()).all(|(a, b)| (a - b).abs() < 1e-15);
        assert!(!all_same);
    }
}