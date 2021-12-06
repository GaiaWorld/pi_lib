fn print(count: &mut usize, id: usize, layout: &layout::tree::LayoutR) {
    *count += 1;
    debug_println!("result: {:?} {:?} {:?}", *count, id, layout);
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
            flex_direction: layout::style::FlexDirection::Column,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Points(87.4f32),
                height: layout::style::Dimension::Points(113.4f32),
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
            flex_grow: 0.7f32,
            flex_basis: layout::style::Dimension::Points(50.3f32),
            size: layout::geometry::Size {
                height: layout::style::Dimension::Points(20.3f32),
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
            flex_grow: 1f32,
            flex_basis: layout::style::Dimension::Points(0.3f32),
            size: layout::geometry::Size {
                height: layout::style::Dimension::Points(9.9f32),
                ..Default::default()
            },
            position: layout::geometry::Rect {
                bottom: layout::style::Dimension::Points(13.3f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        5,
        3,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            flex_grow: 4f32,
            flex_basis: layout::style::Dimension::Points(0.3f32),
            size: layout::geometry::Size {
                height: layout::style::Dimension::Points(1.1f32),
                ..Default::default()
            },
            position: layout::geometry::Rect {
                top: layout::style::Dimension::Points(13.3f32),
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
            flex_grow: 1.6f32,
            size: layout::geometry::Size {
                height: layout::style::Dimension::Points(10f32),
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
            flex_grow: 1.1f32,
            size: layout::geometry::Size {
                height: layout::style::Dimension::Points(10.7f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.compute(print, &mut 0);
}
