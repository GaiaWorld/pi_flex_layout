fn print<T: pi_flex_layout::prelude::LayoutR + std::fmt::Debug>(
    _arg: &mut (),
    id: pi_slotmap_tree::TreeKey,
    layout: &T,
) {
    unsafe { debugit::debugit!("result: {:?} {:?}", id, layout) };
}
#[test]
fn min_max_percent_no_width_height() {
    let mut layout_tree = pi_flex_layout::prelude::LayoutTree::default();
    let node_1 = layout_tree.create_node();
    layout_tree.insert(
        node_1,
        <pi_slotmap_tree::TreeKey as pi_null::Null>::null(),
        <pi_slotmap_tree::TreeKey as pi_null::Null>::null(),
        pi_slotmap_tree::InsertType::Back,
        pi_flex_layout::prelude::Style {
            position_type: pi_flex_layout::prelude::PositionType::Absolute,
            size: pi_flex_layout::prelude::Size {
                width: pi_flex_layout::prelude::Dimension::Points(1920.0),
                height: pi_flex_layout::prelude::Dimension::Points(1024.0),
            },
            position: pi_flex_layout::prelude::Rect {
                left: pi_flex_layout::prelude::Dimension::Points(0.0),
                right: pi_flex_layout::prelude::Dimension::Points(0.0),
                top: pi_flex_layout::prelude::Dimension::Points(0.0),
                bottom: pi_flex_layout::prelude::Dimension::Points(0.0),
            },
            margin: pi_flex_layout::prelude::Rect {
                left: pi_flex_layout::prelude::Dimension::Points(0.0),
                right: pi_flex_layout::prelude::Dimension::Points(0.0),
                top: pi_flex_layout::prelude::Dimension::Points(0.0),
                bottom: pi_flex_layout::prelude::Dimension::Points(0.0),
            },
            ..Default::default()
        },
    );
    let node_2 = layout_tree.create_node();
    layout_tree.insert(
        node_2,
        node_1,
        <pi_slotmap_tree::TreeKey as pi_null::Null>::null(),
        pi_slotmap_tree::InsertType::Back,
        pi_flex_layout::prelude::Style {
            flex_direction: pi_flex_layout::prelude::FlexDirection::Column,
            align_items: pi_flex_layout::prelude::AlignItems::FlexStart,
            size: pi_flex_layout::prelude::Size {
                width: pi_flex_layout::prelude::Dimension::Points(100f32),
                height: pi_flex_layout::prelude::Dimension::Points(100f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let node_3 = layout_tree.create_node();
    layout_tree.insert(
        node_3,
        node_2,
        <pi_slotmap_tree::TreeKey as pi_null::Null>::null(),
        pi_slotmap_tree::InsertType::Back,
        pi_flex_layout::prelude::Style {
            min_size: pi_flex_layout::prelude::Size {
                width: pi_flex_layout::prelude::Dimension::Percent(0.1f32),
                height: pi_flex_layout::prelude::Dimension::Percent(0.1f32),
                ..Default::default()
            },
            max_size: pi_flex_layout::prelude::Size {
                width: pi_flex_layout::prelude::Dimension::Percent(0.1f32),
                height: pi_flex_layout::prelude::Dimension::Percent(0.1f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let c = pi_flex_layout::prelude::LayoutContext {
        mark: std::marker::PhantomData,
        i_nodes: &mut layout_tree.node_states,
        layout_map: &mut pi_flex_layout::prelude::LayoutResults(&mut layout_tree.layout_map),
        notify_arg: &mut (),
        notify: print,
        tree: &mut layout_tree.tree,
        style: &mut layout_tree.style_map,
    };
    let mut layout = pi_flex_layout::prelude::Layout(c);
    layout.compute(&mut layout_tree.dirty);
    let layout = layout_tree.get_layout(node_2).unwrap();
    assert_eq!((layout.rect.right - layout.rect.left).round(), 100f32);
    assert_eq!((layout.rect.bottom - layout.rect.top).round(), 100f32);
    assert_eq!(layout.rect.left.round(), 0f32);
    assert_eq!(layout.rect.top.round(), 0f32);
    let layout = layout_tree.get_layout(node_3).unwrap();
    assert_eq!((layout.rect.right - layout.rect.left).round(), 10f32);
    assert_eq!((layout.rect.bottom - layout.rect.top).round(), 10f32);
    assert_eq!(layout.rect.left.round(), 0f32);
    assert_eq!(layout.rect.top.round(), 0f32);
}
