use crate::thread_pool::ThreadPool;

/// a thread pool wrapping rayon's threadPool
pub struct RayonThreadPool {
    pool: rayon::ThreadPool,
}

// 对于 RayonThreadPool，直接参考官网的样例初始化对应的 pool 并直接 spawn 给其即可。
impl ThreadPool for RayonThreadPool {
    /// init rayon's threadPool
    fn new(num: usize) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(RayonThreadPool {
            pool: rayon::ThreadPoolBuilder::new().num_threads(num).build()?,
        })
    }

    /// spawn job to rayon's threadPool
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.spawn(job);
    }
}