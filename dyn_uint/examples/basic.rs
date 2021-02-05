//! dyn_uint的基本用例

extern crate dyn_uint;

use dyn_uint::{SlabFactory, UintFactory, ClassFactory};

struct A<T> {
    vec1: Vec<(T, usize)>,
    vec2: Vec<(T, usize)>,
    factory: SlabFactory<CurContainer, ()>,
}

// 该数据结构利用SlabFactory来维护元素唯一索引
impl<T> A<T> {
    pub fn new() -> Self {
        A{
            vec1: Vec::new(),
            vec2: Vec::new(),
            factory: SlabFactory::new(),
        }
    }

    // 将元素插入到vec1中，并返回元素在数据结构A中的唯一索引
    pub fn inser_to_vec1(&mut self, v: T) -> usize {
        let unit = self.factory.create(self.vec1.len(), CurContainer::Vec1, ());
        self.vec1.push((v, unit));
        unit
    }

    // 将元素插入到vec2中，并返回元素在数据结构A中的唯一索引
    pub fn inser_to_vec2(&mut self, v: T) -> usize {
        let unit = self.factory.create(self.vec2.len(), CurContainer::Vec2, ());
        self.vec2.push((v, unit));
        unit
    }

    // 将一个元素移动从一个数据结构移动到另一个数据结构
    // 保持元素原有的唯一标识unit不变
    pub fn move_obj(&mut self, unit: usize) {
        match self.factory.try_load(unit) {
            Some(index) => match self.factory.get_class(unit){
                CurContainer::Vec1 => move1(index, unit, &mut self.factory, &mut self.vec1, &mut self.vec2, CurContainer::Vec2),
                CurContainer::Vec2 => move1(index, unit, &mut self.factory, &mut self.vec2, &mut self.vec1, CurContainer::Vec1)
            },
            None => ()
        };
    }

    // 根据unit查询元素
    pub fn query(&self, unit: usize) -> Option<&T> {
        match self.factory.try_load(unit) {
            Some(index) => match self.factory.get_class(unit){
                CurContainer::Vec1 => match self.vec1.get(index) {
                    Some(r) => Some(&r.0),
                    None => None
                } ,
                CurContainer::Vec2 => match self.vec2.get(index) {
                    Some(r) => Some(&r.0),
                    None => None
                }
            },
            None => None,
        }
    }
}

// 用于标记元素所在数据结构的类型
enum CurContainer {
    Vec1,
    Vec2
}

fn move1<T>(index: usize, unit: usize, factory: &mut SlabFactory<CurContainer, ()>,  vec_from: &mut Vec<(T, usize)>, vec_dst: &mut Vec<(T, usize)>, dst_class: CurContainer) {
    let obj = vec_from.swap_remove(index);
    vec_dst.push(obj);
    factory.store(unit, vec_dst.len() - 1);
    factory.set_class(unit, dst_class);

    // 移除的不是最后一个元素，需要修改最后一个元素的索引(最后一个位置上的元素已经和当前index对应的元素交换了)
    if index < vec_from.len() {
        factory.store(vec_from[index].1, index);
    }
}

#[test]
fn test() {
    main()
}

fn main() {
    let mut a = A::new();

    let entity11 = 1;
    let entity12 = 2;
    let entity13 = 3;
    let entity14 = 4;
    let entity15 = 5;
    let entity16 = 6;

    let entity21 = 7;
    let entity22 = 8;
    let entity23 = 9;
    let entity24 = 10;
    let entity25 = 11;
    let entity26 = 12;

    // 插入元素
    let unit11 = a.inser_to_vec1(entity11);
    let unit12 = a.inser_to_vec1(entity12);
    let unit13 = a.inser_to_vec1(entity13);
    let unit14 = a.inser_to_vec1(entity14);
    let unit15 = a.inser_to_vec1(entity15);
    let unit16 = a.inser_to_vec1(entity16);

    let unit21 = a.inser_to_vec2(entity21);
    let unit22 = a.inser_to_vec2(entity22);
    let unit23 = a.inser_to_vec2(entity23);
    let unit24 = a.inser_to_vec2(entity24);
    let unit25 = a.inser_to_vec2(entity25);
    let unit26 = a.inser_to_vec2(entity26);

    // 移动元素
    a.move_obj(unit11);
    a.move_obj(unit12);
    a.move_obj(unit13);
    a.move_obj(unit14);
    a.move_obj(unit15);
    a.move_obj(unit16);

    a.move_obj(unit21);
    a.move_obj(unit22);
    a.move_obj(unit23);
    a.move_obj(unit24);
    a.move_obj(unit25);
    a.move_obj(unit26);

    // 移动后，依然能正确查询到元素
    assert_eq!(*(a.query(unit11).unwrap()), 1);
    assert_eq!(*(a.query(unit12).unwrap()), 2);
    assert_eq!(*(a.query(unit13).unwrap()), 3);
    assert_eq!(*(a.query(unit14).unwrap()), 4);
    assert_eq!(*(a.query(unit15).unwrap()), 5);
    assert_eq!(*(a.query(unit16).unwrap()), 6);

    assert_eq!(*(a.query(unit21).unwrap()), 7);
    assert_eq!(*(a.query(unit22).unwrap()), 8);
    assert_eq!(*(a.query(unit23).unwrap()), 9);
    assert_eq!(*(a.query(unit24).unwrap()), 10);
    assert_eq!(*(a.query(unit25).unwrap()), 11);
    assert_eq!(*(a.query(unit26).unwrap()), 12);
}
