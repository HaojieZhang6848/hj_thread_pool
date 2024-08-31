use hj_thread_pool::{HjThreadPoolCfg, HjThreadPool, HjThreadPoolLogLevel};

fn main() {
    let pool = HjThreadPool::new(HjThreadPoolCfg {
        num_workers: 2,
        log_level: HjThreadPoolLogLevel::Debug,
    });
    pool.execute(|| {
        for i in 0..10 {
            println!("Task 1: {}", i);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}
