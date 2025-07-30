use m17_reflector::config::Config;
use clap::Parser;
use log::info;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

use m17_reflector::reflector::Reflector;
use m17_reflector::api;
use m17_reflector::ws;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();

    let config = Config::load_from_file(&cli.config)?;
    info!("Loaded config: {:?}", config);

    let state = Arc::new(Mutex::new(Reflector::new(&config.reflector_name, &config.modules)));

    let (tx, _rx) = broadcast::channel::<String>(100);

    tokio::spawn(run_api_server(state.clone(), tx.clone()));

    m17_reflector::server::run_with_state(&config, state, tx).await?;

    Ok(())
}

async fn run_api_server(
    state: Arc<Mutex<Reflector>>,
    tx: broadcast::Sender<String>,
) {
    use axum::routing::get;
    use std::net::SocketAddr;

    let app = api::create_router(state.clone())
        .route("/ws", get(move |ws| ws::ws_handler(ws, axum::extract::State(state.clone()), tx.subscribe())));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("API server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
