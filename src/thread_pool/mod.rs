use crate::Result;

mod naive_thread_pool;
mod shared_queue_thread_pool;

pub use naive_thread_pool::NaiveThreadPool;
pub use shared_queue_thread_pool::SharedQueueThreadPool;

/**
 * 为了多线程需要抽象出线程池的概念，
 * ThreadPool trait 定义如下：
 * spawn 函数中的闭包 F 
 * 1. 不仅需要满足 FnOnce() 的 bound 来满足近执行一次的语义，
 * 2. 还要实现 Send + ‘static 的 bound 来实现线程安全的发送接收和足够长的生命周期。
 */
/// a pool which use multi thread to execute tasks
pub trait ThreadPool {
    /// Creates a new thread pool, immediately spawning the specified number of threads.
    /// Returns an error if any thread fails to spawn. All previously-spawned threads are terminated.
    fn new(threads: usize) -> Result<Self>
    where
        Self: Sized;

    /// Spawn a function into the threadPool.
    /// Spawning always succeeds, but if the function panics the threadPool continues to operate with the same number of threads — the thread count is not reduced nor is the thread pool destroyed, corrupted or invalidated.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}
