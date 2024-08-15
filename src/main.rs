use std::sync::Arc;
use std::sync::RwLock;

use poem::listener::TcpListener;
use poem::EndpointExt;
use poem::Route;
use poem::Server;
use poem_openapi::OpenApiService;
use task_runner::TaskRunner;
use tokio::runtime::{Builder, Runtime};
use tracing::info;

mod handler;
mod response;
mod task_runner;

pub(crate) struct State {
    calls: u32,
}

impl State {
    pub fn new() -> Self {
        Self { calls: 0 }
    }

    fn increment(&mut self) -> u32 {
        self.calls += 1;
        self.calls
    }
}

fn create_io_runtime() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
}

fn create_background_runtime() -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}

fn main() {
    // Create a channel to communicate between runtimes
    let (tx, mut rx) = tokio::sync::mpsc::channel::<u32>(32);

    // Create the first Tokio runtime for handling I/O-bound tasks.
    let io_runtime = create_io_runtime();

    // Create the second Tokio runtime for background tasks.
    let background_runtime = create_background_runtime();

    // Spawn a background task on the second runtime.
    background_runtime.spawn(async move {
        loop {
            let subscriber = tracing_subscriber::fmt()
                .compact()
                .with_max_level(tracing::Level::DEBUG)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_target(true)
                .finish();
            tracing::subscriber::set_global_default(subscriber);

            let task_runner = TaskRunner::new();

            // Listen for tasks on the channel.
            // I'm using a u32 here, but futures can also be passed between threads if they implement Send
            while let Some(call_no) = rx.recv().await {
                task_runner.run_task(async move {
                    info!("BLOCKING: Received call: {}", call_no);
                    // Sleep blocking to mimic a cpu-bound task
                    std::thread::sleep(std::time::Duration::from_millis(3000));
                    info!("BLOCKING: Done processing: {}", call_no);
                });
            }
        }
    });

    // Use the first runtime to run the server.
    io_runtime.block_on(async {
        let subscriber = tracing_subscriber::fmt()
            .compact()
            .with_max_level(tracing::Level::DEBUG)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_target(true)
            .finish();
        tracing::subscriber::set_global_default(subscriber);

        let task_runner = Arc::new(task_runner::TaskRunner::new());
        let state = Arc::new(RwLock::new(State::new()));

        let api_service = OpenApiService::new(handler::Api, "monolith", "1.0")
            .server("http://localhost:9292/api");
        let ui = api_service.swagger_ui();
        let spec = api_service.spec();
        let route = Route::new()
            .nest("/api", api_service)
            .nest("/", ui)
            .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
            .data(task_runner)
            .data(state)
            .data(tx);

        Server::new(TcpListener::bind("0.0.0.0:9292"))
            .run(route)
            .await;
    });
}
