use crate::response::MyResponse;
use crate::task_runner::TaskRunner;
use crate::State;
use poem::web::Data;
use poem_openapi::OpenApi;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::info;

pub(crate) enum HandlerError {
    Error,
}

impl<T> From<std::sync::PoisonError<T>> for HandlerError {
    fn from(_value: std::sync::PoisonError<T>) -> Self {
        Self::Error
    }
}

pub(crate) struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/example", method = "post")]
    async fn io_bound(
        &self,
        state: Data<&Arc<RwLock<State>>>,
        task_runner: Data<&Arc<TaskRunner>>,
        tx: Data<&tokio::sync::mpsc::Sender<u32>>,
    ) -> MyResponse<u32> {
        let result = {
            state
                .write()
                .map(|mut state| state.increment())
                .map_err(|_| HandlerError::Error)
        };

        match result {
            Ok(call_no) => {
                // for tasks that are non-blocking, run them in the same runtime
                task_runner.run_task(async move {
                    info!("NONBLOCKING: received call: {}", call_no);
                    // nonblocking sleep
                    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
                    info!("NONBLOCKING: Done processing: {}", call_no);
                });

                // For blocking tasks, like heavy cpu-bound operations, pass a message to the dedicated runtime.
                if let Err(_) = tx.send(call_no).await {
                    // Handle the error if sending fails
                    return Err(HandlerError::Error).into();
                }
                // Return the incremented value
                Ok(Some(call_no)).into()
            }
            Err(_) => {
                // Handle the error if locking the state fails
                Err(HandlerError::Error).into()
            }
        }
    }
}
