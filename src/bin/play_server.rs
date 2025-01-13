use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub const PUBLIC_IP: &str = "127.0.0.1";
pub const PORT: u16 = 6969;

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    let mut args = std::env::args().into_iter();

    let mut file_path = None;
    let mut clients_count: Option<usize> = None;

    while let Some(arg) = args.next()
    {
        match arg.as_str()
        {
            "-file" => file_path = args.next(),
            "-clients" => clients_count = args.next().map(|x| x.parse()).transpose()?,
            _ => ()
        }
    }

    let file_path = file_path.ok_or(anyhow::anyhow!("file not specified (-file [string])"))?;
    let clients_count = clients_count.ok_or(anyhow::anyhow!("clients count missing (-clients [u16])"))?;

    let mut file = tokio::fs::File::open(&file_path).await?;
    let mut file_buffer = Vec::new();
    loop
    {
        if file.read_buf(&mut file_buffer).await? == 0
        {
            break;
        }
    }

    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;
    println!("SERVER\tPORT {}", PORT);

    let clients_written = Arc::new(Mutex::new(HashSet::new()));

    loop
    {
        if clients_written.lock().await.len() == clients_count
        {
            return Ok(());
        }

        if let Ok(Ok((stream, sockaddr))) = tokio::time::timeout(tokio::time::Duration::from_millis(200), listener.accept()).await
        {
            if clients_written.lock().await.contains(&sockaddr.ip())
            {
                continue;
            }

            tokio::spawn(client(stream, sockaddr.ip(), file_buffer.clone(), clients_written.clone()));
        }
    }
}

async fn client(mut stream: TcpStream, ipaddr: IpAddr, file_buffer: Vec<u8>, clients_written: Arc<Mutex<HashSet<IpAddr>>>) -> anyhow::Result<()>
{
    println!("CLIENT\t{}\tCONNECTED", ipaddr);

    stream.write_all(&file_buffer).await?;

    clients_written.lock().await.insert(ipaddr);

    println!("CLIENT\t{}\tFILE SENT", ipaddr);

    stream.shutdown().await?;

    println!("CLIENT\t{}\tDISCONNECTED", ipaddr);

    Ok(())
}
