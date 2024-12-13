use data_structs::{Settings, Sql, Task};
use mime_guess;
use rusqlite::Connection;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
};
use threadspool::ThreadSpool;

mod data_structs;
mod threadspool;

const SETTINGS_PATH: &str = "settings.json";

fn main() {
    let settings = match fs::read_to_string(SETTINGS_PATH) {
        Ok(settings) => settings,
        Err(err) => {
            println!("Error reading {SETTINGS_PATH}");
            panic!("{err}");
        }
    };
    let settings: Settings = match serde_json::from_str(settings.as_str()) {
        Ok(settings) => settings,
        Err(err) => {
            println!("Error parsing {SETTINGS_PATH}");
            panic!("{err}");
        }
    };
    let settings = Arc::new(settings);

    //SQL EXPERIMENT
    // let test_task = fs::read_to_string("testtask.json").unwrap();
    // let test_task = Task::from_json(test_task.as_str()).unwrap();
    // println!("{:?}\n{}", test_task, test_task.to_json());
    let sql_connection = Connection::open(settings.data_path.as_str()).unwrap();
    // let insert_result = sql_connection
    //     .execute(test_task.to_sql_insert().as_str(), ())
    //     .unwrap();
    // println!("insert result {insert_result}");

    let mut stmt = sql_connection.prepare("SELECT * FROM tasks").unwrap();
    let results: Vec<String> = stmt
        .query_map([], |row| Task::from_sql_row(row))
        .unwrap()
        .into_iter()
        .filter_map(|r| r.ok())
        .map(|t| t.to_json())
        .collect();

    for s in results {
        println!("{s}");
    }

    // println!("Insert result {insert_result}");

    //EXPERIMENT END

    let addr = format!("{}:{}", settings.bind_addr, settings.bind_port);
    println!("{addr}");

    let pool = ThreadSpool::new(settings.n_threads);
    let listener: TcpListener = match TcpListener::bind(&addr) {
        Ok(listener) => listener,
        Err(err) => {
            println!("Could not bind on address {}", addr);
            panic!("{err}");
        }
    };
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let settings = settings.clone();

        pool.execute(|| {
            let buf_reader = BufReader::new(&mut stream);
            let http_request: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            if http_request.is_empty() {
                println!("Empty request!");
                return;
            }

            let request_line: Vec<&str> = http_request[0].split(" ").collect();
            if request_line.len() != 3 {
                println!("invalid header\n{}", http_request[0]);
                return;
            }

            let request_type = request_line[0];
            let request_path = match request_line[1] {
                "/" => "/index.html",
                path => path,
            };

            match request_type {
                "GET" => {
                    handle_get_file(stream, settings, request_path);
                }
                _ => {
                    todo!();
                }
            }

            println!("Shutting down...");
        });
    }
}

fn handle_get_file(mut stream: TcpStream, settings: Arc<Settings>, request_path: &str) {
    let file_path = format!("{}{request_path}", settings.root_path);
    let file_data = match fs::read(&file_path) {
        Ok(data) => data,
        Err(err) => {
            serve_404(stream, err.to_string());
            println!("{err}");
            return;
        }
    };

    let mime = mime_guess::from_path(&request_path)
        .first_or_octet_stream()
        .to_string();
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        mime,
        file_data.len()
    );

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

fn serve_404(mut stream: TcpStream, message: String) {
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

    let content404 = format!("{first}{message}{second}");
    let content404_len = content404.len();
    let response =
        format!("HTTP/1.1 404 NOT FOUND\r\ncontent-length: {content404_len}\r\n\r\n{content404}");
    match stream.write_all(response.as_bytes()) {
        Err(err) => {
            println!("Could not write 404 message to stream");
            println!("{err}");
        }
        _ => {}
    }
}
