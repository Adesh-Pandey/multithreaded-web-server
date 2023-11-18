use rust_multithreaded_server::ThreadPool;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, thread};

pub fn result_extractor<T, E: std::fmt::Display>(value: Result<T, E>) -> T {
    match value {
        Ok(v) => return v,
        Err(e) => panic!("failed with error: {}", e),
    }
}

fn main() {
    let listener = result_extractor(TcpListener::bind("127.0.0.1:8000"));

    let thread_pool = ThreadPool::new(10);

    for stream in listener.incoming() {
        let stream = result_extractor(stream);

        thread_pool.execute(|| {
            handle_connection(stream);
        })
    }
    println!("finished serving")
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    let _ = stream.read(&mut buffer);

    let get = b"GET / HTTP/1.1\r\n";

    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, file_name) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let file_content = fs::read_to_string(file_name).unwrap_or_else(|_| {
        println!("Error while parsing {}", file_name);
        String::from("")
    });
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        file_content.len(),
        file_content
    );

    let _ = stream.write(response.as_bytes());

    let _ = stream.flush().unwrap();
}
