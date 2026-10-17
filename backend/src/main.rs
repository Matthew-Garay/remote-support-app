//! Main entry point para el backend de Remote Support App.
//! Ejecuta el Signal Server, STUN y TURN.

mod signal_server;
mod stun_server;
mod turn_server;

use actix_web::{web, App, HttpServer};
use tracing_subscriber::fmt::format::Pretty;
use dotenvy::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::fmt::env_filter()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    let signal_server = signal_server::SignalServer::new();
    let turn_server = turn_server::TurnServer::new();

    // Start Signal Server on port 8443 (or configured)
    let signal_port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8443".to_string())
        .parse()
        .unwrap_or(8443);

    let ws_port: u16 = std::env::var("SIGNAL_WS_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    tracing::info!("Iniciando Signal Server en puerto {}", signal_port);
    tracing::info!("Iniciando WebSocket en puerto {}", ws_port);

    // Ejecutar servidores concurrentes
    let signal_handle = actix_rt::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .route("/signal", web::get().to(signal_server.ws_handler))
                .route("/health", web::get().to(|| async { "OK" }))
        })
        .bind(("0.0.0.0", signal_port))?
        .run()
        .await
    });

    let ws_handle = actix_rt::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .route("/ws", web::get().to(signal_server.ws_handler))
        })
        .bind(("0.0.0.0", ws_port))?
        .run()
        .await
    });

    let _turn_handle = actix_rt::spawn(async move {
        tracing::info!("TURN Server placeholder (use Coturn para producción)");
        // TURN se ejecuta externamente vía Docker Compose (coturn)
        loop {
            actix_rt::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });

    let _ = tokio::try_join!(signal_handle, ws_handle);
    Ok(())
}
