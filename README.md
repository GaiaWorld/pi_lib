# pi_lib 简介

## 概述

pi_lib，是用rust实现的基础库，包括常用的数据结构，和实用功能

## 数据结构

这里的数据结构大多数基于Slab

* atom 字符串原子
   + 相同的字符串内容，可以变成Atom，快速查找和计算hash
* cowlist 写时复制的链表
   + TODO：子总准备用slab重写
* data_view，二进制数据的视图
   + 可以从二进制中读取或者往二进制写入特定的类型，u8，u16...
* dense_vec: 貌似没啥用？
   + ? 可不可以移除
* deque：可以用中间快速删除和插入的双端队列
   + 插入和删除的复杂度是O(1)
* fx_hashmap 用了快速hash的fnv-hashmap
* hashmap
   + ? 不知道为啥又有一个hashmap
* heap 堆
   + simple_heap 常规的堆操作
   + slab_heap 允许从中间节点删除和插入的堆
* map 提供了map的trait
   + VecMap：键是usize的基于slab的map
* ordmap 提供了顺序的map
* slab 管理同类型数据的数据结构，基于vec和bool-vec
* wtree 权重树，用于权重随机数的数据结构
   
## ECS 相关

* ecs
* ecs_derive
* dirty
   + 目前用于ecs库，一个可以设置脏，查询脏的容器

## 主要用于后端的库

* apm ?
* file 文件读写的接口
* future 异步封装的结构
* handler 通用的事件处理
* ifstack ?
* gray ?
* guid 基于时间的全局唯一id
* task-pool 任务池
* timer 基于wheel的定时轮 实现的 定时器
* wheel，定时轮的通用版本，目前仅用于定时器的实现
* worker 线程的封装

## 其他

* adler32
   + 增量的CRC32算法
* base58 编码库
* any：将Rc/Arc<trait object> 向下类型转换的库
* bon，sinfo：序列化，反序列化
   + sinfo，是将数据的结构信息序列化反序列化
   + bon，处理数据本身
* compress：lz4压缩和解压
* debug_info: 提供仅用于debug的println!
* dyn_unit 定义了一个分配id的工厂
   + ? 需要和别的库一起使用。
* enum_default_macro 为枚举定义了Default trait的宏
* listener 监听器
* pointer 
   + ? 暂时不知道啥意义
* rsync 数据同步更新算法
* share rc和arc的统一封装
* time 关于时间的工具函数
* ucd unicode快速查询的函数
   + 比如可以查询某个point是不是中文。
* util 实用库
   + ? 具体是哪些功能，待看。


## 测试

* tests 集成测试，估计很长时间不会有用例。
