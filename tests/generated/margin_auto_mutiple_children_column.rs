fn print(count: &mut usize, id: usize, layout: &layout::tree::LayoutR) {
    *count += 1;
   unsafe{debugit::debugit!("result: {:?} {:?} {:?}", *count, id, layout);
}
#[test]
fn margin_auto_mutiple_children_column() {
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
            align_items: layout::style::AlignItems::Center,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(200f32),
                height: layout::style::Dimension::Points(200f32),
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
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(50f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            margin: layout::geometry::Rect {
                top: layout::style::Dimension::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        4,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(50f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            margin: layout::geometry::Rect {
                top: layout::style::Dimension::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        5,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(50f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.compute(print, &mut 0);
    let layout = layout_tree.get_layout(2).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 200f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 200f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 0f32);
    let layout = layout_tree.get_layout(3).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 50f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 50f32);
    assert_eq!(layout.rect.start, 75f32);
    assert_eq!(layout.rect.top, 25f32);
    let layout = layout_tree.get_layout(4).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 50f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 50f32);
    assert_eq!(layout.rect.start, 75f32);
    assert_eq!(layout.rect.top, 100f32);
    let layout = layout_tree.get_layout(5).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 50f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 50f32);
    assert_eq!(layout.rect.start, 75f32);
    assert_eq!(layout.rect.top, 150f32);
}
