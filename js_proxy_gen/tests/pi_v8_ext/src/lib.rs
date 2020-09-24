use vm_builtin::external::{register_native_object_static_method,
							register_native_object_async_static_method,
							register_native_object_method,
							register_native_object_async_method};

mod export_crate_test_inner;

use export_crate_test_inner::*;

/**
 * 注册所有自动导入的外部扩展库中声明的导出函数
 */
pub fn register_ext_functions() {
	//注册静态函数和本地对象的关联函数
	register_native_object_static_method(static_call_0);
	register_native_object_static_method(static_call_1);
	register_native_object_static_method(static_call_2);
	register_native_object_static_method(static_call_3);

	//注册异步静态函数和本地对象的异步关联函数
	register_native_object_async_static_method(async_static_call_0);

	//注册本地对象的方法
	register_native_object_method(call_0);
	register_native_object_method(call_1);
	register_native_object_method(call_2);
	register_native_object_method(call_3);
	register_native_object_method(call_4);
	register_native_object_method(call_5);
	register_native_object_method(call_6);

	//注册本地对象的异步方法
	register_native_object_async_method(async_call_0);
	register_native_object_async_method(async_call_1);
}

