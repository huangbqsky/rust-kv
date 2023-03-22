use crate::thread_pool::ThreadPool;
use crate::Result;
use std::thread;

/// a naive thread pool
pub struct NaiveThreadPool;

// 对于最简单的 NaiveThreadPool，仅仅需要在 spawn 的时候创建一个线程让其执行即可。
impl ThreadPool for NaiveThreadPool {
    /// do nothing
    fn new(_: usize) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(NaiveThreadPool)
    }

    /// create a new thread for each spawned job.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }
}
