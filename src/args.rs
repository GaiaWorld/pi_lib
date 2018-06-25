use std::any::Any;

/*
* 通用参数类型
*/
pub struct Args(Vec<Box<Any>>);

impl Args {
    //构建一个指定数量的参数对象
    pub fn new(size: usize) -> Self {
        Args(Vec::with_capacity(size))
    }

    //获取参数列表中参数的数量
    pub fn len(&self) -> usize {
        self.0.len()
    }

    //获取指定位置的参数的只读引用
    pub fn get_ref<T: Any>(&self, index: usize) -> Option<&T> {
        if index >= self.0.len() {
            return None;
        }
        self.0[index].downcast_ref()
    }

    //获取指定位置的参数的可写引用
    pub fn get_mut<T: Any>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.0.len() {
            return None;
        }
        self.0[index].downcast_mut()
    }

    //设置指定位置的参数
    pub fn set<T: Any>(&mut self, index: usize, arg: T) -> &mut Self {
        if index >= self.0.len() {
            return self;
        }
        self.0[index] = Box::new(arg) as Box<Any>;
        self
    }

    //在参数列表头取出一个指定类型的参数，返回参数
    pub fn pop<T: Any>(&mut self) -> Option<T> {
        match self.0.pop() {
            None => None,
            Some(any) => {
                match any.downcast::<T>() {
                    Err(_) => None,
                    Ok(arg) => Some(*arg),
                }
            },
        }
    }

    //在参数列表尾加入一个指定类型的参数，返回参数列表
    pub fn push<T: Any>(&mut self, arg: T) -> &mut Self {
        self.0.push(Box::new(arg) as Box<Any>);
        self
    }

    //移除指定位置的参数，返回参数
    pub fn remove<T: Any>(&mut self, index: usize) -> Option<T> {
        if index >= self.0.len() {
            return None;
        }
        match self.0.remove(index).downcast::<T>() {
            Err(_) => None,
            Ok(arg) => Some(*arg),
        }
    }

    //移除所有的参数, 返回参数数量
    pub fn clear(&mut self) -> usize {
        let len = self.0.len();
        self.0.clear();
        len
    }
}