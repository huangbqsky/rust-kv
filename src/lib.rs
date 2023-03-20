#![deny(missing_docs)]
/*!
The KvStore store key/value pairs.
 */
mod command;
mod errors;
mod proto;
mod kv;
mod server;
mod kv_engine;

pub use command::Command;
pub use errors::{KVStoreError, Result};
pub use proto::{Request, Response};
pub use kv::KvStore;
pub use kv_engine::KvsEngine;
