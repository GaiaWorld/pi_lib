//本地对象
declare var NativeObject: NativeObjectClass;

//本地对象同步返回值类型
type NativeObjectRetType = undefined|boolean|number|string|ArrayBuffer|ArrayBufferView|Error|object;
//本地对象异步返回值类型
type AsyncNativeObjectRetType = Promise<undefined|boolean|number|string|ArrayBuffer|ArrayBufferView|object>;

declare class NativeObjectClass {
    registry: NativeObjectRegistry; //本地对象回收器注册器
    static_call(index: number, ...anyArgs: any[]): NativeObjectRetType; //本地对象静态同步调用
    async_static_call(index: number, ...anyArgs: any[]): AsyncNativeObjectRetType; //本地对象静态异步调用
    call(index: number, self: object, ...anyArgs: any[]): NativeObjectRetType; //本地对象同步调用
    async_call(index: number, self: object, ...anyArgs: any[]): AsyncNativeObjectRetType; //本地对象异步调用
    release(cid: number, self: object): void; //释放指定的本地对象
}

declare class NativeObjectRegistry {
    //注册指定本地对象的回收器
    register(obj: object, args: [object]): void;
}