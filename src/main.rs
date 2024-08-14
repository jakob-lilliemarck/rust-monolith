use std::sync::Arc;

use poem::listener::TcpListener;
use poem::EndpointExt;
use poem::Route;
use poem::Server;
use poem_openapi::OpenApiService;

mod handler;
mod response;
mod task_runner;

// Use for prioritized tasks
pub(crate) struct PubSub {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let task_runner = Arc::new(task_runner::TaskRunner::new());

    let api_service =
        OpenApiService::new(handler::Api, "monolith", "1.0").server("http://localhost:9292/api");
    let ui = api_service.swagger_ui();
    let spec = api_service.spec();
    let route = Route::new()
        .nest("/api", api_service)
        .nest("/", ui)
        .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
        .data(task_runner); // TODO - pass senders here
    Server::new(TcpListener::bind("0.0.0.0:9292"))
        .run(route)
        .await?;

    Ok(())
}