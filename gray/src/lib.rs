extern crate fnv;

use fnv::FnvHashMap;

pub trait GrayVersion {
    fn get_gray(&self) -> &Option<usize>;
    fn set_gray(&mut self, Option<usize>);
	fn get_id(&self) -> usize;
}

pub trait Gray{}

/**
* 灰度表
*/
pub struct GrayTab<T: Gray>{
    last: usize,
    tab: FnvHashMap<usize, T>,
}

impl<T: Gray> GrayTab<T> {
	//构建一个灰度表
	pub fn new(first: T) -> Self {
		let mut map = FnvHashMap::default();
		map.insert(0, first);
		GrayTab {
            last: 0,
			tab: map,
		}
	}

	//取灰度
	pub fn get(&self, version: &usize) -> Option<&T> {
		self.tab.get(version)
	}

	//添加一个灰度
	pub fn add(&mut self, gray: T) -> usize {
		self.last += 1;
		self.tab.insert(self.last, gray);
		self.last
	}

	//移除灰度, 如果移除后
	pub fn remove(&mut self, version: &usize) -> Option<T> {
		//如果灰度列表中只存在一个灰度，则不能移除
		match self.last{
			0 => {println!("The only gray cannot be removed"); return None;},
			_ => (),
		};

		//如果移除灰度版本未最新版本， 则更新最新版本未前一版本
		match version == &self.last {
			true => self.version_back(),
			false => (),
		}
		self.tab.remove(version)
	}

    //取到最新灰度
	pub fn get_last(&self) -> &T {
		self.tab.get(&self.last).unwrap()
	}

    fn version_back(&mut self){
        self.last = self.last - 1;
        match self.tab.get(&self.last){
            Some(_) => (),
            None => self.version_back()
        }
    }
}
