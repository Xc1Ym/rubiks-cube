pub mod view_2d;
pub mod view_3d;

use crate::cube::moves::Face;

/// 当前正在进行的旋转动画状态（3D/2D 共用）
#[derive(Clone, Copy, Debug)]
pub struct RotationAnim {
    pub face: Face,
    pub angle: f32, // 当前旋转角度（弧度），从 0 到目标角度
}
