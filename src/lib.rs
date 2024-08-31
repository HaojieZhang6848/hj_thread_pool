//! HjThreadPool是一个简单的线程池实现，支持指定线程池中线程数量和日志级别。
//!
//! # Examples
//!
//! ```rust
//! use hj_thread_pool::{HjThreadPoolCfg, HjThreadPool, HjThreadPoolLogLevel};
//!
//! fn main() {
//!     let pool = HjThreadPool::new(HjThreadPoolCfg {
//!         num_workers: 2,
//!         log_level: HjThreadPoolLogLevel::Debug,
//!     });
//!     pool.execute(|| {
//!         for i in 0..10 {
//!             println!("Task 1: {}", i);
//!             std::thread::sleep(std::time::Duration::from_secs(1));
//!         }
//!     });
//! }
//! ```
mod hj_thread_pool;

pub use hj_thread_pool::{HjThreadPool, HjThreadPoolCfg, HjThreadPoolLogLevel};
