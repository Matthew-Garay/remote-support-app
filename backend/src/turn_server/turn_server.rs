//! TurnServer: Placeholder. TURN se delega a Coturn en Docker.

pub struct TurnServer;

impl TurnServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, port: u16) {
        tracing::info!("TURN placeholder en puerto {} (usa Coturn para producción)", port);
    }
}
