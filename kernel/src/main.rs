//! Main entry point for the Remote Support Kernel
//!
//! Este es el proceso que se ejecutará como servicio/demon.
//! Se encarga de:
//! - Inicializar todos los módulos del kernel
//! - Gestionar la comunicación con Flutter UI
//! - Mantener el estado del sistema

mod ffi;
mod screen_capture;
mod input_simulator;
mod encoder;
mod network;

use flutter_rust_bridge::FrbDefault;

#[tokio::main]
async fn main() {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::fmt::env_filter()
                .add_directive(tracing::level::Level::INFO.into())
        )
        .init();

    tracing::info!("Remote Support Kernel iniciado");

    // Crear el API de FFI
    let api = FrbDefault::new(ffi::KernelApiImpl::default());

    // Inicializar sub-sistemas
    let mut screen_capture = screen_capture::ScreenCapture::new();
    let mut input_simulator = input_simulator::InputSimulator::new();
    let mut encoder = encoder::VideoEncoder::new();
    let mut network = network::NetworkEngine::new();

    // Inicializar servicios
    screen_capture.init().await;
    encoder.init().await;
    network.init().await;

    // Bucle principal de la aplicación
    loop {
        // Procesar eventos del UI (captura y deserialización)
        if let Some(event) = api.recv().await {
            match event {
                ffi::FFIEvent::StartCapture { config } => {
                    tracing::info!("Iniciando captura: {:?}", config);
                    screen_capture.start(config).await;
                }
                ffi::FFIEvent::StopCapture {} => {
                    screen_capture.stop().await;
                }
                ffi::FFIEvent::SetEncoderConfig {
                    codec,
                    bitrate,
                } => {
                    encoder.set_config(codec, bitrate).await;
                }
                ffi::FFIEvent::InjectMouseEvent { event } => {
                    input_simulator.inject_mouse(event).await;
                }
                ffi::FFIEvent::InjectKeyEvent { event } => {
                    input_simulator.inject_key(event).await;
                }
                ffi::FFIEvent::GetState {} => {
                    // Devolver estado al UI (implementar serialización)
                    tracing::debug!("Estado del kernel: {}", network.state());
                }
                ffi::FFIEvent::Restart {} => {
                    tracing::info!("Reiniciando kernel...");
                    screen_capture.stop().await;
                    encoder.init().await;
                    network.init().await;
                }
            }
        }
    }
}
