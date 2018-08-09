use std::rc::Rc;
use std::any::Any;
use std::sync::Arc;
use std::collections::HashMap;

use fnv::FnvHashMap;

use atom::Atom;

/*
* 环境
*/
pub trait Env {
    //获取属性
    fn get_attr(&self, key: Atom) -> Option<GenType>;

    //设置属性，返回上个属性值
    fn set_attr(&self, key: Atom, value: GenType) -> Option<GenType>;

    //移除属性，返回属性值
    fn remove_attr(&self, key: Atom) -> Option<GenType>;
}

/*
* 通用处理器
*/
pub trait Handler: Send + Sync {
    type A;
    type B;
    type C;
    type D;
    type E;
    type F;
    type G;
    type H;
    type HandleResult;

    //处理方法
    fn handle(&self, env: Arc<dyn Env>, func: Atom, args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>) -> Self::HandleResult;
}

/*
* 通用Map属性值
*/
#[derive(Debug, Clone)]
pub enum GenMapType {
    U8KeyMap(FnvHashMap<u8, GenType>),
    U16KeyMap(FnvHashMap<u16, GenType>),
    U32KeyMap(FnvHashMap<u32, GenType>),
    U64KeyMap(FnvHashMap<u64, GenType>),
    U128KeyMap(FnvHashMap<u128, GenType>),
    USizeKeyMap(FnvHashMap<usize, GenType>),
    I8KeyMap(FnvHashMap<i8, GenType>),
    I16KeyMap(FnvHashMap<i16, GenType>),
    I32KeyMap(FnvHashMap<i32, GenType>),
    I64KeyMap(FnvHashMap<i64, GenType>),
    I128KeyMap(FnvHashMap<i128, GenType>),
    ISizeKeyMap(FnvHashMap<isize, GenType>),
    StrKeyMap(HashMap<Atom, GenType>),
    BinKeyMap(HashMap<Vec<u8>, GenType>),
    PtrKeyMap(FnvHashMap<*const Any, GenType>),
}

/*
* 通用属性值
*/
#[derive(Debug, Clone)]
pub enum GenType {
    Nil,
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    USize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    ISize(isize),
    F32(f32),
    F64(f64),
    String(Atom),
    Bin(Vec<u8>),
    BoxBin(Box<Vec<u8>>),
    RcBin(Rc<Vec<u8>>),
    ArcBin(Arc<Vec<u8>>),
    PtrBin(*const u8),
    Pointer(*const Any),
    Array(Vec<GenType>),
    Map(GenMapType),
    Obj(HashMap<Atom, GenType>),
}

/*
* 通用参数列表
*/
pub enum Args<A, B, C, D, E, F, G, H> {
    NilArgs,                            //无参
    OneArgs(A),
    TwoArgs(A, B),
    ThreeArgs(A, B, C),
    FourArgs(A, B, C, D),
    FiveArgs(A, B, C, D, E),
    SixArgs(A, B, C, D, E, F),
    SevenArgs(A, B, C, D, E, F, G),
    EightArgs(A, B, C, D, E, F, G, H),
    VarArgs(Vec<Box<Any>>),             //变长参数
}

impl<A, B, C, D, E, F, G, H> Args<A, B, C, D, E, F, G, H> {
    //构建一个指定数量的参数对象
    pub fn with(size: usize) -> Self {
        if size == 0 {
            return Args::NilArgs;
        }
        Args::VarArgs(Vec::with_capacity(size))
    }

    //获取参数列表中参数的数量
    pub fn len(&self) -> usize {
        match self {
            Args::NilArgs => 0,
            Args::OneArgs(_) => 1,
            Args::TwoArgs(_, _) => 2,
            Args::ThreeArgs(_, _, _) => 3,
            Args::FourArgs(_, _, _, _) => 4,
            Args::FiveArgs(_, _, _, _, _) => 5,
            Args::SixArgs(_, _, _, _, _, _) => 6,
            Args::SevenArgs(_, _, _, _, _, _, _) => 7,
            Args::EightArgs(_, _, _, _, _, _, _, _) => 8,
            Args::VarArgs(args) => args.len(),
        }
    }

    //获取变长参数指定位置的参数的只读引用
    pub fn get_ref<T: Any>(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            return None;
        }
        
        match self {
            Args::VarArgs(args) => args[index].downcast_ref(),
            _ => None,
        }
    }

    //获取变长参数指定位置的参数的可写引用
    pub fn get_mut<T: Any>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }

         match self {
            Args::VarArgs(args) => args[index].downcast_mut(),
            _ => None,
         }
    }

    //设置变长参数指定位置的参数
    pub fn set<T: Any>(&mut self, index: usize, arg: T) -> &mut Self {
        if index >= self.len() {
            return self;
        }

        match self {
            Args::VarArgs(args) => args[index] = Box::new(arg) as Box<Any>,
            _ => (),
        }
        self
    }

    //在变长参数列表头取出一个指定类型的参数，返回参数
    pub fn pop<T: Any>(&mut self) -> Option<T> {
        match self {
            Args::VarArgs(args) => {
                match args.pop() {
                    None => None,
                    Some(any) => {
                        match any.downcast::<T>() {
                            Err(_) => None,
                            Ok(arg) => Some(*arg),
                        }
                    },
                }
            },
            _ => None,
        }
    }

    //在变长参数列表尾加入一个指定类型的参数，返回参数列表
    pub fn push<T: Any>(&mut self, arg: T) -> &mut Self {
        match self {
            Args::VarArgs(args) => args.push(Box::new(arg) as Box<Any>),
            _ => (),
        }
        self
    }

    //移除变长参数列表指定位置的参数，返回参数
    pub fn remove<T: Any>(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        match self {
            Args::VarArgs(args) => {
                 match args.remove(index).downcast::<T>() {
                    Err(_) => None,
                    Ok(arg) => Some(*arg),
                }
            },
            _ => None,
        }
    }

    //移除变长参数列表所有的参数, 返回参数数量
    pub fn clear(&mut self) -> usize {
        let len = self.len();
        match self {
            Args::VarArgs(args) => args.clear(),
            _ => (),
        }
        len
    }
}