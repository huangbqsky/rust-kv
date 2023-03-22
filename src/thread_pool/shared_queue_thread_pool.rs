use crate::thread_pool::ThreadPool;
use crate::Result;
use log::{debug, error};
use std::panic::AssertUnwindSafe;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::{panic, thread};

/**
 * 共享队列的 ThreadPool
 *  std 库自带的 channel 是 MPSC 类型，因而可以支持并发写但不支持并发读。
    因而要想实现多个子 thread 对 channel 的监听便需要用 Arc 来保证不存在并发读。
    此外也可以使用 crossbeam 的 mpsc channel 来支持并发读，那样便直接 clone 即可
 */
/// a shared queue thread pool
pub struct SharedQueueThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

// 为了优雅停机，对于 Job 又包装了一层枚举和 Terminate 类型来支持子 thread 的优雅退出，
// 此外还需要利用 Box 将闭包 F 放在堆上来支持线程安全的传递闭包。
enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool for SharedQueueThreadPool {
    /// init num threads and related resources
    fn new(num: usize) -> Result<Self>
    where
        Self: Sized,
    {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(num);
        for id in 0..num {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(SharedQueueThreadPool { workers, sender })
    }

    /// spawn the job to pools
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // 利用 Box 将闭包 F 放在堆上来支持线程安全的传递闭包。
        self.sender.send(Message::NewJob(Box::new(job))).unwrap()
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        debug!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        debug!("Shutting down {} workers.", self.workers.len());

        for worker in &mut self.workers {
            debug!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    debug!("{} receive a job", id);
                    // 由于单元测试中传入的闭包可能会 panic 但不想看到线程池中的线程减少，
                    // 一种方案是检测到线程 panic 退出之后新增新的线程，
                    // 另一种方式则是panic::catch_unwind捕获可能得 panic。确保该线程不会由于执行闭包而 panic
                    if let Err(err) = panic::catch_unwind(AssertUnwindSafe(job)) {
                        error!("{} executes a job with error {:?}", id, err);
                    }
                }
                Message::Terminate => {
                    debug!("Worker {} terminated", id);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}