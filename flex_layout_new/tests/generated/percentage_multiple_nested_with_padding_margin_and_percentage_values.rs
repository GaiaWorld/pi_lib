fn print(count: &mut usize, id: usize, layout: &layout::tree::LayoutR) {
    *count += 1;
    debug_println!("result: {:?} {:?} {:?}", *count, id, layout);
}
#[test]
fn percentage_multiple_nested_with_padding_margin_and_percentage_values() {
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
    let layout = layout_tree.get_layout(2).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 200f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 200f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 0f32);
    let layout = layout_tree.get_layout(3).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 190f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 48f32);
    assert_eq!(layout.rect.start, 5f32);
    assert_eq!(layout.rect.top, 5f32);
    let layout = layout_tree.get_layout(4).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 92f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 25f32);
    assert_eq!(layout.rect.start, 8f32);
    assert_eq!(layout.rect.top, 8f32);
    let layout = layout_tree.get_layout(5).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 36f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 6f32);
    assert_eq!(layout.rect.start, 10f32);
    assert_eq!(layout.rect.top, 10f32);
    let layout = layout_tree.get_layout(4).unwrap();
    assert_eq!(layout.rect.end - layout.rect.start, 200f32);
    assert_eq!(layout.rect.bottom - layout.rect.top, 142f32);
    assert_eq!(layout.rect.start, 0f32);
    assert_eq!(layout.rect.top, 58f32);
}
