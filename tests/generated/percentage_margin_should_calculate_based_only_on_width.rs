fn print(count: &mut usize, id: usize, layout: &layout::tree::LayoutR) {
    *count += 1;
   unsafe{debugit::debugit!("result: {:?} {:?} {:?}", *count, id, layout);
}
#[test]
fn percentage_margin_should_calculate_based_only_on_width() {
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
            flex_direction: layout::style::FlexDirection::Column,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(200f32),
                height: layout::style::Dimension::Points(100f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        3,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            flex_direction: layout::style::FlexDirection::Column,
            flex_grow: 1f32,
            margin: layout::geometry::Rect {
                start: layout::style::Dimension::Percent(0.1f32),
                end: layout::style::Dimension::Percent(0.1f32),
                top: layout::style::Dimension::Percent(0.1f32),
                bottom: layout::style::Dimension::Percent(0.1f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        4,
        3,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(10f32),
                height: layout::style::Dimension::Points(10f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.compute(print, &mut 0);
    let layout = layout_tree.get_layout(2).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 200f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 100f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 0f32);
    let layout = layout_tree.get_layout(3).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 160f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 60f32);
    assert_eq!(layout.rect.start, 20f32);
    assert_eq!(layout.rect.top, 20f32);
    let layout = layout_tree.get_layout(4).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 10f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 10f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 0f32);
}
