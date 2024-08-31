use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use tklog::{debugs, sync::Logger, LEVEL};

type Job = Box<dyn FnOnce() + Send + 'static>;

/// HjThreadPool是一个简单的线程池实现，支持指定线程池中线程数量和日志级别。
pub struct HjThreadPool {
    workers: Vec<Worker>,
    job_sender: Option<Sender<Job>>,
    logger: Arc<Mutex<Logger>>,
}

/// HjThreadPoolCfg是HjThreadPool的配置结构体，用于指定线程池中线程数量和日志级别。
pub struct HjThreadPoolCfg {
    pub num_workers: usize,              // 线程池中线程数量
    pub log_level: HjThreadPoolLogLevel, // 日志级别
}

/// HjThreadPoolLogLevel是HjThreadPool的日志级别枚举。
pub enum HjThreadPoolLogLevel {
    Debug, // 调试级别
    Info,  // 信息级别
    Warn,  // 警告级别
    Error, // 错误级别
}

struct Worker {
    worker_id: usize,
    job_receiver: Arc<Mutex<Receiver<Job>>>,
    join_handle: Option<JoinHandle<()>>,
    started: bool,
    start_lock: Mutex<()>,
    logger: Arc<Mutex<Logger>>,
}

impl Worker {
    fn new(
        worker_id: usize,
        job_receiver: Arc<Mutex<Receiver<Job>>>,
        logger: Arc<Mutex<Logger>>,
    ) -> Self {
        return Worker {
            worker_id: worker_id,
            job_receiver: job_receiver,
            join_handle: None,
            started: false,
            start_lock: Mutex::new(()),
            logger: logger,
        };
    }

    fn start(&mut self) {
        // 确保一个 Worker 的 start 方法只会被调用一次
        {
            let _lock = self.start_lock.lock().unwrap();
            if self.started {
                panic!("Worker {} already started", self.worker_id);
            }
            self.started = true;
        }

        let job_receiver = Arc::clone(&self.job_receiver);
        let worker_id = self.worker_id;
        let mut logger = Arc::clone(&self.logger);
        let join_handle = thread::spawn(move || loop {
            // 接受主线程发送过来的任务，如果接受失败，则说明线程池已经关闭，则Worker也需要退出
            let recv_job_result = job_receiver.lock().unwrap().recv();
            if recv_job_result.is_err() {
                debugs!(&mut logger, format!("Worker {} exiting", worker_id));
                break;
            }
            let job = recv_job_result.unwrap();
            job();
        });
        self.join_handle = Some(join_handle);
    }
}

impl HjThreadPool {
    /// 创建一个新的 HjThreadPool 实例。
    pub fn new(cfg: HjThreadPoolCfg) -> Self {
        let num_workers = cfg.num_workers;
        let log_level = match cfg.log_level {
            HjThreadPoolLogLevel::Debug => LEVEL::Debug,
            HjThreadPoolLogLevel::Info => LEVEL::Info,
            HjThreadPoolLogLevel::Warn => LEVEL::Warn,
            HjThreadPoolLogLevel::Error => LEVEL::Error,
        };

        let mut logger = Arc::new(Mutex::new(Logger::new()));
        logger.lock().unwrap().set_level(log_level);

        debugs!(
            &mut logger,
            format!("Creating HjThreadPool with {} workers", num_workers)
        );

        let (job_sender, job_receiver) = std::sync::mpsc::channel();
        let job_receiver = Arc::new(Mutex::new(job_receiver));

        let mut workers = Vec::new();
        for worker_id in 0..num_workers {
            let mut worker = Worker::new(worker_id, Arc::clone(&job_receiver), Arc::clone(&logger));
            worker.start();
            workers.push(worker);
        }

        return HjThreadPool {
            workers: workers,
            job_sender: Some(job_sender),
            logger: logger,
        };
    }

    /// 向线程池中提交一个任务。
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.job_sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for HjThreadPool {
    /// 线程池销毁时，需要关闭所有 Worker 线程。
    /// 这里通过关闭Sender来通知所有 Worker 线程退出。Sender关闭后，Worker线程再次尝试获取任务时，会返回错误，从而退出循环。从而实现优雅退出。
    fn drop(&mut self) {
        debugs!(&mut self.logger, "Dropping HjThreadPool");
        drop(self.job_sender.take());
        for worker in self.workers.iter_mut() {
            worker.join_handle.take().unwrap().join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn new() {
        let _pool = HjThreadPool::new(HjThreadPoolCfg {
            num_workers: 2,
            log_level: HjThreadPoolLogLevel::Debug,
        });
    }

    #[test]
    fn test_execute() {
        let pool = HjThreadPool::new(HjThreadPoolCfg {
            num_workers: 2,
            log_level: HjThreadPoolLogLevel::Debug,
        });
        pool.execute(|| {
            for i in 0..10 {
                println!("Task 1: {}", i);
                thread::sleep(Duration::from_secs(1));
            }
        });
        pool.execute(|| {
            for i in 0..10 {
                println!("Task 2: {}", i);
                thread::sleep(Duration::from_secs(1));
            }
        });
        pool.execute(|| {
            for i in 0..10 {
                println!("Task 3: {}", i);
                thread::sleep(Duration::from_secs(1));
            }
        });
    }
}
