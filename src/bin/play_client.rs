use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;
use windows_service::service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::{define_windows_service, service_dispatcher};

mod play_server;

const SERVICE_NAME: &str = "PlayClient";

fn main() -> anyhow::Result<()> {
    Ok(service_dispatcher::start(SERVICE_NAME, ffi_service_main)?)
}

define_windows_service!(ffi_service_main, service_main);

fn service_main(_args: Vec<std::ffi::OsString>) {
    let shutdown_token = CancellationToken::new();

    let status_handle = {
        let shutdown_token = shutdown_token.clone();
        service_control_handler::register(SERVICE_NAME, move |control_event| match control_event {
            ServiceControl::Stop => {
                shutdown_token.cancel();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented
        })
        .unwrap()
    };

    let service_status = ServiceStatus {
        service_type:      ServiceType::OWN_PROCESS,
        current_state:     ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code:         ServiceExitCode::Win32(0),
        checkpoint:        0,
        wait_hint:         std::time::Duration::from_secs(5),
        process_id:        Some(std::process::id())
    };

    _ = status_handle.set_service_status(service_status.clone());

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let restart = rt.block_on(async {
        tokio::select! {
            _ = main_task() => true,
            _ = shutdown_token.cancelled() => false
        }
    });

    if restart {
        std::process::exit(1);
    }

    _ = status_handle.set_service_status(ServiceStatus { current_state: ServiceState::Stopped, ..service_status });
}

async fn main_task() {
    loop {
        _ = connect().await;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn connect() -> anyhow::Result<()> {
    let (_rodio_stream, rodio_stream_handle) = rodio::OutputStream::try_default()?;
    let rodio_sink = rodio::Sink::try_new(&rodio_stream_handle)?;

    loop {
        let mut stream = tokio::net::TcpStream::connect(("bits-projects.uk", play_server::PORT)).await?;
        let mut buffer = bytes::BytesMut::new();

        loop {
            if stream.read_buf(&mut buffer).await? == 0 {
                break;
            }
        }

        let buffer_cursor = std::io::Cursor::new(buffer);
        let buffer_reader = std::io::BufReader::new(buffer_cursor);
        let rodio_source = rodio::Decoder::new(buffer_reader)?;

        rodio_sink.append(rodio_source);
        rodio_sink.sleep_until_end();
    }
}
