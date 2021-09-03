#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use crate::geometry::{Rect, Size};
use crate::number::Number;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

impl Default for AlignItems {
    fn default() -> AlignItems {
        AlignItems::FlexStart
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

impl Default for AlignSelf {
    fn default() -> AlignSelf {
        AlignSelf::Auto
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
}

impl Default for AlignContent {
    fn default() -> AlignContent {
        AlignContent::Stretch
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Inherit,
    LTR,
    RTL,
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Inherit
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Display {
    Flex,
	None,
}

impl Default for Display {
    fn default() -> Display {
        Display::Flex
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl Default for FlexDirection {
    fn default() -> FlexDirection {
        FlexDirection::Row
    }
}

// impl FlexDirection {
//     pub(crate) fn is_row(self) -> bool {
//         self == FlexDirection::Row || self == FlexDirection::RowReverse
//     }

//     pub(crate) fn is_column(self) -> bool {
//         self == FlexDirection::Column || self == FlexDirection::ColumnReverse
//     }

//     pub(crate) fn is_reverse(self) -> bool {
//         self == FlexDirection::RowReverse || self == FlexDirection::ColumnReverse
//     }
// }

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> JustifyContent {
        JustifyContent::FlexStart
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
}

impl Default for Overflow {
    fn default() -> Overflow {
        Overflow::Visible
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PositionType {
    Relative,
    Absolute,
}

impl Default for PositionType {
    fn default() -> PositionType {
        PositionType::Relative
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl Default for FlexWrap {
    fn default() -> FlexWrap {
        FlexWrap::NoWrap
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum Dimension {
    Undefined,
    Auto,
    Points(f32),
    Percent(f32),
}

impl Default for Dimension {
    fn default() -> Dimension {
        Dimension::Points(0.0)
    }
}

impl Dimension {
    pub(crate) fn resolve_value(self, parent: f32) -> f32 {
        match self {
            Dimension::Points(points) => points,
            Dimension::Percent(percent) => parent * percent,
            _ => 0.0,
        }
    }
    // pub(crate) fn resolve(self, parent_width: Number) -> Number {
    //     match self {
    //         Dimension::Points(points) => Number::Defined(points),
    //         Dimension::Percent(percent) => parent_width * percent,
    //         _ => Number::Undefined,
    //     }
    // }

    pub(crate) fn is_defined(self) -> bool {
        match self {
            Dimension::Points(_) => true,
            Dimension::Percent(_) => true,
            _ => false,
        }
    }
    pub(crate) fn is_undefined(self) -> bool {
        match self {
            Dimension::Points(_) => false,
            Dimension::Percent(_) => false,
            _ => true,
        }
    }
    pub(crate) fn is_points(self) -> bool {
        match self {
            Dimension::Points(_) => true,
            _ => false,
        }
    }
}

impl Default for Rect<Dimension> {
    fn default() -> Rect<Dimension> {
        Rect {
            start: Default::default(),
            end: Default::default(),
            top: Default::default(),
            bottom: Default::default(),
        }
    }
}

impl Default for Size<Dimension> {
    fn default() -> Size<Dimension> {
        Size {
            width: Dimension::Undefined,
            height: Dimension::Undefined,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RectStyle {
    pub margin: Rect<Dimension>,
    pub size: Size<Dimension>,
}

impl Default for RectStyle {
    fn default() -> RectStyle {
        RectStyle {
            margin: Default::default(), // dom默认为undefined， 性能考虑，这里默认0.0
            size: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OtherStyle {
    pub display: Display,
    pub position_type: PositionType,
    pub direction: Direction,

    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,

    pub order: isize,
    pub flex_basis: Dimension,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub align_self: AlignSelf,

    pub overflow: Overflow,
    pub position: Rect<Dimension>,
    pub padding: Rect<Dimension>,
    pub border: Rect<Dimension>,
    pub min_size: Size<Dimension>,
    pub max_size: Size<Dimension>,
    pub aspect_ratio: Number,


}

impl Default for OtherStyle {
    fn default() -> OtherStyle {
        OtherStyle {
            display: Default::default(),
            position_type: Default::default(),
            direction: Default::default(),
            flex_direction: Default::default(),
            flex_wrap: Default::default(),
            overflow: Default::default(),
            align_items: Default::default(), // dom默认为stretch， 性能考虑，这里默认flex_start
            align_self: Default::default(),
            align_content: Default::default(),
            justify_content: Default::default(),
            position: Default::default(),
            padding: Default::default(),
            border: Default::default(),
            flex_grow: 0.0,
            flex_shrink: 0.0,  // dom默认为1.0， 性能考虑，这里默认0.0
            order: 0,
            flex_basis: Dimension::Auto,
            min_size: Default::default(),
            max_size: Default::default(),
            aspect_ratio: Default::default(),
        }
    }
}

impl OtherStyle {
    // pub(crate) fn min_main_size(&self, direction: FlexDirection) -> Dimension {
    //     match direction {
    //         FlexDirection::Row | FlexDirection::RowReverse => self.min_size.width,
    //         FlexDirection::Column | FlexDirection::ColumnReverse => self.min_size.height,
    //     }
    // }

    // pub(crate) fn max_main_size(&self, direction: FlexDirection) -> Dimension {
    //     match direction {
    //         FlexDirection::Row | FlexDirection::RowReverse => self.max_size.width,
    //         FlexDirection::Column | FlexDirection::ColumnReverse => self.max_size.height,
    //     }
    // }

    
    // pub(crate) fn min_cross_size(&self, direction: FlexDirection) -> Dimension {
    //     match direction {
    //         FlexDirection::Row | FlexDirection::RowReverse => self.min_size.height,
    //         FlexDirection::Column | FlexDirection::ColumnReverse => self.min_size.width,
    //     }
    // }

    // pub(crate) fn max_cross_size(&self, direction: FlexDirection) -> Dimension {
    //     match direction {
    //         FlexDirection::Row | FlexDirection::RowReverse => self.max_size.height,
    //         FlexDirection::Column | FlexDirection::ColumnReverse => self.max_size.width,
    //     }
    // }

    // pub(crate) fn align_self(&self, parent: &OtherStyle) -> AlignSelf {
    //     if self.align_self == AlignSelf::Auto {
    //         match parent.align_items {
    //             AlignItems::FlexStart => AlignSelf::FlexStart,
    //             AlignItems::FlexEnd => AlignSelf::FlexEnd,
    //             AlignItems::Center => AlignSelf::Center,
    //             AlignItems::Baseline => AlignSelf::Baseline,
    //             AlignItems::Stretch => AlignSelf::Stretch,
    //         }
    //     } else {
    //         self.align_self
    //     }
    // }
}

// #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
// pub struct Style {
//     pub display: Display,
//     pub position_type: PositionType,
//     pub direction: Direction,

//     pub flex_direction: FlexDirection,
//     pub flex_wrap: FlexWrap,
//     pub justify_content: JustifyContent,
//     pub align_items: AlignItems,
//     pub align_content: AlignContent,

//     pub order: isize,
//     pub flex_basis: Dimension,
//     pub flex_grow: f32,
//     pub flex_shrink: f32,
//     pub align_self: AlignSelf,

//     pub overflow: Overflow,
//     pub position: Rect<Dimension>,
//     pub margin: Rect<Dimension>,
//     pub padding: Rect<Dimension>,
//     pub border: Rect<Dimension>,
//     pub size: Size<Dimension>,
//     pub min_size: Size<Dimension>,
//     pub max_size: Size<Dimension>,
// 	pub aspect_ratio: Number,
// 	pub line_start_margin: Number, // 行首的margin_start
// }

// impl Default for Style {
//     fn default() -> Style {
//         Style {
//             display: Default::default(),
//             position_type: Default::default(),
//             direction: Default::default(),
//             flex_direction: Default::default(),
//             flex_wrap: Default::default(),
//             overflow: Default::default(),
//             align_items: Default::default(), // dom默认为stretch， 性能考虑，这里默认flex_start
//             align_self: Default::default(),
//             align_content: Default::default(),
//             justify_content: Default::default(),
//             position: Default::default(),
//             margin: Default::default(), // dom默认为undefined， 性能考虑，这里默认0.0
//             padding: Default::default(),
//             border: Default::default(),
//             flex_grow: 0.0,
//             flex_shrink: 0.0,  // dom默认为1.0， 性能考虑，这里默认0.0
//             order: 0,
//             flex_basis: Dimension::Auto,
//             size: Default::default(),
//             min_size: Default::default(),
//             max_size: Default::default(),
// 			aspect_ratio: Default::default(),
// 			line_start_margin: Default::default(),
//         }
//     }
// }


// impl Style {
//     pub(crate) fn min_main_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.min_size.width,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.min_size.height,
//         }
//     }

//     pub(crate) fn max_main_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.max_size.width,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.max_size.height,
//         }
//     }

//     pub(crate) fn main_margin_start(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.margin.start,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.margin.top,
//         }
//     }

//     pub(crate) fn main_margin_end(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.margin.end,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.margin.bottom,
//         }
//     }

//     pub(crate) fn cross_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.size.height,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.size.width,
//         }
//     }

//     pub(crate) fn min_cross_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.min_size.height,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.min_size.width,
//         }
//     }

//     pub(crate) fn max_cross_size(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.max_size.height,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.max_size.width,
//         }
//     }

//     pub(crate) fn cross_margin_start(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.margin.top,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.margin.start,
//         }
//     }

//     pub(crate) fn cross_margin_end(&self, direction: FlexDirection) -> Dimension {
//         match direction {
//             FlexDirection::Row | FlexDirection::RowReverse => self.margin.bottom,
//             FlexDirection::Column | FlexDirection::ColumnReverse => self.margin.end,
//         }
//     }

//     pub(crate) fn align_self(&self, parent: &Style) -> AlignSelf {
//         if self.align_self == AlignSelf::Auto {
//             match parent.align_items {
//                 AlignItems::FlexStart => AlignSelf::FlexStart,
//                 AlignItems::FlexEnd => AlignSelf::FlexEnd,
//                 AlignItems::Center => AlignSelf::Center,
//                 AlignItems::Baseline => AlignSelf::Baseline,
//                 AlignItems::Stretch => AlignSelf::Stretch,
//             }
//         } else {
//             self.align_self
//         }
//     }
// }

// #[test]
// fn test(){
// 	use map;
// 	use slab;
// 	let mut vec = Vec::new();

// 	let time = std::time::Instant::now();
// 	for i in 0..1000000 {
// 		vec.push(Some(Style::default()));
// 	}
// 	let r = None;
// 	debug_println!("size:{:?}", std::mem::size_of_val(&r));
// 	debug_println!("size:{:?}", std::mem::size_of::<Option<usize>>());
// 	vec.push(r);
// 	debug_println!("{:?}", std::time::Instant::now() - time);


// 	let mut vec = map::vecmap::VecMap::new();
// 	let time = std::time::Instant::now();
// 	for i in 1..1000001 {
// 		vec.insert(i, Style::default());
// 	}
// 	debug_println!("vecmap1:{:?}", std::time::Instant::now() - time);
	

// 	let mut vec = map::vecmap::VecMap::new();
// 	let time = std::time::Instant::now();
// 	vec.insert(1000000, Style::default());
// 	debug_println!("vecmap2: {:?}", std::time::Instant::now() - time);


// 	let mut vec = slab::Slab::new();
// 	let time = std::time::Instant::now();
// 	for i in 1..1000001 {
// 		vec.insert(Style::default());
// 	}
// 	debug_println!("slab1:{:?}", std::time::Instant::now() - time);
// }
