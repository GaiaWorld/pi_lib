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
	register_native_object_static_method(static_call_4);
	register_native_object_static_method(static_call_5);
	register_native_object_static_method(static_call_6);
	register_native_object_static_method(static_call_7);
	register_native_object_static_method(static_call_8);
	register_native_object_static_method(static_call_9);
	register_native_object_static_method(static_call_10);
	register_native_object_static_method(static_call_11);
	register_native_object_static_method(static_call_12);
	register_native_object_static_method(static_call_13);
	register_native_object_static_method(static_call_14);
	register_native_object_static_method(static_call_15);
	register_native_object_static_method(static_call_16);
	register_native_object_static_method(static_call_17);
	register_native_object_static_method(static_call_18);
	register_native_object_static_method(static_call_19);
	register_native_object_static_method(static_call_20);
	register_native_object_static_method(static_call_21);
	register_native_object_static_method(static_call_22);
	register_native_object_static_method(static_call_23);
	register_native_object_static_method(static_call_24);
	register_native_object_static_method(static_call_25);
	register_native_object_static_method(static_call_26);
	register_native_object_static_method(static_call_27);
	register_native_object_static_method(static_call_28);
	register_native_object_static_method(static_call_29);
	register_native_object_static_method(static_call_30);
	register_native_object_static_method(static_call_31);
	register_native_object_static_method(static_call_32);
	register_native_object_static_method(static_call_33);
	register_native_object_static_method(static_call_34);
	register_native_object_static_method(static_call_35);
	register_native_object_static_method(static_call_36);
	register_native_object_static_method(static_call_37);
	register_native_object_static_method(static_call_38);
	register_native_object_static_method(static_call_39);
	register_native_object_static_method(static_call_40);

	//注册异步静态函数和本地对象的异步关联函数
	register_native_object_async_static_method(async_static_call_0);
	register_native_object_async_static_method(async_static_call_1);
	register_native_object_async_static_method(async_static_call_2);
	register_native_object_async_static_method(async_static_call_3);
	register_native_object_async_static_method(async_static_call_4);
	register_native_object_async_static_method(async_static_call_5);
	register_native_object_async_static_method(async_static_call_6);
	register_native_object_async_static_method(async_static_call_7);
	register_native_object_async_static_method(async_static_call_8);
	register_native_object_async_static_method(async_static_call_9);
	register_native_object_async_static_method(async_static_call_10);
	register_native_object_async_static_method(async_static_call_11);
	register_native_object_async_static_method(async_static_call_12);
	register_native_object_async_static_method(async_static_call_13);
	register_native_object_async_static_method(async_static_call_14);
	register_native_object_async_static_method(async_static_call_15);
	register_native_object_async_static_method(async_static_call_16);
	register_native_object_async_static_method(async_static_call_17);
	register_native_object_async_static_method(async_static_call_18);
	register_native_object_async_static_method(async_static_call_19);
	register_native_object_async_static_method(async_static_call_20);
	register_native_object_async_static_method(async_static_call_21);
	register_native_object_async_static_method(async_static_call_22);
	register_native_object_async_static_method(async_static_call_23);
	register_native_object_async_static_method(async_static_call_24);
	register_native_object_async_static_method(async_static_call_25);
	register_native_object_async_static_method(async_static_call_26);
	register_native_object_async_static_method(async_static_call_27);

	//注册本地对象的方法
	register_native_object_method(call_0);
	register_native_object_method(call_1);
	register_native_object_method(call_2);
	register_native_object_method(call_3);
	register_native_object_method(call_4);
	register_native_object_method(call_5);
	register_native_object_method(call_6);
	register_native_object_method(call_7);
	register_native_object_method(call_8);
	register_native_object_method(call_9);
	register_native_object_method(call_10);
	register_native_object_method(call_11);
	register_native_object_method(call_12);
	register_native_object_method(call_13);
	register_native_object_method(call_14);
	register_native_object_method(call_15);
	register_native_object_method(call_16);
	register_native_object_method(call_17);
	register_native_object_method(call_18);
	register_native_object_method(call_19);
	register_native_object_method(call_20);
	register_native_object_method(call_21);
	register_native_object_method(call_22);
	register_native_object_method(call_23);
	register_native_object_method(call_24);
	register_native_object_method(call_25);
	register_native_object_method(call_26);
	register_native_object_method(call_27);
	register_native_object_method(call_28);
	register_native_object_method(call_29);
	register_native_object_method(call_30);
	register_native_object_method(call_31);
	register_native_object_method(call_32);
	register_native_object_method(call_33);
	register_native_object_method(call_34);
	register_native_object_method(call_35);
	register_native_object_method(call_36);
	register_native_object_method(call_37);
	register_native_object_method(call_38);
	register_native_object_method(call_39);
	register_native_object_method(call_40);
	register_native_object_method(call_41);
	register_native_object_method(call_42);
	register_native_object_method(call_43);
	register_native_object_method(call_44);

	//注册本地对象的异步方法
	register_native_object_async_method(async_call_0);
	register_native_object_async_method(async_call_1);
	register_native_object_async_method(async_call_2);
	register_native_object_async_method(async_call_3);
	register_native_object_async_method(async_call_4);
	register_native_object_async_method(async_call_5);
	register_native_object_async_method(async_call_6);
	register_native_object_async_method(async_call_7);
	register_native_object_async_method(async_call_8);
	register_native_object_async_method(async_call_9);
	register_native_object_async_method(async_call_10);
	register_native_object_async_method(async_call_11);
	register_native_object_async_method(async_call_12);
	register_native_object_async_method(async_call_13);
	register_native_object_async_method(async_call_14);
	register_native_object_async_method(async_call_15);
	register_native_object_async_method(async_call_16);
	register_native_object_async_method(async_call_17);
	register_native_object_async_method(async_call_18);
	register_native_object_async_method(async_call_19);
	register_native_object_async_method(async_call_20);
	register_native_object_async_method(async_call_21);
	register_native_object_async_method(async_call_22);
	register_native_object_async_method(async_call_23);
	register_native_object_async_method(async_call_24);
	register_native_object_async_method(async_call_25);
	register_native_object_async_method(async_call_26);
	register_native_object_async_method(async_call_27);
	register_native_object_async_method(async_call_28);
	register_native_object_async_method(async_call_29);
	register_native_object_async_method(async_call_30);
	register_native_object_async_method(async_call_31);
	register_native_object_async_method(async_call_32);
	register_native_object_async_method(async_call_33);
	register_native_object_async_method(async_call_34);
	register_native_object_async_method(async_call_35);
	register_native_object_async_method(async_call_36);
	register_native_object_async_method(async_call_37);
	register_native_object_async_method(async_call_38);
	register_native_object_async_method(async_call_39);
	register_native_object_async_method(async_call_40);
	register_native_object_async_method(async_call_41);
	register_native_object_async_method(async_call_42);
	register_native_object_async_method(async_call_43);
	register_native_object_async_method(async_call_44);
	register_native_object_async_method(async_call_45);
	register_native_object_async_method(async_call_46);
	register_native_object_async_method(async_call_47);
	register_native_object_async_method(async_call_48);
	register_native_object_async_method(async_call_49);
	register_native_object_async_method(async_call_50);
	register_native_object_async_method(async_call_51);
	register_native_object_async_method(async_call_52);
	register_native_object_async_method(async_call_53);
}
