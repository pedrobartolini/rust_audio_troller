use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

pub const PORT: u16 = 6969;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_path = std::env::args().into_iter().skip(1).next().unwrap();

    println!("{}", file_path);

    let file_buffer = {
        let mut file_buffer = Vec::new();
        let mut file = tokio::fs::File::open(&file_path).await?;
        loop {
            if file.read_buf(&mut file_buffer).await? == 0 {
                break;
            }
        }
        file_buffer
    };

    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;
    println!("SERVER\tPORT {}", PORT);

    let acceptor = async move |streams: &mut Vec<(TcpStream, SocketAddr)>| {
        loop {
            let Ok((stream, addr)) = listener.accept().await else { continue };
            streams.push((stream, addr));
        }
    };

    let mut streams = Vec::new();

    tokio::select! {
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => (),
        _ = acceptor(&mut streams) => (),
    };

    let clients_written = Arc::new(RwLock::new(0));

    let streams_count = streams.len();
    for (stream, addr) in streams {
        tokio::spawn(client(stream, addr.ip(), file_buffer.clone(), clients_written.clone()));
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        if *clients_written.read().await == streams_count {
            break Ok(());
        }
    }
}

async fn client(mut stream: TcpStream, ipaddr: IpAddr, file_buffer: Vec<u8>, clients_written: Arc<RwLock<usize>>) -> anyhow::Result<()> {
    println!("CLIENT\t{}\tCONNECTED", ipaddr);

    let result = stream.write_all(&file_buffer).await;

    *clients_written.write().await += 1;

    if result.is_ok() {
        println!("CLIENT\t{}\tFILE SENT", ipaddr);
        stream.shutdown().await?;
    }

    println!("CLIENT\t{}\tDISCONNECTED", ipaddr);

    Ok(())
}
