fn print(count: &mut usize, id: usize, layout: &layout::tree::LayoutR) {
    *count += 1;
   unsafe{debugit::debugit!("result: {:?} {:?} {:?}", *count, id, layout);
}
pub fn compute() {
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
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(100f32),
                height: layout::style::Dimension::Points(100f32),
                ..Default::default()
            },
            padding: layout::geometry::Rect {
                start: layout::style::Dimension::Points(10f32),
                end: layout::style::Dimension::Points(10f32),
                top: layout::style::Dimension::Points(10f32),
                bottom: layout::style::Dimension::Points(10f32),
                ..Default::default()
            },
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
    layout_tree.insert(
        3,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            position_type: layout::style::PositionType::Absolute,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(50f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            position: layout::geometry::Rect {
                start: layout::style::Dimension::Points(0f32),
                top: layout::style::Dimension::Points(0f32),
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
            position_type: layout::style::PositionType::Absolute,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(50f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            position: layout::geometry::Rect {
                end: layout::style::Dimension::Points(0f32),
                bottom: layout::style::Dimension::Points(0f32),
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
            position_type: layout::style::PositionType::Absolute,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(50f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            margin: layout::geometry::Rect {
                start: layout::style::Dimension::Points(10f32),
                end: layout::style::Dimension::Points(10f32),
                top: layout::style::Dimension::Points(10f32),
                bottom: layout::style::Dimension::Points(10f32),
                ..Default::default()
            },
            position: layout::geometry::Rect {
                start: layout::style::Dimension::Points(0f32),
                top: layout::style::Dimension::Points(0f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        6,
        2,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            position_type: layout::style::PositionType::Absolute,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(50f32),
                height: layout::style::Dimension::Points(50f32),
                ..Default::default()
            },
            margin: layout::geometry::Rect {
                start: layout::style::Dimension::Points(10f32),
                end: layout::style::Dimension::Points(10f32),
                top: layout::style::Dimension::Points(10f32),
                bottom: layout::style::Dimension::Points(10f32),
                ..Default::default()
            },
            position: layout::geometry::Rect {
                end: layout::style::Dimension::Points(0f32),
                bottom: layout::style::Dimension::Points(0f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.compute(print, &mut 0);
}
