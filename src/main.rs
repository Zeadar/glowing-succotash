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

struct Response {
    type_text: String,
    content: String,
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
            "{}:{}:{} : Connection established!",
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
    let accept: Vec<&String> = http_request
        .iter()
        .filter(|s| s.contains("Accept:"))
        .collect();

    let accept_text = match accept.first() {
        Some(s) => s.contains("text"),
        _ => {
            println!("No Accept line in Request");
            return;
        }
    };

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

    if accept_text {
        let response: Response =
            match fs::read_to_string(format!("{}/{request_path}", settings.root_path)) {
                Ok(content) => Response {
                    type_text: String::from("200 OK"),
                    content,
                },
                Err(err) => {
                    println!("{err}");
                    Response {
                        type_text: String::from("404 NOT FOUND"),
                        content: fs::read_to_string("404.html").unwrap(),
                    }
                }
            };

        let status_line = format!("HTTP/1.1 {}", response.type_text);
        let content_len = response.content.len();
        let response = format!(
            "{status_line}\r\nContent-length: {content_len}\r\n\r\n{}",
            response.content
        );
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let path = format!("{}{request_path}", settings.root_path);
        let file_data = match fs::read(&path) {
            Ok(data) => data,
            Err(err) => {
                println!("{err}");
                let content404 = fs::read_to_string("404.html").unwrap();
                let content404_len = content404.len();
                let response = format!("HTTP/1.1 404 NOT FOUND\r\ncontent-length: {content404_len}\r\n\r\n{content404}");
                stream.write_all(response.as_bytes()).unwrap();
                return;
            }
        };

        let mime = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();
        println!("Guessed {mime} from {path}");
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            mime,
            file_data.len()
        );
        stream.write_all(header.as_bytes()).unwrap();
        stream.write_all(file_data.as_slice()).unwrap();
    }
}
