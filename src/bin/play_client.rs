use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use windows_service::service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::{define_windows_service, service_dispatcher};

const SERVICE_NAME: &str = "RUST_AUDIO_TROLLER";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
const RETRY_DURATION: tokio::time::Duration = tokio::time::Duration::from_secs(5);

mod play_server;

fn main() -> anyhow::Result<()>
{
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

define_windows_service!(ffi_service_main, my_service_main);

fn my_service_main(_arguments: Vec<std::ffi::OsString>)
{
    if let Err(e) = run_service()
    {
        eprintln!("Service failed: {:?}", e);
    }
}

fn run_service() -> anyhow::Result<()>
{
    let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();

    let status_handle = service_control_handler::register(SERVICE_NAME, move |control_event| match control_event
    {
        ServiceControl::Stop | ServiceControl::Interrogate =>
        {
            shutdown_tx.send(()).unwrap();
            ServiceControlHandlerResult::NoError
        }
        _ => ServiceControlHandlerResult::NotImplemented
    })?;

    status_handle.set_service_status(ServiceStatus {
        service_type:      SERVICE_TYPE,
        current_state:     ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        checkpoint:        0,
        wait_hint:         tokio::time::Duration::default(),
        process_id:        None,
        exit_code:         ServiceExitCode::Win32(0)
    })?;

    tokio::runtime::Runtime::new()?.block_on(async {
        tokio::select! {
            _ = main_task() => (),
            _ = shutdown_rx.recv() => (),
        }
    });

    status_handle.set_service_status(ServiceStatus {
        service_type:      SERVICE_TYPE,
        current_state:     ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code:         ServiceExitCode::Win32(0),
        checkpoint:        0,
        wait_hint:         tokio::time::Duration::default(),
        process_id:        None
    })?;

    Ok(())
}

async fn main_task()
{
    loop
    {
        if let Err(e) = connect().await
        {
            eprintln!("error: {}", e);
        };

        tokio::time::sleep(RETRY_DURATION).await;
    }
}

async fn connect() -> anyhow::Result<()>
{
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&stream_handle)?;

    loop
    {
        let mut stream = tokio::net::TcpStream::connect(format!("{}:{}", play_server::PUBLIC_IP, play_server::PORT)).await?;
        let mut buffer = bytes::BytesMut::new();

        loop
        {
            if stream.read_buf(&mut buffer).await? == 0
            {
                break;
            }
        }

        let buffer_cursor = std::io::Cursor::new(buffer);
        let buffer_reader = std::io::BufReader::new(buffer_cursor);

        let rodio_source = rodio::Decoder::new(buffer_reader)?;

        sink.append(rodio_source);
        sink.sleep_until_end();
    }
}
