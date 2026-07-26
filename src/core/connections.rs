use crate::core::particles::ParticleState;

/// 连接类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    /// 弹簧：始终施加胡克定律力（拉伸和压缩）
    Spring,
    /// 绳子：仅在拉伸时施加力
    Rope,
}

impl ConnectionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ConnectionType::Spring => "弹簧",
            ConnectionType::Rope => "绳子",
        }
    }
}

/// 粒子间的连接
#[derive(Debug, Clone)]
pub struct Connection {
    pub particle_a: usize,
    pub particle_b: usize,
    pub rest_length: f64,
    pub stiffness: f64,
    pub connection_type: ConnectionType,
}

/// 连接集合
#[derive(Debug, Clone)]
pub struct Connections {
    pub list: Vec<Connection>,
}

impl Connections {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    /// 添加新连接
    pub fn add(&mut self, conn: Connection) {
        self.list.push(conn);
    }

    /// 删除指定索引的连接
    pub fn remove(&mut self, index: usize) -> bool {
        if index < self.list.len() {
            self.list.remove(index);
            true
        } else {
            false
        }
    }

    /// 删除涉及指定粒子的所有连接
    /// 返回被删除的连接数量
    pub fn remove_for_particle(&mut self, particle_index: usize) -> usize {
        let before = self.list.len();
        self.list.retain(|c| c.particle_a != particle_index && c.particle_b != particle_index);
        before - self.list.len()
    }

    /// 查找包含指定粒子的连接索引列表
    pub fn find_for_particle(&self, particle_index: usize) -> Vec<usize> {
        self.list.iter().enumerate()
            .filter(|(_, c)| c.particle_a == particle_index || c.particle_b == particle_index)
            .map(|(i, _)| i)
            .collect()
    }

    /// 在删除粒子索引后重新映射连接中的索引
    /// 当粒子 index 被删除时，所有 > index 的粒子索引减1
    pub fn remap_after_removal(&mut self, removed_index: usize) {
        for conn in self.list.iter_mut() {
            if conn.particle_a > removed_index {
                conn.particle_a -= 1;
            }
            if conn.particle_b > removed_index {
                conn.particle_b -= 1;
            }
        }
    }

    /// 应用所有连接的力到粒子受力数组
    /// 在积分前调用，力叠加到 particles.fx/fy
    pub fn apply_forces(&self, particles: &mut ParticleState) {
        for conn in &self.list {
            // 获取两个粒子的位置
            let ax = particles.x[conn.particle_a];
            let ay = particles.y[conn.particle_a];
            let bx = particles.x[conn.particle_b];
            let by = particles.y[conn.particle_b];

            let dx = bx - ax;
            let dy = by - ay;
            let current_length = (dx * dx + dy * dy).sqrt();

            // 避免除以零
            if current_length < 1e-15 {
                continue;
            }

            let dir_x = dx / current_length;
            let dir_y = dy / current_length;

            // 计算弹力大小
            let displacement = current_length - conn.rest_length;

            let force_magnitude = match conn.connection_type {
                ConnectionType::Spring => {
                    // 弹簧：F = -k * displacement，拉伸和压缩都作用
                    -conn.stiffness * displacement
                }
                ConnectionType::Rope => {
                    // 绳子：仅在拉伸时施加力
                    if displacement > 0.0 {
                        -conn.stiffness * displacement
                    } else {
                        continue; // 绳子不施加推力
                    }
                }
            };

            // 施加到两个粒子（方向相反）
            let fx = force_magnitude * dir_x;
            let fy = force_magnitude * dir_y;

            particles.fx[conn.particle_a] += fx;
            particles.fy[conn.particle_a] += fy;
            particles.fx[conn.particle_b] -= fx;
            particles.fy[conn.particle_b] -= fy;
        }
    }
}