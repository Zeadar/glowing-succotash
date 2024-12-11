use data_structs::{Settings, Sql, Task};
use mime_guess;
use rusqlite::Connection;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
};
use webber::ThreadPool;

mod data_structs;

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
    let test_task = fs::read_to_string("testtask.json").unwrap();
    // println!("{test_task}");
    let test_task: Task = serde_json::from_str(test_task.as_str()).unwrap();
    println!("{}", test_task.to_sql());
    let sql_connection = Connection::open(settings.data_path.as_str()).unwrap();
    let insert_result = sql_connection
        .execute(test_task.to_sql().as_str(), ())
        .unwrap();

    let mut stmt = sql_connection.prepare("SELECT * FROM tasks").unwrap();
    let task_iter = stmt
        .query_map([], |row| {
            rusqlite::Result::Ok({
                //TODO implement from_sql_row
                // let hi = row.get_unwrap(0);
            })
        })
        .unwrap();

    println!("Insert result {insert_result}");

    //EXPERIMENT END

    let addr = format!("{}:{}", settings.bind_addr, settings.bind_port);
    println!("{addr}");

    let pool = ThreadPool::new(settings.n_threads);
    let listener: TcpListener = match TcpListener::bind(&addr) {
        Ok(listener) => listener,
        Err(err) => {
            println!("Could not bind on address {}", addr);
            panic!("{err}");
        }
    };
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let settings = settings.clone();

        pool.execute(|| {
            handle_connection(stream, settings);
        });

        println!("Shutting down...");
    }
}

fn handle_connection(mut stream: TcpStream, settings: Arc<Settings>) {
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

    let request_type = request_line[0];
    let request_path = match request_line[1] {
        "/" => "/index.html",
        path => path,
    };

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
