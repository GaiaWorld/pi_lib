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

/// 注意事项：
/// 1. 根节点必须是区域（绝对定位， 绝对位置，绝对尺寸）
/// 2. 

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
use std::ops::{Index, IndexMut};

// use map::vecmap::VecMap;

use crate::calc::*;
use crate::dirty::*;
use crate::idtree::{IdTree as IdTree1, Node as Node1};
use crate::style::*;

type IdTree = IdTree1<usize>;
type Node = Node1<usize>;

pub fn set_display(id: usize, v: Display, dirty: &mut LayerDirty, tree: &IdTree, i_nodes: &mut impl IndexMut<usize, Output = INode>, rect_style_map: &impl Index<usize, Output = RectStyle>, other_style_map: &impl Index<usize, Output = OtherStyle>,) {
	let n = &tree[id];
	let i_node = &mut i_nodes[id];
	let rect_style = &rect_style_map[id];
	let other_style = &other_style_map[id];
	let parent = n.parent();
	let state = i_node.state;
	if v != Display::None {
		if calc_abs(other_style, i_node) {
			calc_abs_rect(rect_style, other_style, i_node);
		}
		calc_size_defined(rect_style, i_node);
		set_self_dirty(dirty, id, n, i_node);
		set_parent(i_nodes, tree, dirty, other_style, state, parent, true)
	} else if n.layer() > 0 {
		mark_children_dirty(tree, i_nodes, dirty, parent)
	}
}

pub fn compute<T>(dirty: &mut LayerDirty, tree: &IdTree, i_nodes: &mut impl IndexMut<usize, Output = INode>, rect_styles: &impl Index<usize, Output = RectStyle>, other_styles: &impl Index<usize, Output = OtherStyle>, layouts: &mut impl IndexMut<usize, Output = LayoutR>, notify: fn(&mut T, usize, &LayoutR), notify_arg: &mut T) {
	for (id, layer) in dirty.iter() {
		let (node, i_node) = match tree.get(*id) {
			Some(n) => (n,  &mut i_nodes[*id]),
			_ => continue,
		};
		debug_println!("    calc: {:?} children_dirty:{:?} self_dirty:{:?} children_abs:{:?} children_abs_rect:{:?} children_no_align_self:{:?} children_index:{:?} vnode:{:?} abs:{:?} abs_rect:{:?} size_defined:{:?}, layer:{}", id, i_node.state.children_dirty(), i_node.state.self_dirty(), i_node.state.children_abs(), i_node.state.children_abs_rect(), i_node.state.children_no_align_self(), i_node.state.children_index(), i_node.state.vnode(), i_node.state.abs(), i_node.state.abs_rect(), i_node.state.size_defined(), layer);
		let state = i_node.state;
		if !(state.self_dirty() || state.children_dirty()) {
			continue;
		}
		i_node.state.set_false(&INodeState::new(INodeStateType::ChildrenDirty as usize + INodeStateType::SelfDirty as usize));
		if node.layer() == 0 || i_node.state.vnode() {
			// 不在树上或虚拟节点
			continue;
		}
		let children = node.children();
		let child_head = children.head;
		let child_tail = children.tail;
		unsafe {
			PC = 0;
			PP = 0
		};
		if state.abs() {
			// 如果节点是绝对定位， 则重新计算自身的布局数据
			let (parent_size, flex) = if !i_node.state.abs_rect() {
				// 如果节点自身不是绝对区域，则需要获得父容器的内容大小
				let layout = &mut layouts[node.parent()];
				let style = &other_styles[node.parent()];
				(layout.get_content_size(), ContainerStyle::new(style))
			} else {
				((0.0, 0.0), ContainerStyle{justify_content: JustifyContent::FlexStart, align_content: AlignContent::FlexStart, flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::NoWrap, align_items: AlignItems::FlexStart})
			};
			abs_layout(
				tree,
				i_nodes,
				rect_styles,
				other_styles,
				layouts,
				notify,
				notify_arg,
				*id,
				child_head,
				child_tail,
				state,
				parent_size,
				&flex
			);
		} else {
			// 如果节点是相对定位，被设脏表示其修改的数据不会影响父节点的布局 则先重新计算自身的布局数据，然后修改子节点的布局数据
			rel_layout(
				tree,
				i_nodes,
				rect_styles,
				other_styles,
				layouts,
				notify,
				notify_arg,
				*id,
				child_head,
				child_tail,
				state,
			);
		}
	}
	dirty.clear();
}
// 样式改变设置父节点
fn set_parent(
	i_nodes: &mut impl IndexMut<usize, Output = INode>,
    tree: &IdTree,
    dirty: &mut LayerDirty,
    style: &OtherStyle,
    state: INodeState,
    parent: usize,
    mark: bool,
) {
    if parent == 0 {
        return;
    }
	let n = &tree[parent];
	let i_node = &mut i_nodes[parent];
    if !state.abs() {
        i_node.state.children_abs_false();
    } else if !state.abs_rect() {
        i_node.state.children_abs_rect_false();
    }
    if style.align_self != AlignSelf::Auto {
        i_node.state.children_no_align_self_false();
    }
    if style.order != 0 {
        i_node.state.children_index_false();
    }
    if mark && n.layer() > 0 {
        mark_children_dirty(tree, i_nodes, dirty, parent)
    }
}
// 设置自身样式， 设自身脏，如果节点是size=auto并且不是绝对定位, 则继续设置其父节点ChildrenDirty脏
pub fn set_self_style(tree: &IdTree, i_nodes: &mut impl IndexMut<usize, Output = INode>, dirty: &mut LayerDirty, id: usize, style: &OtherStyle) {
    if style.display == Display::None {
        // 如果是隐藏
        return;
    }
	let n = &tree[id];
	let i_node = &mut i_nodes[id];
    let parent = set_self_dirty(dirty, id, n, i_node);
    if parent > 0 {
        mark_children_dirty(tree, i_nodes, dirty, parent)
    }
}

// 设置会影响子节点布局的样式， 设children_dirty脏，如果节点是size=auto并且不是绝对定位, 则继续设置其父节点ChildrenDirty脏
pub fn set_children_style(tree: &IdTree, i_nodes: &mut impl IndexMut<usize, Output = INode>, dirty: &mut LayerDirty, id: usize, style: &OtherStyle) {
    if style.display == Display::None {
        // 如果是隐藏
        return;
    }
	mark_children_dirty(tree, i_nodes, dirty, id)
}
// 设置一般样式， 设父节点脏
pub fn set_normal_style(tree: &IdTree, i_nodes: &mut impl IndexMut<usize, Output = INode>, dirty: &mut LayerDirty, id: usize, style: &OtherStyle) {
    if style.display == Display::None {
        // 如果是隐藏
        return;
    }
	let n = &tree[id];
	let i_node = &i_nodes[id];
    let parent = n.parent();
    let state = i_node.state;
    set_parent(i_nodes, tree, dirty, style, state, parent, true)
}
// 设置区域 pos margin size
pub fn set_rect(
	tree: &IdTree,
	i_nodes: &mut impl IndexMut<usize, Output = INode>,
    dirty: &mut LayerDirty,
	id: usize,
	rect_style: &RectStyle,
    other_style: &OtherStyle,
    is_abs: bool,
    is_size: bool,
) {
    if other_style.display == Display::None {
        // 如果是隐藏
        return;
    }
	let n = &tree[id];
	let i_node = &mut i_nodes[id];
    if is_abs {
        calc_abs(other_style, i_node);
    }
    if is_size {
        calc_size_defined(rect_style, i_node);
	}
	debug_println!("set rect dirty=====================");
	set_self_dirty(dirty, id, n, i_node);
	// 如果是绝对定位，则仅设置自身脏
    let mark = if other_style.position_type == PositionType::Absolute {
        calc_abs_rect(rect_style, other_style, i_node);
        false
    } else {
        true
    };
    let parent = n.parent();
    let state = i_node.state;
    set_parent(i_nodes, tree, dirty, other_style, state, parent, mark)
}
// 计算是否绝对区域
fn calc_abs(style: &OtherStyle, n: &mut INode) -> bool {
    if style.position_type == PositionType::Absolute {
        n.state.abs_true();
        true
    } else {
        n.state.abs_false();
        false
    }
}
// 计算是否绝对区域
fn calc_abs_rect(rect_style: &RectStyle, other_style: &OtherStyle, n: &mut INode) -> bool {
    if other_style.position.start.is_points()
        && other_style.position.top.is_points()
        && rect_style.margin.start.is_points()
        && rect_style.margin.top.is_points()
        && rect_style.size.width.is_points()
        && rect_style.size.height.is_points()
    {
        n.state.abs_rect_true();
        true
    } else {
        n.state.abs_rect_false();
        false
    }
}
// 计算是否大小已经定义
fn calc_size_defined(style: &RectStyle, n: &mut INode) -> bool {
    if style.size.width.is_defined() && style.size.height.is_defined() {
        n.state.size_defined_true();
        true
    } else {
        n.state.size_defined_false();
        false
    }
}
// 设置节点自身脏, 如果节点是size=auto并且不是绝对定位, 则返回父节点id，需要继续设置其父节点脏
fn set_self_dirty(dirty: &mut LayerDirty, id: usize, n: &Node, i_node: &mut INode) -> usize {
	if !i_node.state.self_dirty() {
		i_node.state.self_dirty_true();
		if n.layer() > 0 {
			if !i_node.state.children_dirty() {
				dirty.mark(id, n.layer());
			}
			if i_node.state.vnode() || !(i_node.state.size_defined() || i_node.state.abs()) {
				return n.parent();
			}
		}
	}
	0
}
// // 设置节点脏, 如果节点是size=auto并且不是绝对定位, 则返回父节点id，需要继续设置其父节点脏
// fn set_children_dirty(dirty: &mut LayerDirty, id: usize, n: &Node, i_node: &mut INode) -> usize {
// 	if !i_node.state.children_dirty() {
// 		i_node.state.children_dirty_true();
// 		if n.layer() > 0 {
// 			if !i_node.state.self_dirty() {
// 				dirty.mark(id, n.layer());
// 			}
// 			if i_node.state.vnode() || !(i_node.state.size_defined() || i_node.state.abs()) {
// 				return n.parent();
// 			}
// 		}
// 	}
//     0
// }
// 设置节点children_dirty脏, 如果节点是size=auto并且不是绝对定位,也不是虚拟节点, 则继续设置其父节点children_dirty脏
pub fn mark_children_dirty(tree: &IdTree, i_nodes: &mut impl IndexMut<usize, Output = INode>, dirty: &mut LayerDirty, mut id: usize) {
    while id > 0 {
		let i_node = &mut i_nodes[id];
        if i_node.state.children_dirty() {
            break;
		}
		let n = &tree[id];
		i_node.state.children_dirty_true();
		if !i_node.state.self_dirty() {
			dirty.mark(id, n.layer());
		}
        if (i_node.state.size_defined() || i_node.state.abs()) && !i_node.state.vnode() {
            break;
        }
        id = n.parent()
    }
}