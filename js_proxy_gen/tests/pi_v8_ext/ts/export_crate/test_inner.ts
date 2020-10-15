import { NativeObject } from '../native_env';

/**
 * 布尔值
 */
export const BOOL: boolean = true;

/**
 * 32位无符号整数
 */
export const UINT: number = 4294967295;

/**
 * 32位有符号整数
 */
export const INT: number = -999999999;

/**
 * 浮点数
 */
export const FLOAT: number = 0.000000001;

/**
 * 字符串
 */
export const STRING: string = ".\\tests\\test.rs";

/**
 * 发送消息
 */
export function send(_0: boolean, _1: number, _2: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(0, _0, _1, _2) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_bool_bool(x: boolean, y: boolean, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(1, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_bool_usize(x: boolean, y: boolean, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(2, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_bool_String(x: boolean, y: boolean, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(3, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_usize_bool(x: boolean, y: number, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(4, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_usize_usize(x: boolean, y: number, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(5, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_usize_String(x: boolean, y: number, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(6, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_String_bool(x: boolean, y: string, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(7, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_String_usize(x: boolean, y: string, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(8, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_bool_String_String(x: boolean, y: string, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(9, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_bool_bool(x: number, y: boolean, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(10, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_bool_usize(x: number, y: boolean, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(11, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_bool_String(x: number, y: boolean, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(12, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_usize_bool(x: number, y: number, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(13, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_usize_usize(x: number, y: number, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(14, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_usize_String(x: number, y: number, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(15, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_String_bool(x: number, y: string, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(16, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_String_usize(x: number, y: string, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(17, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_usize_String_String(x: number, y: string, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(18, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_bool_bool(x: string, y: boolean, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(19, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_bool_usize(x: string, y: boolean, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(20, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_bool_String(x: string, y: boolean, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(21, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_usize_bool(x: string, y: number, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(22, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_usize_usize(x: string, y: number, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(23, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_usize_String(x: string, y: number, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(24, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_String_bool(x: string, y: string, _z: boolean): ArrayBuffer|Error {
	let result = NativeObject.static_call(25, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_String_usize(x: string, y: string, _z: number): ArrayBuffer|Error {
	let result = NativeObject.static_call(26, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 获取数据
 */
export function get_data_String_String_String(x: string, y: string, _z: string): ArrayBuffer|Error {
	let result = NativeObject.static_call(27, x, y, _z) as ArrayBuffer;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_bool_bool(x: boolean, y: boolean, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(0, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_bool_usize(x: boolean, y: boolean, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(1, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_bool_String(x: boolean, y: boolean, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(2, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_usize_bool(x: boolean, y: number, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(3, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_usize_usize(x: boolean, y: number, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(4, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_usize_String(x: boolean, y: number, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(5, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_String_bool(x: boolean, y: string, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(6, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_String_usize(x: boolean, y: string, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(7, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_bool_String_String(x: boolean, y: string, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(8, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_bool_bool(x: number, y: boolean, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(9, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_bool_usize(x: number, y: boolean, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(10, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_bool_String(x: number, y: boolean, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(11, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_usize_bool(x: number, y: number, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(12, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_usize_usize(x: number, y: number, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(13, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_usize_String(x: number, y: number, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(14, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_String_bool(x: number, y: string, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(15, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_String_usize(x: number, y: string, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(16, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_usize_String_String(x: number, y: string, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(17, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_bool_bool(x: string, y: boolean, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(18, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_bool_usize(x: string, y: boolean, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(19, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_bool_String(x: string, y: boolean, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(20, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_usize_bool(x: string, y: number, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(21, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_usize_usize(x: string, y: number, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(22, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_usize_String(x: string, y: number, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(23, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_String_bool(x: string, y: string, _z: boolean): Promise<object> {
	let result = NativeObject.async_static_call(24, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_String_usize(x: string, y: string, _z: number): Promise<object> {
	let result = NativeObject.async_static_call(25, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 通知
 */
export async function notify_String_String_String(x: string, y: string, _z: string): Promise<object> {
	let result = NativeObject.async_static_call(26, x, y, _z) as Promise<object>;
	return result;
}

/**
 * 测试用结构体
 */
export class TestStruct_bool_u8_ {
	/**
	 * 布尔值
	 */
	static readonly BOOL1: boolean = true;

	/**
	 * 32位无符号整数
	 */
	static readonly UINT1: number = 4294967295;

	/**
	 * 32位有符号整数
	 */
	static readonly INT1: number = -999999999;

	/**
	 * 浮点数
	 */
	static readonly FLOAT1: number = 1.1231658798;

	/**
	 * 字符串
	 */
	static readonly STRING1: string = ".\\tests\\test.rs";

	/**
	 * 本地对象
	 */
	private self: object;

	/**
	 * 类的私有构造方法
	 */
	private constructor(self: object) {
		this.self = self;
		NativeObject.registry.register(self, [self]);
	}

	/**
	 * 释放本地对象的方法
	 */
	public destory() {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ already destory");
		}

		this.self = undefined;
		NativeObject.relese(this.self);
	}

	/**
	 * 释放测试用结构体
	 */
	public drop(): void {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		NativeObject.call(0, this.self);
	}

	/**
	 * 复制测试用结构体
	 */
	public clone(): object {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(1, this.self) as object;
		return result;
	}

	/**
	 * 构建测试用结构体
	 */
	static new(x: boolean, y: boolean, z: boolean, vec: ArrayBuffer): TestStruct_bool_u8_ {
		let result = NativeObject.static_call(28, x, y, z, vec) as object;
		if(result instanceof Error) {
			throw result;
		} else if(result instanceof Object) {
			return new TestStruct_bool_u8_(result);
		} else {
			return;
		}
	}

	/**
	 * 获取x的只读引用
	 */
	public get_x(): boolean {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(2, this.self) as boolean;
		return result;
	}

	/**
	 * 设置x的只读引用
	 */
	public set_x(x: boolean): void {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		NativeObject.call(3, this.self, x);
	}

	/**
	 * 设置指定类型的值
	 */
	static set_bool(x: boolean): boolean|undefined {
		let result = NativeObject.static_call(29, x) as boolean;
		return result;
	}

	/**
	 * 设置指定类型的值
	 */
	static set_usize(x: number): number|undefined {
		let result = NativeObject.static_call(30, x) as number;
		return result;
	}

	/**
	 * 设置指定类型的值
	 */
	static set_String(x: string): string|undefined {
		let result = NativeObject.static_call(31, x) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(4, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_usize(x: boolean, y: boolean, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(5, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_String(x: boolean, y: boolean, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(6, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_bool(x: number, y: number, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(7, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_usize(x: number, y: number, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(8, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_String(x: number, y: number, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(9, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_bool(x: string, y: string, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(10, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_usize(x: string, y: string, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(11, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_String(x: string, y: string, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.call(12, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(0, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_usize(x: boolean, y: boolean, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(1, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_String(x: boolean, y: boolean, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(2, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_bool(x: number, y: number, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(3, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_usize(x: number, y: number, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(4, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_String(x: number, y: number, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(5, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_bool(x: string, y: string, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(6, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_usize(x: string, y: string, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(7, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_String(x: string, y: string, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(8, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

}

/**
 * 测试用结构体
 */
export class TestStruct_usize_u8_ {
	/**
	 * 布尔值
	 */
	static readonly BOOL1: boolean = true;

	/**
	 * 32位无符号整数
	 */
	static readonly UINT1: number = 4294967295;

	/**
	 * 32位有符号整数
	 */
	static readonly INT1: number = -999999999;

	/**
	 * 浮点数
	 */
	static readonly FLOAT1: number = 1.1231658798;

	/**
	 * 字符串
	 */
	static readonly STRING1: string = ".\\tests\\test.rs";

	/**
	 * 本地对象
	 */
	private self: object;

	/**
	 * 类的私有构造方法
	 */
	private constructor(self: object) {
		this.self = self;
		NativeObject.registry.register(self, [self]);
	}

	/**
	 * 释放本地对象的方法
	 */
	public destory() {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ already destory");
		}

		this.self = undefined;
		NativeObject.relese(this.self);
	}

	/**
	 * 释放测试用结构体
	 */
	public drop(): void {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		NativeObject.call(13, this.self);
	}

	/**
	 * 复制测试用结构体
	 */
	public clone(): object {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(14, this.self) as object;
		return result;
	}

	/**
	 * 构建测试用结构体
	 */
	static new(x: number, y: number, z: number, vec: ArrayBuffer): TestStruct_usize_u8_ {
		let result = NativeObject.static_call(32, x, y, z, vec) as object;
		if(result instanceof Error) {
			throw result;
		} else if(result instanceof Object) {
			return new TestStruct_usize_u8_(result);
		} else {
			return;
		}
	}

	/**
	 * 获取x的只读引用
	 */
	public get_x(): number {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(15, this.self) as number;
		return result;
	}

	/**
	 * 设置x的只读引用
	 */
	public set_x(x: number): void {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		NativeObject.call(16, this.self, x);
	}

	/**
	 * 设置指定类型的值
	 */
	static set_bool(x: boolean): boolean|undefined {
		let result = NativeObject.static_call(33, x) as boolean;
		return result;
	}

	/**
	 * 设置指定类型的值
	 */
	static set_usize(x: number): number|undefined {
		let result = NativeObject.static_call(34, x) as number;
		return result;
	}

	/**
	 * 设置指定类型的值
	 */
	static set_String(x: string): string|undefined {
		let result = NativeObject.static_call(35, x) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(17, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_usize(x: boolean, y: boolean, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(18, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_String(x: boolean, y: boolean, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(19, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_bool(x: number, y: number, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(20, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_usize(x: number, y: number, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(21, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_String(x: number, y: number, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(22, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_bool(x: string, y: string, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(23, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_usize(x: string, y: string, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(24, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_String(x: string, y: string, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.call(25, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(9, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_usize(x: boolean, y: boolean, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(10, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_String(x: boolean, y: boolean, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(11, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_bool(x: number, y: number, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(12, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_usize(x: number, y: number, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(13, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_String(x: number, y: number, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(14, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_bool(x: string, y: string, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(15, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_usize(x: string, y: string, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(16, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_String(x: string, y: string, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(17, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

}

/**
 * 测试用结构体
 */
export class TestStruct_String_u8_ {
	/**
	 * 布尔值
	 */
	static readonly BOOL1: boolean = true;

	/**
	 * 32位无符号整数
	 */
	static readonly UINT1: number = 4294967295;

	/**
	 * 32位有符号整数
	 */
	static readonly INT1: number = -999999999;

	/**
	 * 浮点数
	 */
	static readonly FLOAT1: number = 1.1231658798;

	/**
	 * 字符串
	 */
	static readonly STRING1: string = ".\\tests\\test.rs";

	/**
	 * 本地对象
	 */
	private self: object;

	/**
	 * 类的私有构造方法
	 */
	private constructor(self: object) {
		this.self = self;
		NativeObject.registry.register(self, [self]);
	}

	/**
	 * 释放本地对象的方法
	 */
	public destory() {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ already destory");
		}

		this.self = undefined;
		NativeObject.relese(this.self);
	}

	/**
	 * 释放测试用结构体
	 */
	public drop(): void {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		NativeObject.call(26, this.self);
	}

	/**
	 * 复制测试用结构体
	 */
	public clone(): object {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(27, this.self) as object;
		return result;
	}

	/**
	 * 构建测试用结构体
	 */
	static new(x: string, y: string, z: string, vec: ArrayBuffer): TestStruct_String_u8_ {
		let result = NativeObject.static_call(36, x, y, z, vec) as object;
		if(result instanceof Error) {
			throw result;
		} else if(result instanceof Object) {
			return new TestStruct_String_u8_(result);
		} else {
			return;
		}
	}

	/**
	 * 获取x的只读引用
	 */
	public get_x(): string {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(28, this.self) as string;
		return result;
	}

	/**
	 * 设置x的只读引用
	 */
	public set_x(x: string): void {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		NativeObject.call(29, this.self, x);
	}

	/**
	 * 设置指定类型的值
	 */
	static set_bool(x: boolean): boolean|undefined {
		let result = NativeObject.static_call(37, x) as boolean;
		return result;
	}

	/**
	 * 设置指定类型的值
	 */
	static set_usize(x: number): number|undefined {
		let result = NativeObject.static_call(38, x) as number;
		return result;
	}

	/**
	 * 设置指定类型的值
	 */
	static set_String(x: string): string|undefined {
		let result = NativeObject.static_call(39, x) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(30, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_usize(x: boolean, y: boolean, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(31, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_bool_String(x: boolean, y: boolean, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(32, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_bool(x: number, y: number, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(33, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_usize(x: number, y: number, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(34, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_usize_String(x: number, y: number, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(35, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_bool(x: string, y: string, z: boolean, _c: object, _4: object): boolean|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(36, this.self, x, y, z, _c, _4) as boolean;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_usize(x: string, y: string, z: number, _c: object, _4: object): number|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(37, this.self, x, y, z, _c, _4) as number;
		return result;
	}

	/**
	 * 刷新指定类型的值
	 */
	public flush_String_String(x: string, y: string, z: string, _c: object, _4: object): string|Error {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.call(38, this.self, x, y, z, _c, _4) as string;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(18, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_usize(x: boolean, y: boolean, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(19, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_String(x: boolean, y: boolean, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(20, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_bool(x: number, y: number, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(21, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_usize(x: number, y: number, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(22, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_String(x: number, y: number, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(23, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_bool(x: string, y: string, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(24, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_usize(x: string, y: string, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(25, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_String(x: string, y: string, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestStruct_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(26, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

}

/**
 * 测试用枚举
 */
export class TestEnum_bool_u8_ {
	/**
	 * 本地对象
	 */
	private self: object;

	/**
	 * 类的私有构造方法
	 */
	private constructor(self: object) {
		this.self = self;
		NativeObject.registry.register(self, [self]);
	}

	/**
	 * 释放本地对象的方法
	 */
	public destory() {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ already destory");
		}

		this.self = undefined;
		NativeObject.relese(this.self);
	}

	/**
	 * 释放测试用枚举
	 */
	public drop(): void {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		NativeObject.call(39, this.self);
	}

	/**
	 * 复制测试用枚举
	 */
	public clone(): object {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.call(40, this.self) as object;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(27, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_usize(x: boolean, y: boolean, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(28, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_String(x: boolean, y: boolean, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(29, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_bool(x: number, y: number, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(30, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_usize(x: number, y: number, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(31, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_String(x: number, y: number, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(32, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_bool(x: string, y: string, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(33, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_usize(x: string, y: string, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(34, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_String(x: string, y: string, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_bool_u8_ object already destory");
		}

		let result = NativeObject.async_call(35, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

}

/**
 * 测试用枚举
 */
export class TestEnum_usize_u8_ {
	/**
	 * 本地对象
	 */
	private self: object;

	/**
	 * 类的私有构造方法
	 */
	private constructor(self: object) {
		this.self = self;
		NativeObject.registry.register(self, [self]);
	}

	/**
	 * 释放本地对象的方法
	 */
	public destory() {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ already destory");
		}

		this.self = undefined;
		NativeObject.relese(this.self);
	}

	/**
	 * 释放测试用枚举
	 */
	public drop(): void {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		NativeObject.call(41, this.self);
	}

	/**
	 * 复制测试用枚举
	 */
	public clone(): object {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.call(42, this.self) as object;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(36, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_usize(x: boolean, y: boolean, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(37, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_String(x: boolean, y: boolean, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(38, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_bool(x: number, y: number, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(39, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_usize(x: number, y: number, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(40, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_String(x: number, y: number, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(41, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_bool(x: string, y: string, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(42, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_usize(x: string, y: string, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(43, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_String(x: string, y: string, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_usize_u8_ object already destory");
		}

		let result = NativeObject.async_call(44, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

}

/**
 * 测试用枚举
 */
export class TestEnum_String_u8_ {
	/**
	 * 本地对象
	 */
	private self: object;

	/**
	 * 类的私有构造方法
	 */
	private constructor(self: object) {
		this.self = self;
		NativeObject.registry.register(self, [self]);
	}

	/**
	 * 释放本地对象的方法
	 */
	public destory() {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ already destory");
		}

		this.self = undefined;
		NativeObject.relese(this.self);
	}

	/**
	 * 释放测试用枚举
	 */
	public drop(): void {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		NativeObject.call(43, this.self);
	}

	/**
	 * 复制测试用枚举
	 */
	public clone(): object {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.call(44, this.self) as object;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_bool(x: boolean, y: boolean, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(45, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_usize(x: boolean, y: boolean, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(46, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_bool_String(x: boolean, y: boolean, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(47, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_bool(x: number, y: number, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(48, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_usize(x: number, y: number, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(49, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_usize_String(x: number, y: number, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(50, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_bool(x: string, y: string, z: boolean, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(51, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_usize(x: string, y: string, z: number, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(52, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

	/**
	 * 同步指定类型的值
	 */
	public async sync_String_String(x: string, y: string, z: string, _c: object, _r: object): Promise<object> {
		if(this.self == undefined) {
			throw new Error("TestEnum_String_u8_ object already destory");
		}

		let result = NativeObject.async_call(53, this.self, x, y, z, _c, _r) as Promise<object>;
		return result;
	}

}

/**
 * 测试用简单结构体
 */
export class TestSimpleStruct_HashMap_usize_Arc_$u8$___ {
	/**
	 * 本地对象
	 */
	private self: object;

	/**
	 * 类的私有构造方法
	 */
	private constructor(self: object) {
		this.self = self;
		NativeObject.registry.register(self, [self]);
	}

	/**
	 * 释放本地对象的方法
	 */
	public destory() {
		if(this.self == undefined) {
			throw new Error("TestSimpleStruct_HashMap_usize_Arc_$u8$___ already destory");
		}

		this.self = undefined;
		NativeObject.relese(this.self);
	}

	/**
	 * 构造测试用简单结构体
	 */
	static new_HashMap_bool_Vec_u8__(inner: object, x: object): TestSimpleStruct_HashMap_usize_Arc_$u8$___ {
		let result = NativeObject.static_call(40, inner, x) as object;
		if(result instanceof Error) {
			throw result;
		} else if(result instanceof Object) {
			return new TestSimpleStruct_HashMap_usize_Arc_$u8$___(result);
		} else {
			return;
		}
	}

	/**
	 * 异步构造测试用简单结构体
	 */
	static async async_new_HashMap_String_Box_$u8$__(inner: object, x: object): Promise<TestSimpleStruct_HashMap_usize_Arc_$u8$___> {
		let result = NativeObject.async_static_call(27, inner, x) as Promise<object>;
		let r: object = await result;
		if(r instanceof Error) {
			throw r;
		} else if(r instanceof Object) {
			return new TestSimpleStruct_HashMap_usize_Arc_$u8$___(r);
		} else {
			return;
		}
	}

}

