use std::sync::OnceLock;

use tokio::sync::Semaphore;

static SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

// 设定最大并发请求数为 3
pub fn get_semaphore() -> &'static Semaphore {
    SEMAPHORE.get_or_init(|| Semaphore::new(3))
}