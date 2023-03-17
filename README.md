# rust-simple-kv
A Simple Rust KV


# 包结构
对于包含一个 lib 包和一个 bin 包的 crate ，在 lib 包中，需要引用所有新增文件的文件名当做其模块名将其引入，此外还需要使用 pub use 语法来将 bin 包会用到的结构公开导出。

在 lib 包的任何文件里，都可以通过 crate:: 的方式来引入本 lib 库被公开导出的结构。

在 bin 包中，需要通过实际 crate 名：: 的方式来引入同名 lib 库被公开导出的结构。


# 参考
Talent-Plan：用 Rust 实现简易 KV 引擎 
https://tanxinyu.work/naive-kvengine-in-rust/


# 测试
-> cargo test