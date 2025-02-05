use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::{io, thread};
extern crate simplelog;
use simplelog::*;
use std::env;
use std::sync::Arc;

fn forward(from: Arc<TcpStream>, to: Arc<TcpStream>) -> io::Result<()> {
    let mut reader = BufReader::new(from.as_ref());
    let mut writer = BufWriter::new(to.as_ref());
    //Vec::with_capacity(4096) creates an empty vector,
    // but read(&mut buffer) requires the buffer to have pre-allocated space.
    let mut buffer = [0; 4096];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read])?;
        writer.flush()?;
    }
    log::info!("Closing connection");
    //BufWriter may hold data in its buffer instead of immediately sending it.
    let _ = to.shutdown(Shutdown::Both); // Properly close the socket
    Ok(())
}

fn handle_client(client_stream: TcpStream, target_addr: &str) -> io::Result<()> {
    match TcpStream::connect(target_addr) {
        Ok(server_stream) => {
            log::info!("Forwarding connection to {}", target_addr);
            // use Arc more safer for thread safe
            let client = Arc::new(client_stream);
            let server = Arc::new(server_stream);
            let client_clone = Arc::clone(&client);
            let server_clone = Arc::clone(&server);
            // why two threads:
            // bi-directional flow
            // concurrency
            // non blocking
            // scored threads for clean exit, no leftover
            // thread::spawn -> non blocking threads, but may leave lingering threads
            // thread::scope -> safer, but blocks  the funciton forver, but cleaned up automatically
            thread::spawn(move || forward(client, server));
            thread::spawn(move || forward(server_clone, client_clone));
        }
        Err(e) => log::error!("Can not connect to {} : {}", target_addr, e),
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <from> <to> ", args[0]);
        std::process::exit(1);
    }
    let src_port = &args[1];
    let dst_port = &args[2];

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();
    let src_connection_string = format!("127.0.0.1:{}", src_port);
    let dst_connection_string = format!("127.0.0.1:{}", dst_port);

    let listener = TcpListener::bind(&src_connection_string)?;
    log::info!(
        "Port forwarder running on 127.0.0.1:{} -> {}",
        src_port,
        dst_connection_string
    );
    for stream in listener.incoming() {
        match stream {
            Ok(client_stream) => {
                let target_addr = dst_connection_string.to_string();
                thread::spawn(move || handle_client(client_stream, &target_addr));
            }
            Err(e) => log::error!("Connection failed: {}", e),
        }
    }
    Ok(())
}
