pub mod core;
pub mod visual;
pub mod demo;

// 重新导出 core 模块，保持外部 API 兼容
pub use core::grid;
pub use core::integrator;
pub use core::interp;
pub use core::particles;
pub use core::poisson_fft;
pub use core::scatter;
pub use core::sim;
