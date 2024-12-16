
use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash)]
    pub struct NodeState: u32 {
        // 子节点布局需要重新计算
        const ChildrenDirty =       0b0000000000000001;
        // 自身布局需要重新计算
        const SelfDirty =           0b0000000000000010;
        // 子节点是否都是绝对坐标， 如果是，则本节点的自动大小为0.0
        const ChildrenAbs =         0b0000000000000100;
        // 子节点没有设置align_self
        const ChildrenNoAlignSelf = 0b0000000000001000;
        // 子节点是否为顺序排序
        const ChildrenIndex =       0b0000000000010000;
        // 是否为虚拟节点, 虚拟节点下只能放叶子节点
        const VNode =               0b0000000000100000;
        // 是否为绝对坐标
        const Abs =                 0b0000000001000000;
        // 是否为根据子节点自动计算大小
        const SizeDefined =         0b0000000010000000;
        // 如果该元素为行首，则margin_start为0
        const LineStartMarginZero = 0b0000000100000000;
        // 强制换行
        const BreakLine =           0b0000001000000000;
        // 真实节点，对应的虚拟节点一般是组件节点
        const RNode =               0b0000010000000000;
        // 所有子节点都是SelfRect
        const ChildrenRect =        0b0000100000000000;
        // 自身区域不受父节点或子节点影响
        const SelfRect =            0b0001000000000000;
    }
}
impl NodeState {

    pub(crate) fn set_true(&mut self, s: Self) {
        *self |= s;
    }
    pub(crate) fn set_false(&mut self, s: Self) {
        *self &= !s;
    }
}
impl Default for NodeState {
    fn default() -> Self {
        NodeState::ChildrenAbs | NodeState::ChildrenRect
            | NodeState::ChildrenNoAlignSelf
            | NodeState::ChildrenIndex
    }
}
impl Serialize for NodeState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}
impl<'de> Deserialize<'de> for NodeState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u32::deserialize(deserializer)?;
        Ok(NodeState::from_bits_truncate(bits))
    }
}