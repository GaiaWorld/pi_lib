/**
// 布局支持虚拟节点， 虚拟节点下的子节点提到上面来参与布局，这样能很好的支持图文混排的布局
// 如果节点的size=Auto, 在绝对定位下并且设置了right和bottom, 则left-right和top-bottom来决定大小. 否则表明是子节点决定大小.
// 子节点计算大小后, 如果节点是flex并且是相对定位, 并且grow或shrink不为0, 则会再次计算大小
// 设脏的情况: 1. 如果节点是绝对定位, 则只设自身脏. 2. 相对定位下, 如果属性是容器值, 则设节点自身脏, 否则设父节点脏. 如果脏节点的size=Auto, 则向上传播脏, 直到父节点为绝对定位或size!=Auto.
// 计算时, 如果节点为绝对定位, 先检查size=Auto. 如果size=Auto, 则先根据left-right等来确定大小,否则需要根据子节点来计算大小. 如果size!=Auto, 则可能根据父节点大小先计算自身的layout, 然后计算子节点布局.
// 计算时, 节点为相对定位时, size!=Auto. 根据自身的layout, 来计算子节点布局.
// 计算子节点布局时, 第一次遍历子节点, 如果相对定位子节点的大小为Auto, 则判断是否脏, 如果脏, 则需要递归计算大小. 第二次遍历时， 如果节点有grow_shrink并且计算后大小有变化, 或者有Stretch, 则需要再次计算该子节点布局.
// 计算子节点布局时, 节点内部保留缓存计算中间值.
// 在盒子模型中， size position margin，三者中size优先级最高。 首先就是确定size，优先级依次是：1明确指定，2通过left-right能计算出来，3子节点撑大。 在position中left top不指定值的话默认为0, right bottom为自动计算的填充值，比如right=ParentContentWidth-left-margin_left-width--margin_right。而magin=Auto是自动填充left-right和width中间的值，如果没有明确指定left和right，magin=Auto最后的值就是margin=0
// 注意： 为了不反复计算自动大小，如果父节点的主轴为自动大小，则flex-wrap自动为NoWrap。这个和浏览器的实现不一致！
// TODO aspect_ratio 要求width 或 height 有一个为auto，如果都被指定，则aspect_ratio被忽略
// TODO min_size max_size 仅作用在size上， 需要确认是否参与grow shrink的计算，


//浏览器版本的flex实现不合理的地方
// 1、绝对定位的元素不应该受flex中的对齐属性的影响。 绝对定位本身就支持居中等对齐方式
    absolute_layout_align_items_and_justify_content_center
 2、自动大小的容器，其大小受子节点大小计算的影响，flex-basis这个时候并没有参与计算，但浏览器版本行和列的实现不一致，列的情况下子节点的flex-basis会影响父容器的大小，行不会。
    flex_basis_unconstraint_column
3、自动计算主轴大小的容器，其折行属性应该为不折行，这样子节点顺序放置后，才好计算容器的主轴大小。浏览器版本就不是这么实现的
 4、如果A 包含 B，B包含C， A C 都有大小，B本身自动计算大小，这种情况下，浏览器的实现是B就不受A上的flex-basis grow shrink 影响，这样也不太合理。浏览器的计算似乎是从C先算B，然后不在二次计算B受的约束。 而正确的方式应该是先从A算B，发现B为自动大小，接着算C，反过来计算B的大小，然后受flex-basis影响，B大小变化后，再影响C的位置。
    flex_basis_smaller_then_content_with_flex_grow_large_size
*/

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use map::vecmap::VecMap;

use crate::calc::*;
use crate::dirty::*;
use crate::geometry::*;
use crate::idtree::*;
use crate::number::Number;
use crate::style::*;

#[derive(Default)]
pub struct LayoutTree {
    style_map: VecMap<Style>,
    layout_map: VecMap<LayoutR>,
    tree: IdTree<INode>,
    dirty: LayerDirty,
}

impl LayoutTree {
    pub fn get_style(&self, id: usize) -> Option<&Style> {
        self.style_map.get(id)
    }
    pub unsafe fn get_style_unchecked(&self, id: usize) -> &Style {
        self.style_map.get_unchecked(id)
    }
    pub fn get_layout(&self, id: usize) -> Option<&LayoutR> {
        self.layout_map.get(id)
    }
    pub unsafe fn get_layout_unchecked(&self, id: usize) -> &LayoutR {
        self.layout_map.get_unchecked(id)
    }

    pub fn set_style_display(&mut self, id: usize, v: Display) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        if style.display == v {
            return;
        }
        style.display = v;
        let n = unsafe { self.tree.get_unchecked_mut(id) };
        let parent = n.parent();
        let state = n.data.state;
        if v != Display::None {
            if calc_abs(style, n) {
                calc_abs_rect(style, n);
            }
            calc_size_defined(style, n);
            set_parent(&mut self.tree, &mut self.dirty, style, state, parent, true)
        } else if n.layer() > 0 {
            mark_dirty(&mut self.tree, &mut self.dirty, parent)
        }
    }
    pub fn set_style_position_type(&mut self, id: usize, v: PositionType) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        if style.position_type == v {
            return;
        }
        style.position_type = v;
        set_rect(&mut self.tree, &mut self.dirty, id, style, true, false)
    }
    pub fn set_style_direction(&mut self, id: usize, v: Direction) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.direction = v;
        set_self_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_flex_direction(&mut self, id: usize, v: FlexDirection) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.flex_direction = v;
        set_self_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_flex_wrap(&mut self, id: usize, v: FlexWrap) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.flex_wrap = v;
        set_self_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_justify_content(&mut self, id: usize, v: JustifyContent) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.justify_content = v;
        set_self_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_align_items(&mut self, id: usize, v: AlignItems) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.align_items = v;
        set_self_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_align_content(&mut self, id: usize, v: AlignContent) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.align_content = v;
        set_self_style(&mut self.tree, &mut self.dirty, id, style)
    }

    pub fn set_style_order(&mut self, id: usize, v: isize) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.order = v;
        set_normal_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_flex_basis(&mut self, id: usize, v: Dimension) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.flex_basis = v;
        set_normal_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_flex_grow(&mut self, id: usize, v: f32) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.flex_grow = v;
        set_normal_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_flex_shrink(&mut self, id: usize, v: f32) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.flex_shrink = v;
        set_normal_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_align_self(&mut self, id: usize, v: AlignSelf) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.align_self = v;
        set_normal_style(&mut self.tree, &mut self.dirty, id, style)
    }

    pub fn set_style_position(&mut self, id: usize, v: Rect<Dimension>) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.position = v;
        set_rect(&mut self.tree, &mut self.dirty, id, style, false, false)
    }
    pub fn set_style_margin(&mut self, id: usize, v: Rect<Dimension>) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.margin = v;
        set_rect(&mut self.tree, &mut self.dirty, id, style, false, false)
    }
    pub fn set_style_padding(&mut self, id: usize, v: Rect<Dimension>) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.padding = v;
        set_self_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_border(&mut self, id: usize, v: Rect<Dimension>) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.border = v;
        set_self_style(&mut self.tree, &mut self.dirty, id, style)
    }
    pub fn set_style_size(&mut self, id: usize, v: Size<Dimension>) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.size = v;
        set_rect(&mut self.tree, &mut self.dirty, id, style, false, true)
    }
    pub fn set_style_min_size(&mut self, id: usize, v: Size<Dimension>) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.min_size = v;
    }
    pub fn set_style_max_size(&mut self, id: usize, v: Size<Dimension>) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.max_size = v;
    }
    pub fn set_style_aspect_ratio(&mut self, id: usize, v: Number) {
        let style = unsafe { self.style_map.get_unchecked_mut(id) };
        style.aspect_ratio = v;
    }
    pub fn set_node_vnode(&mut self, id: usize, vnode: bool) {
        let node = unsafe { self.tree.get_unchecked_mut(id) };
        if vnode {
            node.data.state.vnode_true();
        } else {
            node.data.state.vnode_false();
        }
    }
    pub fn set_node_measure(&mut self, id: usize, measure: Option<(MeasureFunc, usize)>) {
        let node = unsafe { self.tree.get_unchecked_mut(id) };
        node.data.measure = measure;
        node.data.state.measure_r_false();
    }

    pub fn insert(
        &mut self,
        id: usize,
        parent: usize,
        brother: usize,
        insert: InsertType,
        s: Style,
    ) {
        self.tree.create(id);
        if brother > 0 {
            self.tree.insert_brother(id, brother, insert);
        } else {
            self.tree.insert_child(id, parent, std::usize::MAX);
        }
        self.set_style(id, s);
        self.layout_map.insert(id, LayoutR::default());
    }

    pub fn remove(&mut self, id: usize) {
        match self.tree.get_info(id) {
            Some(r) => {
                if r.0 > 0 {
                    mark_dirty(&mut self.tree, &mut self.dirty, r.0)
                }
                self.tree.remove(id, r);
            }
            _ => (),
        }
	}

    pub fn set_style(&mut self, id: usize, s: Style) {
        if s.display == Display::None {
            self.style_map.insert(id, s);
            return;
        }
        set_rect(&mut self.tree, &mut self.dirty, id, &s, true, true);
        self.style_map.insert(id, s);
    }
    pub fn compute<T>(&mut self, notify: fn(&mut T, usize, &LayoutR), notify_arg: &mut T) {
        for (id, _layer) in self.dirty.iter() {
            let node = match self.tree.get_mut(*id) {
                Some(n) => n,
                _ => continue,
            };
            debug_println!("    calc: {:?} dirty:{:?} children_abs:{:?} children_abs_rect:{:?} children_no_align_self:{:?} children_index:{:?} vnode:{:?} abs:{:?} abs_rect:{:?} size_defined:{:?}", id, node.data.state.dirty(), node.data.state.children_abs(), node.data.state.children_abs_rect(), node.data.state.children_no_align_self(), node.data.state.children_index(), node.data.state.vnode(), node.data.state.abs(), node.data.state.abs_rect(), node.data.state.size_defined());
            if !node.data.state.dirty() {
                continue;
            }
            let state = node.data.state;
            node.data.state.dirty_false();
            if node.layer() == 0 {
                // 不在树上
                continue;
            }
            let children = node.children();
            let child_head = children.head;
            let child_tail = children.tail;
            unsafe {
                pc = 0;
                pp = 0
            };
            if state.abs() {
                // 如果节点是绝对定位， 则重新计算自身的布局数据
                let parent_size = if !node.data.state.abs_rect() {
                    // 如果节点自身不是绝对区域，则需要获得父容器的内容大小
                    let layout = unsafe { self.layout_map.get_unchecked(node.parent()) };
                    layout.get_content_size()
                } else {
                    (0.0, 0.0)
                };
                abs_layout(
                    &mut self.tree,
                    &mut self.style_map,
                    &mut self.layout_map,
                    notify,
                    notify_arg,
                    *id,
                    child_head,
                    child_tail,
                    state,
                    parent_size,
                );
            } else {
                // 如果节点是相对定位，被设脏表示其修改的数据不会影响父节点的布局 则先重新计算自身的布局数据，然后修改子节点的布局数据
                rel_layout(
                    &mut self.tree,
                    &mut self.style_map,
                    &mut self.layout_map,
                    notify,
                    notify_arg,
                    *id,
                    child_head,
                    child_tail,
                    state,
                );
            }
        }
    }
}
// 样式改变设置父节点
fn set_parent(
    tree: &mut IdTree<INode>,
    dirty: &mut LayerDirty,
    style: &Style,
    state: INodeState,
    parent: usize,
    mark: bool,
) {
    if parent == 0 {
        return;
    }
    let n = unsafe { tree.get_unchecked_mut(parent) };
    if !state.abs() {
        n.data.state.children_abs_false();
    } else if !state.abs_rect() {
        n.data.state.children_abs_rect_false();
    }
    if style.align_self != AlignSelf::Auto {
        n.data.state.children_no_align_self_false();
    }
    if style.order != 0 {
        n.data.state.children_index_false();
    }
    if mark && n.layer() > 0 {
        mark_dirty(tree, dirty, parent)
    }
}
// 设置自身样式， 设自身脏，如果节点是size=auto并且不是绝对定位, 则继续设置其父节点脏
fn set_self_style(tree: &mut IdTree<INode>, dirty: &mut LayerDirty, id: usize, style: &Style) {
    if style.display == Display::None {
        // 如果是隐藏
        return;
    }
    let n = unsafe { tree.get_unchecked_mut(id) };
    let parent = set_dirty(dirty, id, n);
    if parent > 0 {
        mark_dirty(tree, dirty, parent)
    }
}
// 设置一般样式， 设父节点脏
fn set_normal_style(tree: &mut IdTree<INode>, dirty: &mut LayerDirty, id: usize, style: &Style) {
    if style.display == Display::None {
        // 如果是隐藏
        return;
    }
    let n = unsafe { tree.get_unchecked(id) };
    let parent = n.parent();
    let state = n.data.state;
    set_parent(tree, dirty, style, state, parent, true)
}
// 设置区域 pos margin size
fn set_rect(
    tree: &mut IdTree<INode>,
    dirty: &mut LayerDirty,
    id: usize,
    style: &Style,
    is_abs: bool,
    is_size: bool,
) {
    if style.display == Display::None {
        // 如果是隐藏
        return;
    }
    let n = unsafe { tree.get_unchecked_mut(id) };
    if is_abs {
        calc_abs(style, n);
    }
    if is_size {
        calc_size_defined(style, n);
    }
    let mark = if style.position_type == PositionType::Absolute {
        calc_abs_rect(style, n);
        // 如果是绝对定位，则仅设置自身脏
        set_dirty(dirty, id, n);
        false
    } else {
        true
    };
    let parent = n.parent();
    let state = n.data.state;
    set_parent(tree, dirty, style, state, parent, mark)
}
// 计算是否绝对区域
fn calc_abs(style: &Style, n: &mut Node<INode>) -> bool {
    if style.position_type == PositionType::Absolute {
        n.data.state.abs_true();
        true
    } else {
        n.data.state.abs_false();
        false
    }
}
// 计算是否绝对区域
fn calc_abs_rect(style: &Style, n: &mut Node<INode>) -> bool {
    if style.position.start.is_points()
        && style.position.top.is_points()
        && style.margin.start.is_points()
        && style.margin.top.is_points()
        && style.size.width.is_points()
        && style.size.height.is_points()
    {
        n.data.state.abs_rect_true();
        true
    } else {
        n.data.state.abs_rect_false();
        false
    }
}
// 计算是否大小已经定义
fn calc_size_defined(style: &Style, n: &mut Node<INode>) -> bool {
    if style.size.width.is_defined() && style.size.height.is_defined() {
        n.data.state.size_defined_true();
        true
    } else {
        n.data.state.size_defined_false();
        false
    }
}
// 设置节点脏, 如果节点是size=auto并且不是绝对定位, 则返回父节点id，需要继续设置其父节点脏
fn set_dirty(dirty: &mut LayerDirty, id: usize, n: &mut Node<INode>) -> usize {
    if n.layer() > 0 && !n.data.state.dirty() {
        n.data.state.dirty_true();
        dirty.mark(id, n.layer());
        if n.data.state.vnode() || !(n.data.state.size_defined() || n.data.state.abs()) {
            return n.parent();
        }
    }
    0
}
// 设置节点脏, 如果节点是size=auto并且不是绝对定位,也不是虚拟节点, 则继续设置其父节点脏
fn mark_dirty(tree: &mut IdTree<INode>, dirty: &mut LayerDirty, mut id: usize) {
    while id > 0 {
        let n = unsafe { tree.get_unchecked_mut(id) };
        if n.data.state.dirty() {
            break;
        }
        n.data.state.dirty_true();
        dirty.mark(id, n.layer());
        if (n.data.state.size_defined() || n.data.state.abs()) && !n.data.state.vnode() {
            break;
        }
        id = n.parent()
    }
}

#[test]
pub fn test_abs() {
    let mut tree = LayoutTree::default();
    tree.insert(
        1,
        0,
        0,
        InsertType::Back,
        abs_rect(0.0, 0.0, 1920.0, 1024.0),
    );
    tree.insert(
        2,
        1,
        0,
        InsertType::Back,
        abs_rect(100.0, 80.0, 600.0, 400.0),
    );
    tree.insert(
        3,
        1,
        0,
        InsertType::Back,
        abs_per(
            Dimension::Percent(0.2),
            Dimension::Percent(0.2),
            Dimension::Percent(0.6),
            Dimension::Percent(0.6),
        ),
    );
    tree.insert(
        4,
        1,
        0,
        InsertType::Back,
        abs_center(
            Dimension::Percent(0.2),
            Dimension::Percent(0.2),
            Dimension::Percent(0.6),
            Dimension::Percent(0.6),
        ),
    );
    tree.insert(
        5,
        1,
        0,
        InsertType::Back,
        abs_center(
            Dimension::Auto,
            Dimension::Auto,
            Dimension::Percent(0.6),
            Dimension::Percent(0.6),
        ),
    );
    tree.insert(
        6,
        1,
        0,
        InsertType::Back,
        abs_center(
            Dimension::Auto,
            Dimension::Auto,
            Dimension::Auto,
            Dimension::Auto,
        ),
    );
    let mut count = 0;
    tree.compute(print, &mut count);
}
#[test]
pub fn test_flex() {
    let mut tree = LayoutTree::default();
    tree.insert(
        1,
        0,
        0,
        InsertType::Back,
        abs_rect(0.0, 0.0, 1920.0, 1024.0),
    );
    tree.insert(2, 1, 0, InsertType::Back, flex(300.0, 260.0));
    tree.set_style_flex_wrap(2, FlexWrap::Wrap);
    tree.insert(3, 2, 0, InsertType::Back, flex(120.0, 100.0));
    tree.insert(4, 2, 3, InsertType::Back, flex(120.0, 100.0));
    tree.insert(5, 2, 0, InsertType::Back, flex(120.0, 100.0));
    let mut count = 0;
    tree.compute(print, &mut count);
}

fn print(count: &mut usize, id: usize, layout: &LayoutR) {
    *count += 1;
    debug_println!("result: {:?} {:?} {:?}", *count, id, layout);
}
fn flex(w: f32, h: f32) -> Style {
    let mut s = Style::default();
    s.size.width = Dimension::Points(w);
    s.size.height = Dimension::Points(h);
    // s.margin.start = mw;
    // s.margin.end = mw;
    // s.margin.top = my;
    // s.margin.bottom = my;
    s
}

fn abs_rect(x: f32, y: f32, w: f32, h: f32) -> Style {
    let mut s = Style::default();
    s.position_type = PositionType::Absolute;
    s.size.width = Dimension::Points(w);
    s.size.height = Dimension::Points(h);
    s.position.start = Dimension::Points(x);
    s.position.top = Dimension::Points(y);
    s.margin.start = Dimension::Points(0.0);
    s.margin.top = Dimension::Points(0.0);
    s
}
fn abs_per(x: Dimension, y: Dimension, w: Dimension, h: Dimension) -> Style {
    let mut s = Style::default();
    s.position_type = PositionType::Absolute;
    s.size.width = w;
    s.size.height = h;
    s.position.start = x;
    s.position.top = y;
    s.margin.start = Dimension::Points(0.0);
    s.margin.top = Dimension::Points(0.0);
    s
}
fn abs_center(mw: Dimension, my: Dimension, w: Dimension, h: Dimension) -> Style {
    let mut s = Style::default();
    s.position_type = PositionType::Absolute;
    s.size.width = w;
    s.size.height = h;
    s.position.start = Dimension::Points(60.0);
    s.position.top = Dimension::Points(60.0);
    s.position.end = Dimension::Points(60.0);
    s.position.bottom = Dimension::Points(60.0);
    s.margin.start = mw;
    s.margin.end = mw;
    s.margin.top = my;
    s.margin.top = my;
    s
}
