use crate::Result;

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
