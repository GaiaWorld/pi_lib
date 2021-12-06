#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
use core::mem::replace;

// use map::vecmap::VecMap;
use std::ops::{Index, IndexMut};
use std::cmp::{Ordering};
use core::cmp::Ord;
use crate::geometry::*;
use crate::number::*;
use crate::style::*;
use idtree::*;
use heap::simple_heap::SimpleHeap;


fn ppp() -> String {
    let mut s = String::from("");
    for _ in 0..unsafe { PC } {
        s.push_str("--");
    }
    for _ in 0..unsafe { PP } {
        s.push_str("**");
    }
    s
}
// 每个子节点根据 justify-content align-items align-self，来计算main cross的位置和大小
macro_rules! item_calc {
    ($self:ident, $tree:ident, $i_nodes:ident, $rect_style_map:ident, $other_style_map:ident, $layout_map:ident, $notify:ident, $notify_arg:ident, $start:ident, $end:ident, $content_size:ident, $cross_start:ident, $cross_end:ident, $normal:ident, $pos:ident, $split:ident, $main_calc:ident, $main_calc_reverse:ident) => {
        let ai = $self.flex.align_items;
        if $normal {
            if $self.row {
                while *$start < $end {
                    let (info, temp) = unsafe { $self.rel_vec.get_unchecked_mut(*$start) };
                    *$start += 1;
                    let main = $main_calc(info, $split, &mut $pos);
                    let cross = cross_calc(info, $cross_start, $cross_end, ai);
                    layout_node(
                        $tree,
                        $i_nodes,
                        $rect_style_map,
                        $other_style_map,
                        $layout_map,
                        $notify,
                        $notify_arg,
                        info.id,
                        main,
                        cross,
                        temp,
                        $content_size,
                    );
                }
            } else {
                while *$start < $end {
                    let (info, temp) = unsafe { $self.rel_vec.get_unchecked_mut(*$start) };
                    *$start += 1;
                    let main = $main_calc(info, $split, &mut $pos);
                    let cross = cross_calc(info, $cross_start, $cross_end, ai);
                    layout_node(
                        $tree,
                        $i_nodes,
                        $rect_style_map,
                        $other_style_map,
                        $layout_map,
                        $notify,
                        $notify_arg,
                        info.id,
                        cross,
                        main,
                        temp,
                        $content_size,
                    );
                }
            }
        } else {
            if $self.row {
                while *$start < $end {
                    let (info, temp) = unsafe { $self.rel_vec.get_unchecked_mut(*$start) };
                    *$start += 1;
                    let main = $main_calc_reverse(info, $split, &mut $pos);
                    let cross = cross_calc(info, $cross_start, $cross_end, ai);
                    layout_node(
                        $tree,
                        $i_nodes,
                        $rect_style_map,
                        $other_style_map,
                        $layout_map,
                        $notify,
                        $notify_arg,
                        info.id,
                        main,
                        cross,
                        temp,
                        $content_size,
                    );
                }
            } else {
                while *$start < $end {
                    let (info, temp) = unsafe { $self.rel_vec.get_unchecked_mut(*$start) };
                    *$start += 1;
                    let main = $main_calc_reverse(info, $split, &mut $pos);
                    let cross = cross_calc(info, $cross_start, $cross_end, ai);
                    layout_node(
                        $tree,
                        $i_nodes,
                        $rect_style_map,
                        $other_style_map,
                        $layout_map,
                        $notify,
                        $notify_arg,
                        info.id,
                        cross,
                        main,
                        temp,
                        $content_size,
                    );
                }
            }
        }
    };
}

macro_rules! make_func {
    ($name:ident, $type:ident) => {
        $crate::paste::item! {
            pub(crate) fn $name(&self) -> bool {
                (self.0 & INodeStateType::$type as usize) != 0
            }

            #[allow(dead_code)]
            pub(crate) fn [<$name _true>](&mut self) {
                self.0 |= INodeStateType::$type as usize
            }

            #[allow(dead_code)]
            pub(crate) fn [<$name _false>](&mut self) {
                self.0 &= !(INodeStateType::$type as usize)
            }
            #[allow(dead_code)]
            pub(crate) fn [<$name _set>](&mut self, v: bool) {
                if v {
                    self.0 |= INodeStateType::$type as usize
                }else {
                    self.0 &= !(INodeStateType::$type as usize)
                }
                
            }
        }
    };
}

macro_rules! make_impl {
    ($struct:ident) => {
        impl $struct {
            pub(crate) fn new(s: usize) -> Self {
                INodeState(s)
            }
            make_func!(children_dirty, ChildrenDirty);
            make_func!(self_dirty, SelfDirty);
            make_func!(children_abs, ChildrenAbs);
			make_func!(children_rect, ChildrenRect); // 相对定位大小由自身确定
			make_func!(self_rect, SelfRect);
            make_func!(children_no_align_self, ChildrenNoAlignSelf);
            make_func!(children_index, ChildrenIndex);
			make_func!(vnode, VNode);
			make_func!(rnode, RNode);
            make_func!(abs, Abs);
            make_func!(size_defined, SizeDefined);
			make_func!(line_start_margin_zero, LineStartMarginZero);
			make_func!(breakline, BreakLine);
            pub(crate) fn set_true(&mut self, other: &Self) {
                self.0 |= other.0;
            }
            pub(crate) fn set_false(&mut self, other: &Self) {
                self.0 &= !other.0
            }
        }
    };
}

// 布局计算结果
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutR {
    pub rect: Rect<f32>,
    pub border: Rect<f32>,
    pub padding: Rect<f32>,
}

#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub(crate) struct INodeState(usize);
make_impl!(INodeState);
// struct II {
// 	measure: usize, // 0表示为标准版，1~n表示使用特定的测量函数
// 	state: usize,
// }
// trait Context {
// 	fn get_style(&self, id: usize) -> &Style;
// 	fn get_layout(&self, id: usize) -> &LayoutR;
// 	fn set_layout(&mut self, id: usize, layout: LayoutR);
// 	fn get_children_head();
// 	fn get_children_tail();
// 	fn get_node_next();
// 	fn get_node_prev();
// 	fn get_node(&self, id: usize) -> &INodeState;
// }
// trait Node {
// 	is_children_dirty();
// 	is_self_dirty();
// 	is_abs();
// 	is_abs_rect();
// 	is_size_defined();
// }
//节点状态
pub enum INodeStateType {
    ChildrenDirty = 1,        // 子节点布局需要重新计算
    SelfDirty = 2,            // 自身布局需要重新计算
    ChildrenAbs = 4,          // 子节点是否都是绝对坐标， 如果是，则本节点的自动大小为0.0
    ChildrenNoAlignSelf = 16, // 子节点没有设置align_self
    ChildrenIndex = 32,  // 子节点是否为顺序排序

    VNode = 64, // 是否为虚拟节点, 虚拟节点下只能放叶子节点

    Abs = 128,                  // 是否为绝对坐标
    SizeDefined = 512, // 是否为根据子节点自动计算大小
    LineStartMarginZero = 1024, // 如果该元素为行首，则margin_start为0
	BreakLine = 2048, // 强制换行
	
	RNode = 4096,// 真实节点

	ChildrenRect = 8192, 
	SelfRect = 16384,// 自身区域不受父节点或子节点影响
}
// TODO max min aspect_ratio， RectStyle也可去掉了. 将start end改为left right。 将数据结构统一到标准结构下， 比如Rect Size Point
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharNode {
    pub ch: char,                // 字符
    pub margin_start: f32, // margin
    pub size: (f32, f32),        // 字符大小
    pub pos: (f32, f32),         // 位置
    pub ch_id_or_count: usize,   // 字符id或单词的字符数量
    pub base_width: f32,         // font_size 为32 的字符宽度
	pub char_i: isize,// 字符在整个节点中的索引
	pub context_id: isize, // 如果是多字符文字中的某个字符，则存在一个容易索引
}

impl Default for CharNode {
	fn default() -> Self {
		CharNode {
			ch: char::from(0),
			margin_start: 0.0,
			size: (0.0, 0.0),
			pos: (0.0, 0.0),
			ch_id_or_count: 0,
			base_width: 0.0,
			char_i: -1,
			context_id: -1,
		}
	}
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct INode {
    pub(crate) state: INodeState,
	pub text: Vec<CharNode>, // 文字节点
	pub char_index: usize, // 如果是图文混排，代表在Vec<CharNode>中的位置
	pub scale: f32, // 文字布局的缩放值， 放到其它地方去？TODO
}

impl INode {
    pub fn new(value: INodeStateType, char_index: usize) -> Self {
        INode {
            state: INodeState::new(value as usize + INodeStateType::ChildrenIndex as usize),
			text: Vec::new(),
			char_index: char_index,
			scale: 1.0,
        }
    }
}

impl Default for INode {
    fn default() -> Self {
        INode {
            state: INodeState::new(
                INodeStateType::ChildrenAbs as usize
					+ INodeStateType::ChildrenRect as usize
                    + INodeStateType::ChildrenNoAlignSelf as usize
					+ INodeStateType::ChildrenIndex as usize
					+ INodeStateType::RNode as usize,
            ),
			text: Vec::new(),
			char_index: 0,
			scale: 1.0
        }
    }
}

impl INode {
	pub fn is_vnode(&self) -> bool {
        self.state.vnode()
	}

	// 是否为真实节点
	pub fn is_rnode(&self) -> bool {
        self.state.rnode()
	}
	
    pub fn set_vnode(&mut self, vnode: bool) {
        if vnode {
            self.state.vnode_true();
        } else {
            self.state.vnode_false();
        }
	}
	
	pub fn set_rnode(&mut self, rnode: bool) {
        if rnode {
            self.state.rnode_true();
        } else {
            self.state.rnode_false();
        }
	}
	
    pub fn set_line_start_margin_zero(&mut self, b: bool) {
        if b {
            self.state.line_start_margin_zero_true();
        } else {
            self.state.line_start_margin_zero_false();
        }
	}
	pub fn set_breakline(&mut self, b: bool) {
        if b {
            self.state.breakline_true();
        } else {
            self.state.breakline_false();
        }
    }
}

impl Default for LayoutR {
    fn default() -> LayoutR {
        LayoutR {
            rect: Rect {
                start: 0.0,
                end: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
            border: Rect {
                start: 0.0,
                end: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
            padding: Rect {
                start: 0.0,
                end: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
        }
    }
}

impl LayoutR {
    // 从LayoutR上获得节点的内容区大小
    pub(crate) fn get_content_size(&self) -> (f32, f32) {
        (
            self.rect.end
                - self.rect.start
                - self.border.start
                - self.border.end
                - self.padding.start
                - self.padding.end,
            self.rect.bottom
                - self.rect.top
                - self.border.top
                - self.border.bottom
                - self.padding.top
                - self.padding.bottom,
        )
    }
}
// 计算时使用的临时数据结构
struct Cache {
    // size: Size<Number>,
    size1: (f32, f32), // 最小大小
    main: Number,   // 主轴的大小, 用于约束换行，该值需要参考节点设置的width或height，以及max_width或max_height, 如果都未设置，则该值为无穷大
    cross: Number,  // 交叉轴的大小
    main_line: f32, // 主轴的大小, 用于判断是否折行

    main_value: f32, // 主轴的像素大小，该值需要参考width或height，以及min_width或min_height，用于子节点未将该节点撑得更大时，节点的主轴布局结果
    cross_value: f32, // 交叉轴的像素大小，该值需要参考width或height，以及min_width或min_height，用于子节点未将该节点撑得更大时，节点的交叉轴布局结果

    state: INodeState, // 统计子节点的 ChildrenAbs ChildrenNoAlignSelf ChildrenIndex

    heap: SimpleHeap<OrderSort>,
    temp: Temp, // 缓存的子节点数组
}
#[derive(Clone, PartialEq, PartialOrd, Debug)]
enum TempType {
    None,
    Ok,
    R(Temp),
    CharIndex(usize),
}
impl Default for TempType {
    fn default() -> Self {
        TempType::None
    }
}
// 排序节点
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
struct OrderSort(isize, usize, RelNodeInfo, TempType); // (order, index, Info, temp)
impl Ord for OrderSort {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 > other.0 {
            Ordering::Greater
        }else if self.0 < other.0 {
            Ordering::Less
        }else if self.1 > other.1 {
            Ordering::Greater
        }else if self.1 < other.1 {
            Ordering::Less
        }else{
            Ordering::Equal
        }
    }
}
impl Eq for OrderSort {}

//临时缓存的节点样式、大小和子节点数组
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
struct Temp {
    flex: ContainerStyle,
    row: bool,
    abs_vec: Vec<(usize, usize, usize, INodeState, bool)>, // (id, children_head, children_tail, state, is_text) 绝对定位的子节点数组
    rel_vec: Vec<(RelNodeInfo, TempType)>,                 // 相对定位的子节点数组
    children_percent: bool,                                // 子节点是否有百分比宽高
}
//容器样式
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
pub(crate) struct ContainerStyle {
    pub(crate) flex_direction: FlexDirection,
    pub(crate) flex_wrap: FlexWrap,
    pub(crate) justify_content: JustifyContent,
    pub(crate) align_items: AlignItems,
    pub(crate) align_content: AlignContent,
}
//相对定位下缓存的节点信息
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
struct RelNodeInfo {
    id: usize,
    grow: f32,                    // 节点grow的值
    shrink: f32,                  // 节点shrink的值
    main: f32,                    // 节点主轴尺寸(受basis影响)
    cross: f32,                   // 节点交叉轴尺寸
    margin_main: f32,             // 节点主轴方向 margin_start margin_end的大小
    margin_main_start: Number,    // 节点主轴方向 margin_start的大小
    margin_main_end: Number,      // 节点主轴方向 margin_end的大小
    margin_cross_start: Number,   // 节点交叉轴方向 margin_start的大小
    margin_cross_end: Number,     // 节点交叉轴方向 margin_end的大小
    align_self: AlignSelf,        // 节点的align_self
    main_d: Dimension,            // 节点主轴大小
    cross_d: Dimension,           // 节点交叉轴大小
	line_start_margin_zero: bool, // 如果该元素为行首，则margin_start为0
	breakline: bool, 			// 强制换行
	// min_main: Number,  //主轴最小尺寸
	// max_main: Number, // 主轴最大尺寸
}

// 计算时统计的行信息
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
struct LineInfo {
    main: f32,            // 行内节点主轴尺寸的总值，不受basis影响
    cross: f32,           // 多行子节点交叉轴的像素的累计值
    item: LineItem,       // 当前计算的行
    items: Vec<LineItem>, // 已计算的行
}
//行信息中每行条目
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
struct LineItem {
    count: usize,       // 行内节点总数量
    grow: f32,          // 行内节点grow的总值
    shrink: f32,        // 行内节点shrink的总值
    margin_auto: usize, // 行内节点主轴方向 margin=auto 的数量
    main: f32,          // 行内节点主轴尺寸的总值（包括size margin）
    cross: f32,         // 行内节点交叉轴尺寸的最大值
}

#[derive(PartialEq, Debug)]
enum LayoutResult {
    None,
    Size((f32, f32)),
}

impl LineItem {
    // 将节点信息统计到行条目上
    fn merge(&mut self, info: &RelNodeInfo, line_start: bool) {
        self.count += 1;
        self.grow += info.grow;
        self.shrink += info.shrink;
        self.main += info.main;
        let mut cross = info.cross;
        match info.margin_main_end {
            Number::Defined(r) => self.main += r,
            _ => self.margin_auto += 1,
        }
        match info.margin_cross_start {
            Number::Defined(r) => cross += r,
            _ => (),
        }
        match info.margin_cross_end {
            Number::Defined(r) => cross += r,
            _ => (),
        }
        if self.cross < cross {
            self.cross = cross;
        }
        if line_start && info.line_start_margin_zero {
            return;
        }
        match info.margin_main_start {
            Number::Defined(r) => self.main += r,
            _ => self.margin_auto += 1,
        }
    }
}

impl Cache {
    fn new(flex: ContainerStyle, size: Size<Number>, min_size: Size<Number>, max_width: Number, max_height: Number) -> Self {
        // 计算主轴和交叉轴，及大小
        let row = flex.flex_direction == FlexDirection::Row
            || flex.flex_direction == FlexDirection::RowReverse;
        let (main, cross, max_main, min_main, min_cross) = if row {
            (size.width, size.height, max_width, min_size.width, min_size.height)
        } else {
            (size.height, size.width, max_height, min_size.height, min_size.width)
        };
        let m = if flex.flex_wrap == FlexWrap::NoWrap {
            std::f32::INFINITY
        } else {
			max_calc(main, max_main).or_else(std::f32::INFINITY)
        };
        unsafe { PP += 1 };
        Cache {
            // size,
            size1: (min_size.width.or_else(0.0), min_size.height.or_else(0.0)),
            main,
            cross,
            main_line: m,
            main_value: min_main.or_else(0.0),
            cross_value: min_cross.or_else(0.0),
            state: INodeState::new(
                INodeStateType::ChildrenAbs as usize
					+ INodeStateType::ChildrenRect as usize
                    + INodeStateType::ChildrenNoAlignSelf as usize
                    + INodeStateType::ChildrenIndex as usize,
            ),
            heap: SimpleHeap::new(Ordering::Less),
            temp: Temp::new(flex, row),
        }
    }
    // 自动布局，计算宽高， 如果is_notify则返回Temp(宽度或高度auto、宽度或高度undefined的节点会进入此方法)
    fn auto_layout<T>(
        &mut self,
        tree: &IdTree<u32>,
        i_nodes: &mut impl IndexMut<usize, Output = INode>,
        rect_style_map: &impl Index<usize, Output = RectStyle>,
        other_style_map: &impl Index<usize, Output = OtherStyle>,
        layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
        notify: fn(&mut T, usize, &LayoutR),
        notify_arg: &mut T,
        is_notify: bool,
        id: usize,
        is_text: bool,
        child_head: usize,
        child_tail: usize,
        children_index: bool,
        direction: Direction,
        border: &Rect<Dimension>,
        padding: &Rect<Dimension>,
    ) -> (f32, f32, TempType) {
        debug_println!(
            "{:?}auto_layout1: id:{:?} head:{:?} tail:{:?} is_notify:{:?}",
            ppp(),
            id,
            child_head,
            child_tail,
            is_notify
        );
        self.do_layout(
            tree,
            i_nodes,
            rect_style_map,
            other_style_map,
            layout_map,
            notify,
            notify_arg,
            is_notify,
            id,
            is_text,
            child_head,
            child_tail,
            children_index,
            direction,
        );
        debug_println!(
            "{:?}auto_layout2: id:{:?}, size:{:?}",
            ppp(),
            id,
            (self.main_value, self.cross_value)
        );
        let (w, h) = self.temp.main_cross(self.main_value, self.cross_value);
        (
            calc_size_from_content(w, border.start, border.end, padding.start, padding.end),
            calc_size_from_content(h, border.top, border.bottom, padding.top, padding.bottom),
            if is_notify {
                TempType::Ok
            } else {
                // 则将布局的中间数组暂存下来
                TempType::R(replace(&mut self.temp, Temp::default()))
            },
        )
    }
    fn do_layout<T>(
        &mut self,
        tree: &IdTree<u32>,
        i_nodes: &mut impl IndexMut<usize, Output = INode>,
        rect_style_map: &impl Index<usize, Output = RectStyle>,
        other_style_map: &impl Index<usize, Output = OtherStyle>,
        layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
        notify: fn(&mut T, usize, &LayoutR),
        notify_arg: &mut T,
        is_notify: bool,
        id: usize,
        is_text: bool,
        child_head: usize,
        child_tail: usize,
        children_index: bool,
        direction: Direction,
    ) {
        let mut line = LineInfo::default();
        debug_println!(
            "{:?}do layout1, id:{:?} is_notify:{:?}",
            ppp(),
            id,
            is_notify
        );
        if is_text {
            let i_node = &mut i_nodes[id];
            self.text_layout(id, &mut i_node.text, &mut line, 0);
        } else {
            self.node_layout(
                tree,
                i_nodes,
                rect_style_map,
                other_style_map,
                layout_map,
                notify,
                notify_arg,
                is_notify && self.main.is_defined() && self.cross.is_defined(),
                &mut line,
                if direction != Direction::RTL {
                    child_head
                } else {
                    child_tail
                },
                children_index,
                direction,
            );
		}
		line.cross += line.item.cross;

        debug_println!(
            "{:?}do layout2, id:{:?} line:{:?}, vec:{:?}",
            ppp(),
            id,
            &line,
            &self.temp.rel_vec
        );
        if children_index { // 从堆中添加到数组上
            while let Some(OrderSort(_, _, info, temp)) = self.heap.pop() {
                self.temp.rel_vec.push((info, temp))
            }
        }
        // 如果自动大小， 则计算实际大小
        if !self.main.is_defined() {
            self.main_value = f32::max(line.main, self.main_value);
        }
        if !self.cross.is_defined() {
			// self.cross1 = line.cross + line.item.cross; ？？？
			self.cross_value = f32::max(line.cross, self.cross_value);
        }
        // 记录节点的子节点的统计信息
        // let node = &tree[id];
        let i_node = &mut i_nodes[id];
        i_node.state.set_false(&INodeState::new(
            INodeStateType::ChildrenAbs as usize
				+ INodeStateType::ChildrenRect as usize
                + INodeStateType::ChildrenNoAlignSelf as usize
                + INodeStateType::ChildrenIndex as usize,
        ));
        i_node.state.set_true(&self.state);
        // 根据is_notify决定是否继续计算
        if is_notify {
            self.temp.layout(
                tree,
                i_nodes,
                rect_style_map,
                other_style_map,
                layout_map,
                notify,
                notify_arg,
                self.temp.main_cross(self.main_value, self.cross_value),
                self.main_value,
                self.cross_value,
                &line,
            );
        }
    }
    // 文字的flex布局
    fn text_layout(
        &mut self,
        id: usize,
        text: &mut Vec<CharNode>,
        line: &mut LineInfo,
        mut char_index: usize,
    ) {
		debug_println!("text_layout, id:{}", id);
        let len = text.len();
        while char_index < len {
            let r = &text[char_index];
            let (main_d, cross_d) = self.temp.main_cross(r.size.0, r.size.1);
            let margin = self.temp.main_cross(
                (Dimension::Points(r.margin_start), Dimension::Points(0.0)),
                (Dimension::Points(0.0), Dimension::Points(0.0)),
            );
            let mut info = RelNodeInfo {
                id,
                grow: 0.0,
                shrink: 0.0,
                main: main_d,
                cross: cross_d,
                margin_main: 0.0,
                margin_main_start: calc_number((margin.0).0, self.main_value),
                margin_main_end: calc_number((margin.0).1, self.main_value),
                margin_cross_start: calc_number((margin.1).0, self.cross_value),
                margin_cross_end: calc_number((margin.1).1, self.cross_value),
                align_self: AlignSelf::Auto,
                main_d: Dimension::Points(main_d),
                cross_d: Dimension::Points(cross_d),
				line_start_margin_zero: true,
				breakline: r.ch == char::from('\n'),
				// min_main: Number::Undefined,
				// max_main: Number::Undefined,
            };
            let start = info.margin_main_start.or_else(0.0);
            let end = info.margin_main_end.or_else(0.0);
            // 主轴auto时记录子节点实际大
            let line_start = if line.item.count == 0 {
                // 处理行首
                0.0
            } else {
                start
            };
            info.margin_main = start + end;
            line.main += info.main + line_start + end;
			self.add_vec(line, 0, info, TempType::CharIndex(char_index));
            // 判断是否为单词容器
            if r.ch == char::from(0) {
                char_index += r.ch_id_or_count;
            } else {
                char_index += 1;
            }
		}
    }
    // 节点的flex布局
    fn node_layout<T>(
        &mut self,
        tree: &IdTree<u32>,
        i_nodes: &mut impl IndexMut<usize, Output = INode>,
        rect_style_map: &impl Index<usize, Output = RectStyle>,
        other_style_map: &impl Index<usize, Output = OtherStyle>,
        layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
        notify: fn(&mut T, usize, &LayoutR),
        notify_arg: &mut T,
        is_notify: bool,
        line: &mut LineInfo,
        mut child: usize,
        children_index: bool,
        direction: Direction,
    ) {
        while child > 0 {
            let n = &tree[child];
            let i_node = &mut i_nodes[child];
            if i_node.state.abs() {
                if i_node.state.self_rect() {
                    // 绝对区域不需计算
                    child = node_iter(direction, n);
                    continue;
                }
                self.state.children_rect_false();
                let id = child;
                child = node_iter(direction, n);
                let child_head = n.children().head;
                let child_tail = n.children().tail;
                let state = i_node.state;
                i_node.state.set_false(&INodeState::new(
                    INodeStateType::ChildrenDirty as usize + INodeStateType::SelfDirty as usize,
                ));
                let is_text = i_node.text.len() > 0;
                if is_notify {
                    abs_layout(
                        tree,
                        i_nodes,
                        rect_style_map,
                        other_style_map,
                        layout_map,
                        notify,
                        notify_arg,
                        id,
                        is_text,
                        child_head,
                        child_tail,
                        state,
                        self.size1,
                        &self.temp.flex,
                    );
                } else {
                    self.temp
                        .abs_vec
                        .push((id, child_head, child_tail, state, is_text));
                }
                continue;
			}
            let style = &other_style_map[child];
            let rect_style = &rect_style_map[child];
            if style.display == Display::None {
                child = node_iter(direction, n);
                continue;
			}
			if !i_node.state.self_rect() {
				self.state.children_rect_false();
			}
            self.state.children_abs_false();
            let id = child;
            child = node_iter(direction, n);
            let vnode = i_node.state.vnode();
            if vnode {
                // 如果是虚拟节点， 则遍历其子节点， 加入到列表中
                let node = unsafe { tree.get_unchecked(id) };
                let child = if direction != Direction::RTL {
                    node.children().head
                } else {
                    node.children().tail
                };
                self.node_layout(
                    tree,
                    i_nodes,
                    rect_style_map,
                    other_style_map,
                    layout_map,
                    notify,
                    notify_arg,
                    is_notify,
                    line,
                    child,
                    children_index,
                    direction,
				);
				// if is_notify {
					notify(notify_arg, id, &layout_map[id]);
				// }
                continue;
            }
            let order = style.order;
            if order != 0 {
                self.state.children_index_false();
            }
            if style.align_self != AlignSelf::Auto {
                self.state.children_no_align_self_false();
            }
            // flex布局时， 如果子节点的宽高未定义，则根据子节点的布局进行计算。如果子节点的宽高为百分比，并且父节点对应宽高未定义，则为0
            let w = calc_number(rect_style.size.width, self.size1.0);
			let h = calc_number(rect_style.size.height, self.size1.1);
			debug_println!("id: {}, parent_size:{:?}", id, self.size1);
            let basis = style.flex_basis;
            let (main_d, cross_d) = self
                .temp
				.main_cross(rect_style.size.width, rect_style.size.height);
			
			let (min_width, max_width, min_height, max_height) = (
				calc_number(style.min_size.width, self.main_value),
				calc_number(style.max_size.width, self.main_value),
				calc_number(style.min_size.height, self.cross_value),
				calc_number(style.max_size.height, self.cross_value),
			);
			let (max_main, max_cross) = self
                .temp
				.main_cross(max_width, max_height);
			let (min_main, min_cross) = self
                .temp
				.main_cross(min_width, min_height);
            let margin = self.temp.main_cross(
                (rect_style.margin.start, rect_style.margin.end),
                (rect_style.margin.top, rect_style.margin.bottom),
			);
			debug_println!("main1,id:{}, main1:{:?}, main_d: {:?}, rect_style: {:?}, min_main: {:?}, max_main: {:?}", id, self.main_value, main_d, rect_style, min_main, max_main);
            let mut info = RelNodeInfo {
                id,
                grow: style.flex_grow,
                shrink: style.flex_shrink,
                main: min_max_calc(main_d.resolve_value(self.main_value), min_main, max_main),
				cross: min_max_calc(cross_d.resolve_value(self.cross_value),min_cross,max_cross),
                margin_main: 0.0,
                margin_main_start: calc_number((margin.0).0, self.main_value),
                margin_main_end: calc_number((margin.0).1, self.main_value),
                margin_cross_start: calc_number((margin.1).0, self.cross_value),
                margin_cross_end: calc_number((margin.1).1, self.cross_value),
                align_self: style.align_self,
                main_d: main_d,
                cross_d: cross_d,
				line_start_margin_zero: i_node.state.line_start_margin_zero(),
				breakline: i_node.state.breakline(),
				// min_main: min_main,
				// max_main: max_main,
				
            };
            let temp = if w == Number::Undefined || h == Number::Undefined {
                // 需要计算子节点大小
                let flex = ContainerStyle::new(style);
                let direction = style.direction;
                let border = style.border.clone();
                let padding = style.padding.clone();
                // 子节点大小是否不会改变， 如果不改变则直接布局
                let mut fix = true;
                // 主轴有3种情况后面可能会被改变大小
                if main_d.is_undefined() {
                    fix =
                        basis.is_undefined() && style.flex_grow == 0.0 && style.flex_shrink == 0.0;
                }
                //  交叉轴有2种情况后面可能会被改变大小
                if fix && cross_d.is_undefined() {
                    fix = style.align_self != AlignSelf::Stretch
                        && style.align_items != AlignItems::Stretch;
                }
                debug_println!(
                    "{:?}calc size: id:{:?} fix:{:?} size:{:?} next:{:?}",
                    ppp(),
                    id,
                    fix,
                    (w, h),
                    child
                );
                let child_head = n.children().head;
                let child_tail = n.children().tail;
                let n_children_index = i_node.state.children_index();
                let is_text = i_node.text.len() > 0;
                let w = calc_content_size(w, border.start, border.end, padding.start, padding.end);
                let h =
                    calc_content_size(h, border.top, border.bottom, padding.top, padding.bottom);
                let mut cache = Cache::new(
                    flex,
                    Size {
                        width: w,
                        height: h,
					},Size {
                        width: calc_length(w, min_width),
                        height: calc_length(h, min_height),
					},
					calc_content_size(max_width, border.start, border.end, padding.start, padding.end),
					calc_content_size(max_height, border.top, border.bottom, padding.top, padding.bottom)
				);
					debug_println!("cache, main_line: {:?}, id: {}", cache.main_line, id);
				// cache.main_line = 
				// max_calc(w, max_width);
				// max_calc(h, max_height);
                let (ww, hh, r) = cache.auto_layout(
                    tree,
                    i_nodes,
                    rect_style_map,
                    other_style_map,
                    layout_map,
                    notify,
                    notify_arg,
                    fix,
                    id,
                    is_text,
                    child_head,
                    child_tail,
                    n_children_index,
                    direction,
                    &border,
                    &padding,
                );
                let mc = self.temp.main_cross(ww, hh);
                info.main = min_max_calc(mc.0, min_main, max_main);
                info.cross = min_max_calc(mc.1, min_cross, max_cross);
                r
            } else {
                // 确定大小的节点， TempType为None
                // debug_println!("static size: id:{:?} size:{:?} next:{:?}", id, (w, h), child);
                TempType::None
            };
            let start = info.margin_main_start.or_else(0.0);
            let end = info.margin_main_end.or_else(0.0);
            // 主轴auto时记录子节点实际大
            let line_start = if line.item.count == 0 && info.line_start_margin_zero {
                // 处理行首
                0.0
            } else {
                start
            };
            info.margin_main = start + end;
            line.main += info.main + line_start + end;
            match basis {
                // 如果有basis, 则修正main
                Dimension::Points(r) => {
                    info.main = r;
                    info.main_d = basis;
                }
                Dimension::Percent(r) => {
                    info.main = self.main_value * r;
                    info.main_d = basis;
                }
                _ => (),
            };
            match info.main_d {
                Dimension::Percent(_r) => self.temp.children_percent = true,
                _ => match info.cross_d {
                    Dimension::Percent(_r) => self.temp.children_percent = true,
                    _ => (),
                },
            };
            // 设置shrink的大小
            info.shrink *= info.main;
            if children_index {
                // 如果需要排序，调用不同的添加方法
                self.add_vec(line, order, info, temp);
            } else {
                self.add_heap(line, order, info, temp);
            };
		}
    }
    // 添加到数组中，计算当前行的grow shrink 是否折行及折几行
    fn add_vec(&mut self, line: &mut LineInfo, _order: isize, info: RelNodeInfo, temp: TempType) {
        // debug_println!("add info:{:?}", info);
        line.add(self.main_line, &info);
        self.temp.rel_vec.push((info, temp));
    }
    // 添加到堆中
    fn add_heap(
        &mut self,
        line: &mut LineInfo,
        order: isize,
        info: RelNodeInfo,
        temp: TempType,
    ) {
        line.add(self.main_line, &info);
        self.heap.push(OrderSort(order, self.heap.len(), info, temp));
    }
}

impl Temp {
    fn new(flex: ContainerStyle, row: bool) -> Self {
        Temp {
            flex,
            row,
            abs_vec: Vec::new(),
            rel_vec: Vec::new(),
            children_percent: false,
        }
    }
    fn main_cross<T>(&self, w: T, h: T) -> (T, T) {
        if self.row {
            (w, h)
        } else {
            (h, w)
        }
    }

    // 用缓存的相对定位的子节点数组重建行信息
    fn reline(&mut self, main: f32, cross: f32) -> LineInfo {
        let mut line = LineInfo::default();
        if self.children_percent {
            for r in self.rel_vec.iter_mut() {
                // 修正百分比的大小
                match r.0.main_d {
                    Dimension::Percent(rr) => {
                        r.0.main = main * rr;
                    }
                    _ => (),
                }
                // 修正百分比的大小
                match r.0.cross_d {
                    Dimension::Percent(rr) => {
                        r.0.cross = cross * rr;
                    }
                    _ => (),
                }
                line.add(main, &r.0);
            }
        } else {
            for r in self.rel_vec.iter() {
                line.add(main, &r.0);
            }
        }
        unsafe { PP += 1 };
        debug_println!("{:?}reline: line:{:?}", ppp(), &line);
        line
    }
    // 实际进行子节点布局
    fn layout<T>(
        &mut self,
        tree: &IdTree<u32>,
        i_nodes: &mut impl IndexMut<usize, Output = INode>,
        rect_style_map: &impl Index<usize, Output = RectStyle>,
        other_style_map: &impl Index<usize, Output = OtherStyle>,
        layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
        notify: fn(&mut T, usize, &LayoutR),
        notify_arg: &mut T,
        size: (f32, f32),
        main: f32,
        cross: f32,
        line: &LineInfo,
    ) {
        debug_println!(
            "{:?}layout: style:{:?} size:{:?} main_cross:{:?}",
            ppp(),
            self.flex,
            size,
            (main, cross)
        );
        // 处理abs_vec
        for e in self.abs_vec.iter() {
            abs_layout(
                tree,
                i_nodes,
                rect_style_map,
                other_style_map,
                layout_map,
                notify,
                notify_arg,
                e.0,
                e.4,
                e.1,
                e.2,
                e.3,
                size,
                &self.flex,
            );
        }
        let normal = self.flex.flex_direction == FlexDirection::Row
            || self.flex.flex_direction == FlexDirection::Column;
        let mut start = 0;
        // 根据行列信息，对每个节点布局
        if line.items.len() == 0 {
            // 单行处理
            self.single_line(
                tree,
                i_nodes,
                rect_style_map,
                other_style_map,
                layout_map,
                notify,
                notify_arg,
                main,
                &line.item,
                &mut start,
                self.rel_vec.len(),
                size,
                0.0,
                cross,
                normal,
            );
            return;
        }

        // 多行布局，计算开始位置和分隔值
        let (mut pos, split) = match self.flex.align_content {
            AlignContent::FlexStart => {
                if self.flex.flex_wrap != FlexWrap::WrapReverse {
                    (0.0, 0.0)
                } else {
                    (cross, 0.0)
                }
            }
            AlignContent::FlexEnd => {
                if self.flex.flex_wrap != FlexWrap::WrapReverse {
                    (cross - line.cross, 0.0)
                } else {
                    (line.cross, 0.0)
                }
            }
            AlignContent::Center => {
                if self.flex.flex_wrap != FlexWrap::WrapReverse {
                    ((cross - line.cross) / 2.0, 0.0)
                } else {
                    ((cross + line.cross) / 2.0, 0.0)
                }
            }
            AlignContent::SpaceBetween => {	
                if self.flex.flex_wrap != FlexWrap::WrapReverse {
                    if line.items.len() > 0 {
                        (0.0, (cross - line.cross) / line.items.len() as f32)
                    } else {
                        ((cross - line.cross) / 2.0, 0.0)
                    }
                } else {
                    if line.items.len() > 0 {
                        (cross, (cross - line.cross) / line.items.len() as f32)
                    } else {
                        ((cross + line.cross) / 2.0, 0.0)
                    }
                }
            }
            AlignContent::SpaceAround => {
                let s = (cross - line.cross) / (line.items.len() + 1) as f32;
                if self.flex.flex_wrap != FlexWrap::WrapReverse {
                    (s / 2.0, s)
                } else {
                    (cross - s / 2.0, s)
                }
            }
            _ => {
                if line.cross - cross > EPSILON {
                    if self.flex.flex_wrap != FlexWrap::WrapReverse {
                        (0.0, 0.0)
                    } else {
                        (cross, 0.0)
                    }
                } else {
                    // 伸展， 平分交叉轴
                    let mut pos = if self.flex.flex_wrap != FlexWrap::WrapReverse {
                        0.0
                    } else {
                        cross
                    };
                    let cross = cross / (line.items.len() + 1) as f32;
                    for item in line.items.iter() {
                        let (cross_start, cross_end) = self.multi_calc(cross, 0.0, &mut pos);
                        self.single_line(
                            tree,
                            i_nodes,
                            rect_style_map,
                            other_style_map,
                            layout_map,
                            notify,
                            notify_arg,
                            main,
                            &item,
                            &mut start,
                            item.count,
                            size,
                            cross_start,
                            cross_end,
                            normal,
                        );
                    }
                    let (cross_start, cross_end) = self.multi_calc(cross, 0.0, &mut pos);
                    self.single_line(
                        tree,
                        i_nodes,
                        rect_style_map,
                        other_style_map,
                        layout_map,
                        notify,
                        notify_arg,
                        main,
                        &line.item,
                        &mut start,
                        line.item.count,
                        size,
                        cross_start,
                        cross_end,
                        normal,
                    );
                    return;
                }
            }
        };
        for item in line.items.iter() {
			debug_println!("single_line!!, item: {:?}, split: {:?}, pos: {:?}", item, split, pos);
            let (cross_start, cross_end) = self.multi_calc(item.cross, split, &mut pos);
            self.single_line(
                tree,
                i_nodes,
                rect_style_map,
                other_style_map,
                layout_map,
                notify,
                notify_arg,
                main,
                &item,
                &mut start,
                item.count,
                size,
                cross_start,
                cross_end,
                normal,
            );
		}
		debug_println!("single_line!!, item: {:?}, split: {:?}, pos: {:?}, cross:{:?}", line.item, split, pos, line.cross);
        let (cross_start, cross_end) = self.multi_calc(line.item.cross, split, &mut pos);
        self.single_line(
            tree,
            i_nodes,
            rect_style_map,
            other_style_map,
            layout_map,
            notify,
            notify_arg,
            main,
            &line.item,
            &mut start,
            line.item.count,
            size,
            cross_start,
            cross_end,
            normal,
        );
    }
    // 多行的区间计算
    fn multi_calc(&self, cross: f32, split: f32, pos: &mut f32) -> (f32, f32) {
        let start = *pos;
        if self.flex.flex_wrap != FlexWrap::WrapReverse {
            let end = *pos + cross;
            *pos = end + split;
            (start, end)
        } else {
            let end = *pos - cross;
            *pos = end - split;
            (end, start)
        }
    }

    // 处理单行的节点布局
    fn single_line<T>(
        &mut self,
        tree: &IdTree<u32>,
        i_nodes: &mut impl IndexMut<usize, Output = INode>,
        rect_style_map: &impl Index<usize, Output = RectStyle>,
        other_style_map: &impl Index<usize, Output = OtherStyle>,
        layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
        notify: fn(&mut T, usize, &LayoutR),
        notify_arg: &mut T,
        main: f32,
        item: &LineItem,
        start: &mut usize,
        count: usize,
        content_size: (f32, f32),
        cross_start: f32,
        cross_end: f32,
        normal: bool,
    ) {
        debug_println!(
            "{:?}single_line: normal:{:?} content_size:{:?}, cross:{:?} start_end:{:?} main:{:?}",
            ppp(),
            normal,
            content_size,
            (cross_start, cross_end),
            (*start, count),
            (main, item.main),
        );
        let first = unsafe { self.rel_vec.get_unchecked_mut(*start) };
        if first.0.line_start_margin_zero {
            // 修正行首的margin
            first.0.margin_main_start = Number::Defined(0.0);
        }
        let end = *start + count;
        let mut pos = if normal { 0.0 } else { main };
        // 浮点误差计算
        if main - item.main > EPSILON {
            // 表示需要放大
            if item.grow > 0.0 {
                // grow 填充
                let split = (main - item.main) / item.grow;
                item_calc!(
                    self,
                    tree,
                    i_nodes,
                    rect_style_map,
                    other_style_map,
                    layout_map,
                    notify,
                    notify_arg,
                    start,
                    end,
                    content_size,
                    cross_start,
                    cross_end,
                    normal,
                    pos,
                    split,
                    grow_calc,
                    grow_calc_reverse
                );
                return;
            } else if item.margin_auto > 0 {
                // margin_auto 填充
                let split = (main - item.main) / item.margin_auto as f32;
                item_calc!(
                    self,
                    tree,
                    i_nodes,
                    rect_style_map,
                    other_style_map,
                    layout_map,
                    notify,
                    notify_arg,
                    start,
                    end,
                    content_size,
                    cross_start,
                    cross_end,
                    normal,
                    pos,
                    split,
                    margin_calc,
                    margin_calc_reverse
                );
                return;
            }
        } else if EPSILON < item.main - main {
            if item.shrink > 0.0 {
                // 表示需要收缩
                let split = (item.main - main) / item.shrink;
                item_calc!(
                    self,
                    tree,
                    i_nodes,
                    rect_style_map,
                    other_style_map,
                    layout_map,
                    notify,
                    notify_arg,
                    start,
                    end,
                    content_size,
                    cross_start,
                    cross_end,
                    normal,
                    pos,
                    split,
                    shrink_calc,
                    shrink_calc_reverse
                );
                return;
            }
        }
        let (mut pos, split) = match self.flex.justify_content {
            JustifyContent::FlexStart => {
                if normal {
                    (0.0, 0.0)
                } else {
                    (main, 0.0)
                }
            }
            JustifyContent::FlexEnd => {
                if normal {
                    (main - item.main, 0.0)
                } else {
                    (item.main, 0.0)
                }
            }
            JustifyContent::Center => {
                if normal {
                    ((main - item.main) / 2.0, 0.0)
                } else {
                    ((main + item.main) / 2.0, 0.0)
                }
            }
            JustifyContent::SpaceBetween => {	
                if normal {
                    if item.count > 1 {
                        (0.0, (main - item.main) / (item.count - 1) as f32)
                    } else {
                        ((main - item.main) / 2.0, 0.0)
                    }
                } else {
                    if item.count > 1 {
                        (main, (main - item.main) / (item.count - 1) as f32)
                    } else {
                        ((main - item.main) / 2.0, 0.0)
                    }
                }
            }
            JustifyContent::SpaceAround => {
                let s = (main - item.main) / item.count as f32;
                if normal {
                    (s / 2.0, s)
                } else {
                    (main - s / 2.0, s)
                }
            }
            _ => {
                let s = (main - item.main) / (item.count + 1) as f32;
                if normal {
                    (s, s)
                } else {
                    (main - s, s)
                }
            }
        };
        debug_println!("{:?}main calc: pos:{:?} split:{:?}", ppp(), pos, split);
        item_calc!(
            self,
            tree,
            i_nodes,
            rect_style_map,
            other_style_map,
            layout_map,
            notify,
            notify_arg,
            start,
            end,
            content_size,
            cross_start,
            cross_end,
            normal,
            pos,
            split,
            main_calc,
            main_calc_reverse
        );
    }
}

impl LineInfo {
    // 添加到数组中，计算当前行的grow shrink 是否折行及折几行
    fn add(&mut self, main: f32, info: &RelNodeInfo) {
		debug_println!("add, main: {:?}, {:?}, self.item: {:?}", main, info, self.item);
        // 浮点误差判断是否折行
        if (self.item.count > 0 && self.item.main + info.main + info.margin_main - main > EPSILON) || info.breakline {
			self.cross += self.item.cross;
			debug_println!("breakline, self.cross:{:?}, self.item.cross: {:?}", self.cross, self.item.cross);
            let t = replace(&mut self.item, LineItem::default());
            self.items.push(t);
            self.item.merge(info, true);
        } else {
            self.item.merge(info, self.item.count == 0);
        }
    }
}
impl ContainerStyle {
    pub(crate) fn new(s: &OtherStyle) -> Self {
        ContainerStyle {
            flex_direction: s.flex_direction,
            flex_wrap: s.flex_wrap,
            justify_content: s.justify_content,
            align_items: s.align_items,
            align_content: s.align_content,
        }
    }
}

// 绝对定位下的布局，如果size=auto， 会先调用子节点的布局
pub(crate) fn abs_layout<T>(
    tree: &IdTree<u32>,
    i_nodes: &mut impl IndexMut<usize, Output = INode>,
    rect_style_map: &impl Index<usize, Output = RectStyle>,
    other_style_map: &impl Index<usize, Output = OtherStyle>,
    layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
    notify: fn(&mut T, usize, &LayoutR),
    notify_arg: &mut T,
    id: usize,
    is_text: bool,
    child_head: usize,
    child_tail: usize,
    state: INodeState,
    parent_size: (f32, f32),
    flex: &ContainerStyle,
) {
    let style = &other_style_map[id];
    let rect_style = &rect_style_map[id];
    if style.display == Display::None {
        return;
    }
    let a1 = match flex.justify_content {
        JustifyContent::Center => 0,
        JustifyContent::FlexEnd => 1,
        _ => -1,
    };
    let a2 = match flex.align_items {
        AlignItems::Center => 0,
        AlignItems::FlexEnd => 1,
        _ => -1,
    };
    let (walign, halign) = if flex.flex_direction == FlexDirection::Row
        || flex.flex_direction == FlexDirection::RowReverse
    {
        (a1, a2)
    } else {
        (a2, a1)
	};

	debug_println!("abs_layout, id:{} size:{:?} position:{:?}", id, rect_style.size, style.position);
    let mut w = calc_rect(
        style.position.start,
        style.position.end,
        rect_style.size.width,
        rect_style.margin.start,
        rect_style.margin.end,
        parent_size.0,
        state.children_abs(),
        walign,
	);
    let mut h = calc_rect(
        style.position.top,
        style.position.bottom,
        rect_style.size.height,
        rect_style.margin.top,
        rect_style.margin.bottom,
        parent_size.1,
        state.children_abs(),
        halign,
	);
	let (min_width, max_width, min_height, max_height) = (
		calc_number( style.min_size.width, parent_size.0),
		calc_number(style.max_size.width, parent_size.0),
		calc_number(style.min_size.height, parent_size.1),
		calc_number(style.max_size.height, parent_size.1),
	);
	debug_println!("abs_layout11, id:{} w:{:?}, h:{:?}", id, w, h);
    if w.0 == Number::Undefined || h.0 == Number::Undefined {
        // 根据子节点计算大小
        let direction = style.direction;
        let pos = style.position.clone();
        let margin = rect_style.margin.clone();
        let border = style.border.clone();
        let padding = style.padding.clone();
        let ww = calc_content_size(w.0, border.start, border.end, padding.start, padding.end);
		let hh = calc_content_size(h.0, border.top, border.bottom, padding.top, padding.bottom);
		let flex = ContainerStyle::new(style);
        let mut cache = Cache::new(
            flex.clone(),
            Size {
                width: ww,
                height: hh,
			},Size {
                width: calc_length(ww, min_width),
                height: calc_length(hh, min_height),
			},
			calc_content_size(max_width, border.start, border.end, padding.start, padding.end),
			calc_content_size(max_height, border.top, border.bottom, padding.top, padding.bottom)
		);

        let (ww, hh, _r) = cache.auto_layout(
            tree,
            i_nodes,
            rect_style_map,
            other_style_map,
            layout_map,
            notify,
            notify_arg,
            true,
            id,
            is_text,
            child_head,
            child_tail,
            state.children_index(),
            direction,
            &border,
            &padding,
		);
		debug_println!("calc_rect: id: {}, hh:{:?}", id, hh);
        // 再次计算区域
        w = calc_rect(
            pos.start,
            pos.end,
            Dimension::Points(ww),
            margin.start,
            margin.end,
            parent_size.0,
            false,
            walign,
        );
        h = calc_rect(
            pos.top,
            pos.bottom,
            Dimension::Points(hh),
            margin.top,
            margin.bottom,
            parent_size.1,
            false,
            halign,
		);
		
        let layout = &mut layout_map[id];
        // 设置布局的值
        set_layout_result(
            layout,
            notify,
            notify_arg,
            id,
            (w.1, h.1),
            (
				min_max_calc(w.0.or_else(0.0), min_width, max_width),
				min_max_calc(h.0.or_else(0.0), min_height, max_height)
			),
            &border,
            &padding,
        );
    } else {
        let flex = ContainerStyle::new(style);
        let direction = style.direction;
        let border = style.border.clone();
		let padding = style.padding.clone();
        set_layout(
            tree,
            i_nodes,
            rect_style_map,
            other_style_map,
            layout_map,
            notify,
            notify_arg,
            id,
            is_text,
            child_head,
            child_tail,
            flex,
            direction,
            border,
            padding,
            state,
            (w.1, h.1),
            (
				min_max_calc(w.0.or_else(0.0), min_width, max_width),
				min_max_calc(h.0.or_else(0.0), min_height, max_height)
			),
        );
    };
}

// 如果节点是相对定位，被设脏表示其修改的数据不会影响父节点的布局 则先检查自身的布局数据，然后修改子节点的布局数据
pub(crate) fn rel_layout<T>(
    tree: &IdTree<u32>,
    i_nodes: &mut impl IndexMut<usize, Output = INode>,
    rect_style_map: &impl Index<usize, Output = RectStyle>,
    other_style_map: &impl Index<usize, Output = OtherStyle>,
    layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
    notify: fn(&mut T, usize, &LayoutR),
    notify_arg: &mut T,
    id: usize,
    is_text: bool,
    child_head: usize,
    child_tail: usize,
    state: INodeState,
) {
    let style = &other_style_map[id];
    if style.display == Display::None {
        return;
    }
    let flex = ContainerStyle::new(style);
    let direction = style.direction;
    let border = style.border.clone();
    let padding = style.padding.clone();
    let rect = layout_map[id].rect;
    set_layout(
        tree,
        i_nodes,
        rect_style_map,
        other_style_map,
        layout_map,
        notify,
        notify_arg,
        id,
        is_text,
        child_head,
        child_tail,
        flex,
        direction,
        border,
        padding,
        state,
        (rect.start, rect.top),
        (rect.end - rect.start, rect.bottom - rect.top),
    );
}

// 设置节点的布局数据，如果内容宽高有改变，则调用自身的子节点布局方法
fn set_layout<T>(
    tree: &IdTree<u32>,
    i_nodes: &mut impl IndexMut<usize, Output = INode>,
    rect_style_map: &impl Index<usize, Output = RectStyle>,
    other_style_map: &impl Index<usize, Output = OtherStyle>,
    layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
    notify: fn(&mut T, usize, &LayoutR),
    notify_arg: &mut T,
    id: usize,
    is_text: bool,
    child_head: usize,
    child_tail: usize,
    flex: ContainerStyle,
    direction: Direction,
    border: Rect<Dimension>,
    padding: Rect<Dimension>,
    state: INodeState,
    pos: (f32, f32),
    size: (f32, f32),
) {
    debug_println!(
        "{:?}set_layout: pos:{:?} size:{:?} id:{:?} head:{:?} tail:{:?} children_dirty:{} self_dirty:{} children_rect:{} children_abs:{}",
        ppp(),
        pos,
        size,
        id,
        child_head,
        child_tail,
		state.children_dirty(),
		state.self_dirty(),
		state.children_rect(),
		state.children_abs()
    );
    // 设置布局的值
    let layout = &mut layout_map[id];
    let r = if state.self_dirty()
        || layout.rect.start != pos.0
        || layout.rect.top != pos.1
        || layout.rect.end - layout.rect.start != size.0
        || layout.rect.bottom - layout.rect.top != size.1
    {
        set_layout_result(layout, notify, notify_arg, id, pos, size, &border, &padding)
    } else {
        LayoutResult::None
    };
    // 递归布局子节点
    let rr = if state.children_dirty() {
        layout.get_content_size()
    } else {
        match r {
            LayoutResult::Size(rr) => {
                if state.children_rect()
                    && (state.children_abs()
                        || (state.children_no_align_self()
                            && (flex.flex_direction == FlexDirection::Row
                                || flex.flex_direction == FlexDirection::Column)
                            && flex.flex_wrap == FlexWrap::NoWrap
                            && flex.justify_content == JustifyContent::FlexStart
                            && flex.align_items == AlignItems::FlexStart))
                {
                    // 节点的宽高变化不影响子节点的布局，还可进一步优化仅交叉轴大小变化
                    return;
                }
                rr
            }
            _ => return,
        }
    };
    // 宽高变动重新布局
    let mut cache = Cache::new(
        flex,
        Size {
            width: Number::Defined(rr.0),
            height: Number::Defined(rr.1),
		},Size {
            width: Number::Defined(rr.0),
            height: Number::Defined(rr.1),
		},
		Number::Undefined,
		Number::Undefined,
    );
    cache.do_layout(
        tree,
        i_nodes,
        rect_style_map,
        other_style_map,
        layout_map,
        notify,
        notify_arg,
        true,
        id,
        is_text,
        child_head,
        child_tail,
        state.children_index(),
        direction,
    );
}

// 设置布局结果
fn set_layout_result<T>(
    layout: &mut LayoutR,
    notify: fn(&mut T, usize, &LayoutR),
    notify_arg: &mut T,
    id: usize,
    pos: (f32, f32),
    size: (f32, f32),
    border: &Rect<Dimension>,
    padding: &Rect<Dimension>,
) -> LayoutResult {
    unsafe {
        PC += 1;
        PP = 0
    };
    let old_rect = layout.rect.clone();
    let old_w = old_rect.end
        - layout.border.end
        - layout.padding.end
        - (old_rect.start + layout.border.start + layout.padding.start);
    let old_h = old_rect.bottom
        - layout.border.bottom
        - layout.padding.bottom
        - (old_rect.top + layout.border.top + layout.padding.top);
    layout.rect.start = pos.0;
    layout.rect.top = pos.1;
    layout.rect.end = pos.0 + size.0;
	layout.rect.bottom = pos.1 + size.1;
    calc_border_padding(border, size.0, size.1, &mut layout.border);
    calc_border_padding(padding, size.0, size.1, &mut layout.padding);
    notify(notify_arg, id, layout);
    let new_pos1 = (
        layout.rect.start + layout.border.start + layout.padding.start,
        layout.rect.top + layout.border.top + layout.padding.top,
    );
    let new_pos2 = (
        layout.rect.end - layout.border.end - layout.padding.end,
        layout.rect.bottom - layout.border.bottom - layout.padding.bottom,
    );
    let size = (new_pos2.0 - new_pos1.0, new_pos2.1 - new_pos1.1);
    if eq_f32(size.0, old_w) && eq_f32(size.1, old_h) {
        LayoutResult::None
    } else {
        LayoutResult::Size(size)
    }
}

const EPSILON: f32 = std::f32::EPSILON * 1024.0;
#[inline]
fn eq_f32(v1: f32, v2: f32) -> bool {
    v1 == v2 || ((v2 - v1).abs() <= EPSILON)
}

// 节点的兄弟节点
fn node_iter<T: Default>(direction: Direction, node: &Node<T>) -> usize {
    if direction != Direction::RTL {
        node.next()
    } else {
        // 处理倒排的情况
        node.prev()
    }
}

fn grow_calc(info: &RelNodeInfo, per: f32, pos: &mut f32) -> (f32, f32) {
	let size = info.main + info.grow * per;
	// if let Number::Defined(r) = info.max_main {
	// 	size = size.min(r);
	// }
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + size + info.margin_main_end.or_else(0.0);
    (start, size)
}
fn grow_calc_reverse(info: &RelNodeInfo, per: f32, pos: &mut f32) -> (f32, f32) {
	let size = info.main + info.grow * per;
	// if let Number::Defined(r) = info.max_main {
	// 	size = size.min(r);
	// }
    let start = *pos - info.margin_main_end.or_else(0.0) - size;
    *pos = start - info.margin_main_start.or_else(0.0);
    (start, size)
}
fn margin_calc(info: &RelNodeInfo, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos + info.margin_main_start.or_else(per);
    *pos = start + info.main + info.margin_main_end.or_else(per);
    (start, info.main)
}
fn margin_calc_reverse(info: &RelNodeInfo, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos - info.margin_main_end.or_else(per) - info.main;
    *pos = start - info.margin_main_end.or_else(per);
    (start, info.main)
}
fn shrink_calc(info: &RelNodeInfo, per: f32, pos: &mut f32) -> (f32, f32) {
	let size = info.main - info.shrink as f32 * per;
	// if let Number::Defined(r) = info.min_main {
	// 	size = size.max(r);
	// }
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + size + info.margin_main_end.or_else(0.0);
    (start, size)
}
fn shrink_calc_reverse(info: &RelNodeInfo, per: f32, pos: &mut f32) -> (f32, f32) {
	let size = info.main - info.shrink as f32 * per;
	// if let Number::Defined(r) = info.min_main {
	// 	size = size.max(r);
	// }
    let start = *pos - info.margin_main_end.or_else(0.0) - size;
    *pos = start - info.margin_main_start.or_else(0.0);
    (start, size)
}

fn min_max_calc(mut value: f32, min_value: Number, max_value: Number) -> f32 {
	if let Number::Defined(r) = min_value {
		value = value.max(r);
	}
	if let Number::Defined(r) = max_value {
		value = value.min(r);
	}
    value
}

fn max_calc(value: Number, max_value: Number) -> Number {
	match (value, max_value) {
		(Number::Undefined, Number::Defined(_r)) => max_value,
		_ => value,
	}
}

fn main_calc(info: &RelNodeInfo, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos + info.margin_main_start.or_else(0.0);
    *pos = start + info.main + info.margin_main_end.or_else(0.0) + per;
    (start, info.main)
}
fn main_calc_reverse(info: &RelNodeInfo, per: f32, pos: &mut f32) -> (f32, f32) {
    let start = *pos - info.margin_main_end.or_else(0.0) - info.main;
    *pos = start - info.margin_main_start.or_else(0.0) - per;
    (start, info.main)
}
// 返回位置和大小
fn cross_calc(info: &RelNodeInfo, start: f32, end: f32, align_items: AlignItems) -> (f32, f32) {
    debug_println!(
        "{:?}cross_calc, start:{:?}, end:{:?}, info:{:?}",
        ppp(),
        start,
        end,
        info
    );
    match info.align_self {
        AlignSelf::Auto => match align_items {
            AlignItems::FlexStart => align_start(start, end, info),
            AlignItems::FlexEnd => align_end(start, end, info),
            AlignItems::Center => align_center(start, end, info),
            _ if info.cross_d.is_undefined() => align_stretch(start, end, info),
            _ => align_start(start, end, info), // 不支持baseline
        },
        AlignSelf::FlexStart => align_start(start, end, info),
        AlignSelf::FlexEnd => align_end(start, end, info),
        AlignSelf::Center => align_center(start, end, info),
        _ if info.cross_d.is_undefined() => align_stretch(start, end, info),
        _ => align_start(start, end, info), // 不支持baseline
    }
}
// 返回位置和大小
fn align_start(start: f32, end: f32, info: &RelNodeInfo) -> (f32, f32) {
    match info.margin_cross_start {
        Number::Defined(r) => (start + r, info.cross),
        _ => match info.margin_cross_end {
            Number::Defined(r) => (end - r - info.cross, info.cross),
            _ => ((start + end - info.cross) / 2.0, info.cross),
        },
    }
}
// 返回位置和大小
fn align_end(start: f32, end: f32, info: &RelNodeInfo) -> (f32, f32) {
    match info.margin_cross_end {
        Number::Defined(r) => (end - r - info.cross, info.cross),
        _ => match info.margin_cross_start {
            Number::Defined(r) => (start + r, info.cross),
            _ => ((start + end - info.cross) / 2.0, info.cross),
        },
    }
}
// 返回位置和大小
fn align_center(start: f32, end: f32, info: &RelNodeInfo) -> (f32, f32) {
    match info.margin_cross_start {
        Number::Defined(r) => match info.margin_cross_end {
            Number::Defined(rr) => ((start + end - info.cross - r - rr) / 2.0 + r, info.cross),
            _ => (start + r, info.cross),
        },
        _ => match info.margin_cross_end {
            Number::Defined(r) => (end - r - info.cross, info.cross),
            _ => ((start + end - info.cross) / 2.0, info.cross),
        },
    }
}
// 返回位置和大小
fn align_stretch(start: f32, end: f32, info: &RelNodeInfo) -> (f32, f32) {
    let r = info.margin_cross_start.or_else(0.0);
    let rr = info.margin_cross_end.or_else(0.0);
    (start + r, end - r - rr)
}

fn layout_node<T>(
    tree: &IdTree<u32>,
    i_nodes: &mut impl IndexMut<usize, Output = INode>,
    rect_style_map: &impl Index<usize, Output = RectStyle>,
    other_style_map: &impl Index<usize, Output = OtherStyle>,
    layout_map: &mut impl IndexMut<usize, Output = LayoutR>,
    notify: fn(&mut T, usize, &LayoutR),
    notify_arg: &mut T,
    id: usize,
    width: (f32, f32),
    height: (f32, f32),
    temp: &mut TempType,
    parent_size: (f32, f32),
) {
    let i_node = &mut i_nodes[id];
    match temp {
        TempType::CharIndex(r) => {
            // 文字布局
            let cnode = &mut i_node.text[*r];
            cnode.pos = (width.0, height.0);
            return;
        }
        _ => (),
    }
    let s = &other_style_map[id];
    let flex = ContainerStyle::new(s);
    let direction = s.direction;
    let border = s.border.clone();
    let padding = s.padding.clone();
    let n = &tree[id];
    let state = i_node.state;
    i_node.state.set_false(&INodeState::new(
        INodeStateType::ChildrenDirty as usize + INodeStateType::SelfDirty as usize,
    ));
    let child_head = n.children().head;
    let child_tail = n.children().tail;
    let x = calc_pos(s.position.start, s.position.end, parent_size.0, width.0);
    let y = calc_pos(s.position.top, s.position.bottom, parent_size.1, height.0);
    // 设置布局的值
    match temp {
        TempType::R(t) => {
            // 有Auto的节点需要父确定大小，然后自身的temp重计算及布局
            let layout = &mut layout_map[id];
            set_layout_result(
                layout,
                notify,
                notify_arg,
                id,
                (x, y),
                (width.1, height.1),
                &border,
                &padding,
            );
            let s = layout.get_content_size();
            let mc = t.main_cross(s.0, s.1);
            let line = t.reline(mc.0, mc.1);
            // 如果有临时缓存子节点数组
            t.layout(
                tree,
                i_nodes,
                rect_style_map,
                other_style_map,
                layout_map,
                notify,
                notify_arg,
                s,
                mc.0,
                mc.1,
                &line,
            );
        }
        TempType::None => {
            // 确定大小的节点，需要进一步布局
            let is_text = i_node.text.len() > 0 && !state.vnode();
            set_layout(
                tree,
                i_nodes,
                rect_style_map,
                other_style_map,
                layout_map,
                notify,
                notify_arg,
                id,
                is_text,
                child_head,
                child_tail,
                flex,
                direction,
                border,
                padding,
                state,
                (x, y),
                (width.1, height.1),
            );
        }
        _ => {
            // 有Auto的节点在计算阶段已经将自己的子节点都布局了，节点自身等待确定位置
            let layout = &mut layout_map[id];
            set_layout_result(
                layout,
                notify,
                notify_arg,
                id,
                (x, y),
                (width.1, height.1),
                &border,
                &padding,
            );
        }
    }
}

// 获得计算区域(大小和位置)， 大小为None表示自动计算
fn calc_rect(
    start: Dimension,
    end: Dimension,
    size: Dimension,
    margin_start: Dimension,
	margin_end: Dimension,
    parent: f32,
    _children_abs: bool,
    align: isize,
) -> (Number, f32) {
    let r = match size {
        Dimension::Points(r) => r,
        Dimension::Percent(r) => parent * r,
        _ => {
			// 通过明确的前后确定大小
            let mut rr = match start {
                Dimension::Points(rr) => rr,
				Dimension::Percent(rr) => parent * rr,
				_ => return (Number::Undefined, match end {
					Dimension::Points(rrr) => parent - rrr - margin_end.resolve_value(parent),
					Dimension::Percent(rrr) => parent - parent * rrr - margin_end.resolve_value(parent),
					_ => 0.0,
				}),
			};
			let mut rrr = match end {
                Dimension::Points(rrr) => rrr,
				Dimension::Percent(rrr) => parent * rrr,
				_ => return (Number::Undefined, margin_start.resolve_value(parent)),
			};
            rr += margin_start.resolve_value(parent);
			rrr += margin_end.resolve_value(parent);
			return (Number::Defined(parent - rr - rrr), rr);
        }
    };
    let rr = match start {
        Dimension::Points(rr) => rr,
        Dimension::Percent(rr) => parent * rr,
        _ => {
            // 后对齐
            let rrr = match end {
                Dimension::Points(rrr) => rrr,
                Dimension::Percent(rrr) => parent * rrr,
                _ => {
                    if align == 0 {
                        // 居中对齐
                        let s = (parent - r) * 0.5;
                        return calc_margin(s, s + r, r, margin_start, margin_end, parent);
                    } else if align > 0 {
                        // 后对齐
                        return (
                            Number::Defined(r),
                            parent - margin_end.resolve_value(parent) - r,
                        );
                    } else {
                        // 前对齐
                        return (Number::Defined(r), margin_start.resolve_value(parent));
                    }
                }
            };
            return (
                Number::Defined(r),
                parent - rrr - margin_end.resolve_value(parent) - r,
            );
        }
    };
    // 左右对齐
    let rrr = match end {
        Dimension::Points(rrr) => rrr,
        Dimension::Percent(rrr) => parent * rrr,
        _ => {
            // 前对齐
            return (Number::Defined(r), rr + margin_start.resolve_value(parent));
        }
    };
    calc_margin(rr, parent - rrr, r, margin_start, margin_end, parent)
}
// 根据宽高获得内容宽高
fn calc_content_size(
    size: Number,
    b_start: Dimension,
    b_end: Dimension,
    p_start: Dimension,
    p_end: Dimension,
) -> Number {
    match size {
        Number::Defined(r) => Number::Defined(
            r - b_start.resolve_value(r)
                - b_end.resolve_value(r)
                - p_start.resolve_value(r)
                - p_end.resolve_value(r),
        ),
        _ => size,
    }
}
// 根据内容宽高计算宽高
fn calc_size_from_content(
    mut points: f32,
    b_start: Dimension,
    b_end: Dimension,
    p_start: Dimension,
    p_end: Dimension,
) -> f32 {
    let mut p = 0.0;
    percent_calc(b_start, &mut points, &mut p);
    percent_calc(b_end, &mut points, &mut p);
    percent_calc(p_start, &mut points, &mut p);
    percent_calc(p_end, &mut points, &mut p);
    reverse_calc(points, p)
}
// 根据固定值和百分比反向计算大小
fn reverse_calc(points: f32, percent: f32) -> f32 {
    if percent >= 1.0 {
        // 防止百分比大于100%
        points
    } else {
        points / (1.0 - percent)
    }
}
fn percent_calc(d: Dimension, points: &mut f32, percent: &mut f32) -> bool {
    match d {
        Dimension::Points(r) => *points += r,
        Dimension::Percent(r) => *percent += r,
        _ => return false,
    };
    true
}

// 已经确定了布局的区域， 需要计算布局中的border和padding
fn calc_border_padding(s: &Rect<Dimension>, w: f32, h: f32, r: &mut Rect<f32>) {
    r.start = s.start.resolve_value(w);
    r.end = s.end.resolve_value(w);
    r.top = s.top.resolve_value(h);
    r.bottom = s.bottom.resolve_value(h);
}

// 计算margin, margin=Auto时自动填充剩余空间， 两边都Auto时平分剩余空间
fn calc_margin(
    mut start: f32,
    mut end: f32,
    size: f32,
    margin_start: Dimension,
    margin_end: Dimension,
    parent: f32,
) -> (Number, f32) {
    match margin_start {
        Dimension::Points(r) => {
            start += r;
            end = start + size;
        }
        Dimension::Percent(r) => {
            start += r * parent;
            end = start + size;
        }
        _ => {
            match margin_end {
                Dimension::Points(r) => {
                    end -= r;
                    start = end - size;
                }
                Dimension::Percent(r) => {
                    end -= r * parent;
                    start = end - size;
                }
                _ => {
                    debug_println!(
                        "calc_margin auto=============end: {}, start:{}, size:{}",
                        end,
                        start,
                        size
                    );
                    // 平分剩余大小
                    let d = (end - start - size) / 2.0;
                    start += d;
                    end -= d;
                }
            }
        }
    }
    (Number::Defined(end - start), start)
}

// 在flex计算的区域中 根据pos的位置进行偏移
fn calc_pos(position_start: Dimension, position_end: Dimension, parent: f32, pos: f32) -> f32 {
    match position_start {
        Dimension::Points(r) => pos + r,
        Dimension::Percent(r) => pos + parent * r,
        _ => match position_end {
            Dimension::Points(r) => pos - r,
            Dimension::Percent(r) => pos - parent * r,
            _ => pos,
        },
    }
}
// 计算子节点的大小
fn calc_number(s: Dimension, parent: f32) -> Number {
    match s {
        Dimension::Points(r) => Number::Defined(r),
        Dimension::Percent(r) => Number::Defined(parent * r),
        _ => Number::Undefined,
    }
}

fn calc_length(length: Number, min_length: Number) -> Number{
	match (length, min_length) {
		(Number::Undefined, Number::Defined(_)) => min_length,
		(Number::Defined(l1), Number::Defined(l2)) => if l1 > l2 {length} else {min_length},
		_ => length
	}
}
pub(crate) static mut PP: usize = 0;
pub(crate) static mut PC: usize = 0;
