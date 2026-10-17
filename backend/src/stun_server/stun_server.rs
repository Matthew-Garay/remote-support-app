//! StunServer: Placeholder. STUN/TURN se delega a Coturn en Docker.
//! Para desarrollo, este stub acepta peticiones básicas.

pub struct StunServer;

impl StunServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, port: u16) {
        tracing::info!("STUN placeholder en puerto {} (usa Coturn para producción)", port);
    }
}
