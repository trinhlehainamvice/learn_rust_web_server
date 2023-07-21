use learn_rust_web_server::ThreadPool;
use std::fs::read_to_string;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(move || {
            handle_connection(stream);
        })
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request = buf_reader
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let method = http_request.get(0).cloned();
    let (status_line, filepath) = match method.as_deref() {
        Some("GET / HTTP/1.1") => ("HTTP/1.1 200 OK", "examples/contents/home.html"),
        Some("GET /sleep HTTP/1.1") => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "examples/contents/home.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "examples/contents/error_404.html"),
    };

    let content = read_to_string(filepath).unwrap();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );
    stream.write_all(response.as_bytes()).unwrap();
}
