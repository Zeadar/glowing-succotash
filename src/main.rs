use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Deserialize, Serialize, Debug)]
struct Settings {
    root_path: String,
}

const ADDR: &str = "127.0.0.1";
const PORT: &str = "7878";

fn main() {
    let settings = fs::read_to_string("settings.json").unwrap();
    let settings: Settings = serde_json::from_str(settings.as_str()).unwrap();
    let listener = TcpListener::bind(format!("{ADDR}:{PORT}")).unwrap();
    let start_time = Utc::now();
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let now = Utc::now() - start_time;
        println!("{}\t : Connection established!", now.num_seconds());
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

    let status_line = "HTTP/1.1 200 OK";
    let content = fs::read_to_string(format!("{}/index.html", settings.root_path)).unwrap();
    let content_len = content.len();

    let response = format!("{status_line}\r\nContent-length: {content_len}\r\n\r\n{content}");
    stream.write_all(response.as_bytes()).unwrap();
}
