use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        // server listens to communication from clients
        let listener = TcpListener::bind("127.0.0.1:8888").unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            handle_connection(stream);
        }
    });

    // server sends work to clients
    let instructions = [
        (r"Ejemplo1ParaTests.rdp.subred0.json", "127.0.0.1:8000"),
        (r"Ejemplo1ParaTests.rdp.subred0.json", "127.0.0.1:8001"),
        (r"Ejemplo1ParaTests.rdp.subred0.json", "127.0.0.1:8002"),
    ];

    for (path, address) in instructions {
        let mut stream = TcpStream::connect(address).unwrap();
        let message = format!("{}\n\n", path);
        stream.write_all(message.as_bytes()).unwrap();
    }

    handle.join().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let message: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("{:?}", message);
}
