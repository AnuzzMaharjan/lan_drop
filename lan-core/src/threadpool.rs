use std::sync::OnceLock;

use lan_engine::ThreadPool;

static THREAD_POOL: OnceLock<ThreadPool> = OnceLock::new();

pub fn get_thread_pool() -> &'static ThreadPool {
    THREAD_POOL.get_or_init(|| ThreadPool::new(2))
}