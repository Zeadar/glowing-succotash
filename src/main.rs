use chrono::Utc;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

const ADDR: &str = "127.0.0.1";
const PORT: &str = "7878";

fn main() {
    let start_time = Utc::now();
    let listener = TcpListener::bind(format!("{ADDR}:{PORT}")).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let now = Utc::now() - start_time;
        println!("{}\t: Connection established!", now.num_milliseconds());
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");
}
