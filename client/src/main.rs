mod engine;
mod error;
mod json;
mod polyfill;

use crate::{engine::Engine, polyfill::Lefs};
use error::Result;
use std::env;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let port = &args[1];
    let last_cycle = &args[2];
    let last_cycle = last_cycle.parse::<usize>().unwrap();
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&address).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, last_cycle, &address)?;
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, last_cycle: usize, address: &str) -> Result<()> {
    let buf_reader = BufReader::new(&mut stream);
    let message: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if let Some(path) = message.get(0) {
        println!("{path}");
        let lefs = Lefs::new(path)?;
        let mut engine = Engine::new(lefs);
        engine.simulate(0, last_cycle);

        let ip_server = "127.0.0.1:8888";
        let mut stream = TcpStream::connect(ip_server).unwrap();
        let message = format!("{address} processed petri network {path}\n\n");
        stream.write_all(message.as_bytes()).unwrap();
    }

    Ok(())
}
