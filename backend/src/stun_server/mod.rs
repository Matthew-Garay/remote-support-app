//! STUN Server (stub).
//! En producción se usa Coturn (ver docker-compose.yml).

pub mod stun_server;

pub use stun_server::StunServer;
