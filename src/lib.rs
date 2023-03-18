#![deny(missing_docs)]
/*!
The KvStore store key/value pairs.
 */
mod command;
mod errors;
mod proto;
mod kv;

pub use command::Command;
pub use errors::{KVStoreError, Result};
pub use proto::{Request, Response};
pub use kv::KvStore;
