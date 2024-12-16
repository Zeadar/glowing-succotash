use data_structs::{Settings, Sql, Task};
use mime_guess;
use rusqlite::Connection;
use serde::de::IntoDeserializer;
use std::{
    fs,
    io::{prelude::*, BufReader},
    mem::take,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
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

    let sql_connection = Connection::open(settings.data_path.as_str()).unwrap();
    let sql_connection = Arc::new(Mutex::new(sql_connection));

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
        let sql_connection = sql_connection.clone();

        pool.execute(|| {
            //lets have a limit of 10 mibibytes as for now
            let mut buf_reader = BufReader::new(&mut stream).take(10240);
            // let http_request: Vec<String> = buf_reader
            //     .lines()
            //     .map(|result| result.unwrap())
            //     .take_while(|line| !line.is_empty())
            //     .collect();

            let mut http_request: Vec<String> = Vec::new();
            let mut buffer = String::new();

            loop {
                match buf_reader.read_line(&mut buffer) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error reading: {err}");
                    }
                }
                let trim_line = buffer.trim();
                if trim_line.is_empty() {
                    break;
                }
                http_request.push(trim_line.to_string());
                buffer.clear();
            }

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
                    if request_path.contains("/api/") {
                        handle_get_api(stream, sql_connection, request_path);
                    } else {
                        handle_get_file(stream, settings, request_path);
                    }
                }
                "POST" => {
                    if request_path.contains("/api/") {
                        let content_length: u64 = http_request
                            .iter()
                            .find(|line| line.to_lowercase().contains("content-length"))
                            .and_then(|line| line.split(":").nth(1))
                            .and_then(|length| length.trim().parse().ok())
                            .unwrap_or(0);

                        if content_length > 0 {
                            let mut body = String::with_capacity(content_length as usize);
                            buf_reader
                                .take(content_length)
                                .read_to_string(&mut body)
                                .unwrap();

                            println!("{body}");
                        }

                        handle_post_api(stream, sql_connection, request_path);
                    } else {
                        serve_404_json(stream, format!("Invalid api: {request_path}"));
                    }
                }
                _ => {
                    serve_404_html(stream, format!("Unrecognized request type {request_type}"));
                    return;
                }
            }
        });
    }
}

fn handle_post_api(
    mut stream: TcpStream,
    sql_connection: Arc<Mutex<Connection>>,
    request_path: &str,
) {
    match request_path {
        "/api/tasks" => {
            // let task = Task::from_json()
            serve_204_nocontent(stream);
        }
        _ => {
            serve_404_json(stream, format!("Invalid api: {request_path}"));
        }
    }
    // let insert_result = sql_connection
    //     .execute(test_task.to_sql_insert().as_str(), ())
    //     .unwrap();
}

fn handle_get_api(stream: TcpStream, sql_connection: Arc<Mutex<Connection>>, request_path: &str) {
    match request_path {
        "/api/tasks" => {
            let json_tasks = match query_to_json(sql_connection, "SELECT * FROM tasks") {
                Ok(strings) => strings.join(","),
                Err(err) => {
                    serve_500_json(stream, err.to_string());
                    return;
                }
            };
            serve_json(stream, format!("[{json_tasks}]"));
        }
        _ => {
            serve_404_json(stream, format!("Invalid api: {request_path}"));
        }
    }
}

fn handle_get_file(mut stream: TcpStream, settings: Arc<Settings>, request_path: &str) {
    let file_path = format!("{}{request_path}", settings.root_path);
    let file_data = match fs::read(&file_path) {
        Ok(data) => data,
        Err(err) => {
            serve_404_html(stream, err.to_string());
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

fn serve_json(mut stream: TcpStream, body: String) {
    let body = body.as_bytes();
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    match stream.write_all(header.as_bytes()) {
        Err(err) => {
            println!("Could not write header to stream");
            println!("{err}");
        }
        _ => {}
    }
    match stream.write_all(body) {
        Err(err) => {
            println!("Could not write header to stream");
            println!("{err}");
        }
        _ => {}
    }
}

fn query_to_json(
    sql_connection: Arc<Mutex<Connection>>,
    sql_query: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut results = Vec::new();

    {
        let conn = sql_connection.lock().unwrap();
        let mut statement = conn.prepare(sql_query)?;
        //Would totally love to drop the connection mutex before any data conversions,
        //however, the data from 'query()' does not live long enough rip
        let query = statement.query_map([], |row| Task::from_sql_row(row))?;

        results.extend(query);
    }

    let json: Vec<String> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .map(|task| task.to_json())
        .collect();

    Ok(json)
}

fn serve_404_json(mut stream: TcpStream, message: String) {
    let message = format!("{{\"error\":{{\"code\":404,\"message\":\"404 Resource not found\",\"internalMessage\":\"{message}\"}}}}");
    let response = format!(
        "HTTP/1.1 404 Resource Not Found\r\nContent-Length: {}\r\n\r\n{}",
        message.as_bytes().len(),
        message
    );
    match stream.write_all(response.as_bytes()) {
        Err(err) => {
            println!("Could not write 404 message to stream");
            println!("{err}");
        }
        _ => {}
    };
}

fn serve_500_json(mut stream: TcpStream, message: String) {
    let message = format!("{{\"error\":{{\"code\":500,\"message\":\"500 Internal Server Error\",\"internalMessage\":\"{message}\"}}}}");
    let response = format!(
        "HTTP/1.1 500 Internal Server Error\r\nContent-Length: {}\r\n\r\n{}",
        message.as_bytes().len(),
        message
    );
    match stream.write_all(response.as_bytes()) {
        Err(err) => {
            println!("Could not write 500 message to stream");
            println!("{err}");
        }
        _ => {}
    };
}

fn serve_204_nocontent(mut stream: TcpStream) {
    let header = "HTTP/1.1 204 No Content\r\n";
    match stream.write_all(header.as_bytes()) {
        Err(err) => {
            println!("Could not write 204 message to stream");
            println!("{err}");
        }
        _ => {}
    }
}

fn serve_404_html(mut stream: TcpStream, message: String) {
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
