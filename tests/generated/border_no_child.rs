fn print(count: &mut usize, id: usize, layout: &layout::tree::LayoutR) {
    *count += 1;
   unsafe{debugit::debugit!("result: {:?} {:?} {:?}", *count, id, layout);
}
#[test]
fn border_no_child() {
    let mut layout_tree = layout::tree::LayoutTree::default();
    layout_tree.insert(
        1,
        0,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            position_type: layout::style::PositionType::Absolute,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(1920.0),
                height: layout::style::Dimension::Points(1024.0),
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        2,
        1,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            border: layout::geometry::Rect {
                start: layout::style::Dimension::Points(10f32),
                end: layout::style::Dimension::Points(10f32),
                top: layout::style::Dimension::Points(10f32),
                bottom: layout::style::Dimension::Points(10f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.compute(print, &mut 0);
    let layout = layout_tree.get_layout(2).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 20f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 20f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 0f32);
}
