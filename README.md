# rust-simple-kv
A Simple KV Engine Written in Rust 🦀



# Example

KvStore实现：
```
use std::env;
use kvs::{KvStore, Result};
use crate::kvs::KvsEngine;

fn try_main() -> Result<()> {
    let mut store = KvStore::open(env::current_dir()?)?;
    store.set("1".to_owned(),"1".to_owned())?;
    assert_eq!(store.get("1".to_owned())?, Some("1".to_owned()));
    store.remove("1".to_owned())?;
    assert_eq!(store.get("1".to_owned())?, None);
    Ok(())
}
```
SledKvsEngine实现：（Sled 是一款基于 Bw 树构建的嵌入式 KV 数据库）
```
use std::env;
use kvs::{SledKvsEngine, Result};
use crate::kvs::KvsEngine;

fn try_main() -> Result<()> {
    let mut store = SledKvsEngine::open(env::current_dir()?)?;
    store.set("1".to_owned(),"1".to_owned())?;
    assert_eq!(store.get("1".to_owned())?, Some("1".to_owned()));
    store.remove("1".to_owned())?;
    assert_eq!(store.get("1".to_owned())?, None);
    Ok(())
}
```

# 测试
-> cargo test

# 参考
Talent-Plan：用 Rust 实现简易 KV 引擎 
https://tanxinyu.work/naive-kvengine-in-rust/
