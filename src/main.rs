use clap::Parser;
use simplelog::*;
use tokio::{
    io,
    net::{TcpListener, TcpStream},
    signal,
};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, short = 's')]
    src_port: String,

    #[clap(long, short = 'd')]
    dst_port: String,
}

async fn handle_client(client_stream: TcpStream, target_addr: &str) -> io::Result<()> {
    match TcpStream::connect(target_addr).await {
        Ok(server_stream) => {
            log::info!("Forwarding connection to {}", target_addr);
            let (mut client_reader, mut client_writer) = client_stream.into_split();
            let (mut server_reader, mut server_writer) = server_stream.into_split();

            // Forward data between client and server
            let client_to_server = tokio::spawn(async move {
                if let Err(e) = tokio::io::copy(&mut client_reader, &mut server_writer).await {
                    log::error!("Error forwarding data from client to server: {}", e);
                }
            });
            let server_to_client = tokio::spawn(async move {
                if let Err(e) = tokio::io::copy(&mut server_reader, &mut client_writer).await {
                    log::error!("Error forwarding data from server to client: {}", e);
                }
            });
            // Wait for both tasks to complete
            let _ = tokio::join!(client_to_server, server_to_client);
        }
        Err(e) => log::error!("Can not connect to {}: {}", target_addr, e),
    }
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let src_connection_string = format!("127.0.0.1:{}", args.src_port);
    let dst_connection_string = format!("127.0.0.1:{}", args.dst_port);

    let listener = TcpListener::bind(&src_connection_string).await?;
    log::info!(
        "Port forwarder running on 127.0.0.1:{} -> {}",
        args.src_port,
        dst_connection_string
    );

    let shutdown_signal = signal::ctrl_c();

    let server_loop = async {
        loop {
            match listener.accept().await {
                Ok((client_stream, _)) => {
                    let target_addr = dst_connection_string.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(client_stream, &target_addr).await {
                            log::error!("Failed to handle client: {}", e);
                        }
                    });
                }
                Err(e) => log::error!("Connection failed: {}", e),
            }
        }
    };

    tokio::select! {
        _ = server_loop => {},
        _ = shutdown_signal => {
            log::info!("Received Ctrl-C, shutting down gracefully...");
        },
    }

    Ok(())
}
