use chrono::{Local, Timelike};
use mime_guess;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Deserialize, Serialize, Debug)]
struct Settings {
    root_path: String,
    bind_addr: String,
    bind_port: String,
}

const SETTINGS_PATH: &str = "settings.json";

fn main() {
    let settings = match fs::read_to_string(SETTINGS_PATH) {
        Ok(settings) => settings,
        Err(err) => {
            println!("could open {SETTINGS_PATH}");
            panic!("{err}");
        }
    };
    let settings: Settings = serde_json::from_str(settings.as_str()).unwrap();
    let addr = format!("{}:{}", settings.bind_addr, settings.bind_port);
    println!("{addr}");
    let listener: TcpListener = match TcpListener::bind(&addr) {
        Ok(listener) => listener,
        Err(err) => {
            println!("Could not bind on address {}", addr);
            panic!("{err}");
        }
    };
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let now = Local::now();
        println!(
            "{:02}:{:02}:{:02} : Connection established!",
            now.hour(),
            now.minute(),
            now.second()
        );
        handle_connection(stream, &settings);
        println!()
    }
}

fn handle_connection(mut stream: TcpStream, settings: &Settings) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    if http_request.is_empty() {
        println!("Empty request!");
        return;
    }

    let request_line: Vec<&str> = http_request[0].split(" ").collect();

    let request_type = request_line[0];
    let request_path = match request_line[1] {
        "/" => "/index.html",
        path => path,
    };
    let request_version = request_line[2];

    println!(
        "type {}, path {}, version {}",
        request_type, request_path, request_version
    );

    if request_type != "GET" {
        println!("Request type {} not understood", request_type);
        return;
    }

    let file_path = format!("{}{request_path}", settings.root_path);
    let file_data = match fs::read(&file_path) {
        Ok(data) => data,
        Err(err) => {
            println!("{err}");
            let content404 = content_404(err.to_string());
            let content404_len = content404.len();
            let response = format!(
                "HTTP/1.1 404 NOT FOUND\r\ncontent-length: {content404_len}\r\n\r\n{content404}"
            );
            match stream.write_all(response.as_bytes()) {
                Err(err) => {
                    println!("Could not write 404 message to stream");
                    println!("{err}");
                }
                _ => {}
            }
            return;
        }
    };

    let mime = mime_guess::from_path(&request_path)
        .first_or_octet_stream()
        .to_string();
    println!("Guessed {mime} from {request_path}");
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        mime,
        file_data.len()
    );
    println!("Sent with header:\n{header}");
    match stream.write_all(header.as_bytes()) {
        Err(err) => {
            println!("Could not write header to stream");
            println!("{err}");
        }
        _ => {}
    }
    match stream.write_all(file_data.as_slice()) {
        Err(err) => {
            println!("Could not write content to stream");
            println!("{err}");
        }
        _ => {}
    }
}

fn content_404(message: String) -> String {
    let first = r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Hello!</title>
  </head>
  <body>
    <h1>Oops!</h1>
      <p>Sorry, I don't know what you're asking for.</p>
    <div>
    <p style="background-color: beige; padding: 4px;">
    "#;
    let second = r#"
    </p>
    </div>
  </body>
</html>
    "#;
    return format!("{first}{message}{second}");
}
