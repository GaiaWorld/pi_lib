// 当且仅当测试套件运行时，才条件编译 `test` 模块。
#[cfg(test)]
extern crate pi_lib;

use pi_lib::ordmap::Entry;
use pi_lib::ordmap::{OrdMap};
use pi_lib::sbtree::{Tree, new};
use pi_lib::atom::Atom;


#[test]
fn atom() {
	let a = Atom::from("aaa");
	let b = Atom::from(String::from("aaa"));
	assert!(*a == String::from("aaa"));
	assert!(*a == *b);
}

// 需要一个辅助函数
fn show(t: &OrdMap<Tree<usize, usize>>) -> Vec<usize> {
	let mut v = Vec::new();
	{
		let mut f = |e:&Entry<usize, usize>| {v.push(e.0.clone()); v.push(e.1.clone())};
		t.select(None, false, &mut f);
	}
	v
}
#[test]
fn sb_test() {
	let xt:Tree<usize, usize> = new();
	let mut t= OrdMap::new(xt);
	t = t.clone();
	assert!(t.is_empty());
	assert!(t.insert(1, 10));
	let t1 = t.clone();
	assert!(t.ptr_eq(&t1));

	assert!(t.insert(2, 20));

	assert!(t.size() == 2);
	assert!(t.insert(3, 30));
	assert!(t.size() == 3);
	assert!(!(t.insert(3, 30)));
	assert!(!t.is_empty());
	assert!(t.has(&3));
	assert!(t.has(&2));
	assert!(t.has(&1));
	assert!(!t.has(&4));
	assert!(t.update(2, 21, false).is_some());
	assert!(t.update(1, 11, false).is_some());
	assert!(t.update(3, 31, false).is_some());
	assert!(!t.update(40, 40, true).is_some());
	assert!(t.size() == 3);
	assert!(t.insert(40, 40));
	assert!(t.size() == 4);
	assert!(t.get(&2) == Some(&21));
	assert!(t.get(&1) == Some(&11));
	assert!(t.get(&3) == Some(&31));
	assert!(t.get(&40) == Some(&40));
	assert!(t.get(&5) == None);
	assert!((t.min().unwrap()).0 == 1);
	assert!((t.max().unwrap()).0 == 40);
	assert!(t.rank(&1) == 1);
	assert!(t.rank(&2) == 2);
	assert!(t.rank(&3) == 3);
	assert!(t.rank(&40) == 4);
	assert!(t.rank(&30) == -4);
	assert!(t.rank(&50) == -5);
	assert!((t.index(1).unwrap()).0 == 1);
	assert!((t.index(2).unwrap()).0 == 2);
	assert!((t.index(3).unwrap()).0 == 3);
	assert!((t.index(4).unwrap()).0 == 40);
	assert!(show(&t) == vec![1,11,2,21, 3, 31, 40, 40]);
	assert!(t.insert(90, 90));
	assert!(show(&t) == vec![1,11,2,21, 3, 31, 40, 40, 90, 90]);
	assert!(t.insert(80, 80));
	assert!(show(&t) == vec![1,11,2,21, 3, 31, 40, 40, 80, 80, 90, 90]);
	assert!(t.insert(70, 70));
	assert!(show(&t) == vec![1,11,2,21, 3, 31, 40, 40,  70, 70, 80, 80, 90, 90]);
	assert!(t.insert(60, 60));
	assert!(show(&t) == vec![1,11,2,21, 3, 31, 40, 40, 60, 60,  70, 70, 80, 80, 90, 90]);
	assert!(t.insert(50, 50));
	assert!(show(&t) == vec![1,11,2,21, 3, 31, 40, 40, 50, 50, 60, 60,  70, 70, 80, 80, 90, 90]);
	assert!(t.delete(&70, true).unwrap().unwrap() == 70);
	assert!(show(&t) == vec![1,11,2,21, 3, 31, 40, 40, 50, 50, 60, 60, 80, 80, 90, 90]);
	assert!(t.insert(70, 71));
	assert!(show(&t) == vec![1,11,2,21, 3, 31, 40, 40, 50, 50, 60, 60,  70, 71, 80, 80, 90, 90]);
	assert!(t.pop_min(true).unwrap().unwrap().1 == 11);
	assert!(show(&t) == vec![2,21, 3, 31, 40, 40, 50, 50, 60, 60,  70, 71, 80, 80, 90, 90]);
	assert!(t.safe_pop_max(true).unwrap().unwrap().1 == 90);
	assert!(show(&t) == vec![2,21, 3, 31, 40, 40, 50, 50, 60, 60,  70, 71, 80, 80]);
	assert!(t.remove(3, true).unwrap().unwrap().0 == 40);
	assert!(show(&t) == vec![2,21, 3, 31, 50, 50, 60, 60,  70, 71, 80, 80]);
}

#[test]
fn test_handler() {
	use std::any::Any;
	use std::sync::Arc;

	use pi_lib::atom::Atom;
	use pi_lib::handler::{Env, GenType, Handler, Args};

	struct Tmp(u8);

	impl Env for Tmp {
		fn get_attr(&self, key: Atom) -> Option<GenType> {
			None
		}

		fn set_attr(&mut self, key: Atom, value: GenType) -> Option<GenType> {
			None
		}

		fn remove_attr(&mut self, key: Atom) -> Option<GenType> {
			None
		}
	}

	struct Test;

	impl Handler for Test {
		type A = Arc<Vec<u8>>;
		type B = ();
		type C = ();
		type D = ();
		type E = ();
		type F = ();
		type G = ();
		type H = ();
		type HandleResult = ();

		fn handle(&self, env: Arc<dyn Env>, func: Atom, args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>) -> Self::HandleResult {
			return;
		}
	}

	let test = Test;
	test.handle(Arc::new(Tmp(10)), Atom::from(""), Args::OneArgs(Arc::new(Vec::new())));
}
