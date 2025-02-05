use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{io, thread};
extern crate simplelog;
use simplelog::*;
use std::env;

fn forward(mut from: TcpStream, mut to: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 4096];
    loop {
        let bytes_read = from.read(&mut buffer)?;
        log::info!("Read {} bytes", bytes_read);

        if bytes_read == 0 {
            break;
        }
        to.write_all(&buffer[..bytes_read])?;
    }
    Ok(())
}

fn handle_client(client_stream: TcpStream, target_addr: &str) -> io::Result<()> {
    match TcpStream::connect(target_addr) {
        Ok(server_stream) => {
            log::info!("Forwarding connection to {}", target_addr);
            let client = client_stream.try_clone()?;
            let server = server_stream.try_clone()?;
            let _ = thread::spawn(move || forward(client, server));
            let _ = thread::spawn(move || forward(server_stream, client_stream));
        }
        Err(e) => log::error!("Connection failed: {}", e),
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
        "Port forwarder running on 127.0.0.1:3003 -> {}",
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
