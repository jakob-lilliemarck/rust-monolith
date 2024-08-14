use std::future;

// Use for long running background tasks
pub(crate) struct TaskRunner {}

impl TaskRunner {
    pub fn new() -> TaskRunner {
        Self {}
    }

    pub fn run_task<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output> where
        F: future::Future + Send + 'static,
        F::Output: Send + 'static {
        tokio::spawn(future)
    }
}