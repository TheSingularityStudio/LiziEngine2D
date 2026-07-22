/// 将浮点值映射到 RGB 热力图颜色（类似 viridis 风格）
/// value 应在 [min, max] 范围内
pub fn heatmap_rgb(value: f64, min: f64, max: f64) -> (u8, u8, u8) {
    let range = max - min;
    if range < 1e-12 {
        return (68, 1, 84); // 深紫，默认值
    }
    let t = ((value - min) / range).clamp(0.0, 1.0);

    let r = viridis_r(t);
    let g = viridis_g(t);
    let b = viridis_b(t);

    (r, g, b)
}

// 简化 viridis colormap 近似
fn viridis_r(t: f64) -> u8 {
    if t < 0.25 {
        (68.0 + t * 4.0 * (253.0 - 68.0)) as u8
    } else if t < 0.5 {
        (253.0 - (t - 0.25) * 4.0 * 200.0) as u8
    } else if t < 0.75 {
        (53.0 + (t - 0.5) * 4.0 * 100.0) as u8
    } else {
        (153.0 + (t - 0.75) * 4.0 * 100.0) as u8
    }
}

fn viridis_g(t: f64) -> u8 {
    if t < 0.25 {
        (1.0 + t * 4.0 * 140.0) as u8
    } else if t < 0.5 {
        (141.0 + (t - 0.25) * 4.0 * 60.0) as u8
    } else if t < 0.75 {
        (201.0 - (t - 0.5) * 4.0 * 60.0) as u8
    } else {
        (141.0 - (t - 0.75) * 4.0 * 141.0) as u8
    }
}

fn viridis_b(t: f64) -> u8 {
    if t < 0.25 {
        (84.0 - t * 4.0 * 50.0) as u8
    } else if t < 0.5 {
        (34.0 + (t - 0.25) * 4.0 * 120.0) as u8
    } else if t < 0.75 {
        (154.0 + (t - 0.5) * 4.0 * 80.0) as u8
    } else {
        (234.0 - (t - 0.75) * 4.0 * 234.0) as u8
    }
}

/// 将 u32 RGB 打包为 0x00RRGGBB 格式
pub fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}