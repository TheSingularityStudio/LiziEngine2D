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
}

impl ParticleState {
    /// 创建 n 个粒子：位置在 [0,1) 内随机初始化，速度/力为 0
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
        }
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
        }
    }
}