use crate::core::sim::StateSnapshot;

/// 可视化窗口统一接口
///
/// render() 每帧调用一次，返回 false 表示窗口已关闭
pub trait VisualWindow {
    /// 使用新的模拟快照渲染窗口
    /// 返回 true 表示继续运行，false 表示用户关闭了窗口
    fn render(&mut self, snapshot: &StateSnapshot) -> bool;

    /// 窗口是否已关闭
    fn should_close(&self) -> bool;

    /// 获取当前暂停状态
    fn is_paused(&self) -> bool;
}