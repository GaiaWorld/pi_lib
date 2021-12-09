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
            flex_direction: layout::style::FlexDirection::Column,
            flex_grow: 1f32,
            flex_basis: layout::style::Dimension::Percent(0.1f32),
            min_size: layout::geometry::Size {
                width: layout::style::Dimension::Percent(0.6f32),
                ..Default::default()
            },
            margin: layout::geometry::Rect {
                start: layout::style::Dimension::Points(5f32),
                end: layout::style::Dimension::Points(5f32),
                top: layout::style::Dimension::Points(5f32),
                bottom: layout::style::Dimension::Points(5f32),
                ..Default::default()
            },
            padding: layout::geometry::Rect {
                start: layout::style::Dimension::Points(3f32),
                end: layout::style::Dimension::Points(3f32),
                top: layout::style::Dimension::Points(3f32),
                bottom: layout::style::Dimension::Points(3f32),
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
            flex_direction: layout::style::FlexDirection::Column,
            size: layout::geometry::Size {
                width: layout::style::Dimension::Percent(0.5f32),
                ..Default::default()
            },
            margin: layout::geometry::Rect {
                start: layout::style::Dimension::Points(5f32),
                end: layout::style::Dimension::Points(5f32),
                top: layout::style::Dimension::Points(5f32),
                bottom: layout::style::Dimension::Points(5f32),
                ..Default::default()
            },
            padding: layout::geometry::Rect {
                start: layout::style::Dimension::Percent(0.03f32),
                end: layout::style::Dimension::Percent(0.03f32),
                top: layout::style::Dimension::Percent(0.03f32),
                bottom: layout::style::Dimension::Percent(0.03f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.insert(
        5,
        4,
        0,
        layout::idtree::InsertType::Back,
        layout::style::Style {
            size: layout::geometry::Size {
                width: layout::style::Dimension::Percent(0.45f32),
                ..Default::default()
            },
            margin: layout::geometry::Rect {
                start: layout::style::Dimension::Percent(0.05f32),
                end: layout::style::Dimension::Percent(0.05f32),
                top: layout::style::Dimension::Percent(0.05f32),
                bottom: layout::style::Dimension::Percent(0.05f32),
                ..Default::default()
            },
            padding: layout::geometry::Rect {
                start: layout::style::Dimension::Points(3f32),
                end: layout::style::Dimension::Points(3f32),
                top: layout::style::Dimension::Points(3f32),
                bottom: layout::style::Dimension::Points(3f32),
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
            flex_grow: 4f32,
            flex_basis: layout::style::Dimension::Percent(0.15f32),
            min_size: layout::geometry::Size {
                width: layout::style::Dimension::Percent(0.2f32),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    layout_tree.compute(print, &mut 0);
}
