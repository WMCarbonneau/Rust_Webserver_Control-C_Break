use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use web_server_1::ThreadPool;
use ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;


fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    let pool = ThreadPool::new(4);
  // handle interrupt currently onlly quits upon next request. This is because it halts when looking its next request.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    loop {
        if running.load(Ordering::SeqCst) == false {break} // interrupt handler end
        let stream = match listener.accept() {
            Ok((_socket, _)) => _socket,
            Err(_) => panic!("Could not accept listener from TcpStream, aborting."),
        };
        // println!("{}", running.load(Ordering::SeqCst));

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = match fs::read_to_string(filename){
        Ok(str) => {str}
        Err(error) => {
            println!("Error, could not read file, {}", error);
            let res = r#"<!DOCTYPE html>
            <html lang="en">
              <head>
                <meta charset="utf-8">
                <title>404 Not Found</title>
              </head>
              <body>
                <h1>404</h1>
                <p>The page you are looking for cannot be found.</p>
              </body>
            </html>"#.to_string();
            res
    }
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
