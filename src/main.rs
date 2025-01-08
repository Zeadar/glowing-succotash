use chrono::{TimeDelta, Utc};
use data_structs::{JsonError, SessionUser, Settings, Sql, Task, User};
use mime_guess;
use rusqlite::Connection;
use sha256::digest;
use std::{
    collections::HashMap,
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex, RwLock},
};
use threadspool::ThreadSpool;
use uuid::Uuid;

mod data_structs;
mod threadspool;

const SETTINGS_PATH: &str = "settings.json";

enum HttpError {
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    LengthRequired = 411,
    InternalServerError = 500,
}

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
    sql_connection
        .execute("PRAGMA foreign_keys = ON;", [])
        .unwrap();
    let sql_connection = Arc::new(Mutex::new(sql_connection));

    let addr = format!("{}:{}", settings.bind_addr, settings.bind_port);
    println!("{addr}");

    let session: HashMap<String, SessionUser> = HashMap::new();
    let session = Arc::new(RwLock::new(session));

    let spool = ThreadSpool::new(settings.n_threads);
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
        let sql_connection = sql_connection.clone();
        let session = session.clone();

        spool.execute(move || {
            //lets have a limit of one mibibyte as for now
            let mut buf_reader = BufReader::new(&stream).take(1048576);

            let mut http_header: Vec<String> = Vec::new();
            let mut buffer = String::new();

            //Used to make iterator with lines() but that took ownership
            //over the reader which made it impossible to extract the body
            loop {
                match buf_reader.read_line(&mut buffer) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("{err}");
                    }
                }
                let trim_line = buffer.trim();
                if trim_line.is_empty() {
                    break;
                }
                http_header.push(trim_line.to_string());
                buffer.clear();
            }

            if http_header.is_empty() {
                println!("Empty request!");
                return;
            }

            let top_header: Vec<&str> = http_header[0].split(" ").collect();
            let request_line = match top_header
                .iter()
                .filter(|header| !header.contains("HTTP"))
                .map(|h| h.to_string())
                .reduce(|a, b| format!("{a} {b}"))
            {
                Some(h) => h,
                None => {
                    serve_404_html(stream, String::from("Your header sucks"));
                    return;
                }
            };

            println!("{request_line}");

            let header_map: HashMap<String, &str> = http_header[1..]
                .into_iter()
                .filter_map(|line| line.split_once(":"))
                .map(|pair| {
                    let (key, value) = pair;
                    (key.to_lowercase(), value.trim())
                })
                .collect();

            if request_line.contains("/api/") {
                handle_api_request(
                    &stream,
                    buf_reader,
                    header_map,
                    sql_connection,
                    session,
                    request_line,
                );
            } else {
                handle_file_request(stream, settings, request_line);
            }
        });
    }
}

fn handle_api_request(
    stream: &TcpStream,
    buf_reader: std::io::Take<BufReader<&TcpStream>>,
    header: HashMap<String, &str>,
    sql_connection: Arc<Mutex<Connection>>,
    session: Arc<RwLock<HashMap<String, SessionUser>>>,
    request_line: String,
) {
    match request_line.as_str() {
        "GET /api/task" => {
            let user_id = match extract_user_id(&header, session) {
                Ok(user_id) => user_id,
                Err(err) => {
                    serve_error_json(stream, HttpError::Forbidden, String::from(err));
                    return;
                }
            };

            let json_tasks: Vec<String> = match query_to_object::<Task>(
                sql_connection,
                format!("SELECT * FROM tasks WHERE user_id = '{user_id}'").as_str(),
            ) {
                Ok(vec_of_boxes) => vec_of_boxes
                    .into_iter()
                    .map(|task| task.to_json())
                    .collect(),
                Err(err) => {
                    serve_error_json(stream, HttpError::InternalServerError, err.to_string());
                    return;
                }
            };
            serve_200_json(stream, format!("[{}]", json_tasks.join(",")));
        }
        "POST /api/task" => {
            let user_id = match extract_user_id(&header, session) {
                Ok(user_id) => user_id,
                Err(err) => {
                    serve_error_json(stream, HttpError::Forbidden, String::from(err));
                    return;
                }
            };

            let body = extract_body(stream, buf_reader, header);
            if body.is_none() {
                return;
            }

            let task = match Task::from_json(body.unwrap().as_str()) {
                Ok(task) => task,
                Err(err) => {
                    serve_error_json(stream, HttpError::BadRequest, err.to_string());
                    return;
                }
            };

            let sql_connection = sql_connection.lock().unwrap();
            match sql_connection.execute(
                task.to_sql_insert()
                    .replace("{}", user_id.as_str())
                    .as_str(),
                (),
            ) {
                Ok(_) => {}
                Err(err) => {
                    serve_error_json(stream, HttpError::InternalServerError, err.to_string());
                    return;
                }
            }
            drop(sql_connection);

            serve_200_json(stream, task.to_json());
        }
        "GET /api/user" => {
            let user_id = match extract_user_id(&header, session) {
                Ok(user_id) => user_id,
                Err(err) => {
                    serve_error_json(stream, HttpError::Forbidden, String::from(err));
                    return;
                }
            };

            serve_200_json(stream, format!("{{\"userId\":\"{}\"}}", user_id));
        }
        "POST /api/user" => {
            let body = match extract_body(stream, buf_reader, header) {
                Some(body) => body,
                None => return,
            };

            let mut user = match User::from_json(body.as_str()) {
                Ok(user) => user,
                Err(err) => {
                    serve_error_json(stream, HttpError::BadRequest, err.to_string());
                    return;
                }
            };
            let mut passwd = user.password.as_bytes().to_owned();
            passwd.extend([user.salt]);
            user.password = digest(passwd);

            let sql_connection = sql_connection.lock().unwrap();
            match sql_connection.execute(user.to_sql_insert().as_str(), ()) {
                Ok(_) => {}
                Err(err) => {
                    serve_error_json(stream, HttpError::BadRequest, err.to_string());
                    return;
                }
            }
            drop(sql_connection);

            serve_200_json(stream, user.to_json());
        }
        "POST /api/login" => {
            let body = match extract_body(stream, buf_reader, header) {
                Some(body) => body,
                None => return,
            };

            let user: User = match serde_json::de::from_str(body.as_str()) {
                Ok(login) => login,
                Err(err) => {
                    serve_error_json(stream, HttpError::BadRequest, err.to_string());
                    return;
                }
            };

            let passwd = user.password;

            let user = match query_to_object::<User>(
                sql_connection,
                format!("SELECT * FROM users WHERE username = '{}';", user.username).as_str(),
            ) {
                Ok(user) => user,
                Err(err) => {
                    serve_error_json(stream, HttpError::BadRequest, err.to_string());
                    return;
                }
            };

            let user = match user.first() {
                Some(user) => user,
                None => {
                    serve_error_json(
                        stream,
                        HttpError::BadRequest,
                        String::from("User not found or invalid password"),
                    );
                    return;
                }
            };

            let mut user_passwd: Vec<u8> = passwd.as_bytes().into_iter().map(|b| *b).collect();
            user_passwd.extend([user.salt]);
            let hashed_passwd = digest(user_passwd);
            if user.password == hashed_passwd {
                let session_uuid = Uuid::new_v4();

                {
                    let mut session = session.write().unwrap();
                    session.insert(
                        session_uuid.to_string(),
                        SessionUser {
                            user_id: user.id.clone(),
                            expire: Utc::now() + TimeDelta::seconds(60 * 60),
                        },
                    );
                }

                let json = format!(
                    "{{\"username\": \"{}\",\"userId\":\"{}\",\"authority\":\"{}\"}}",
                    user.username,
                    user.id,
                    session_uuid.to_string(),
                );
                println!("{json}");
                serve_200_json(stream, json);
            } else {
                serve_error_json(
                    stream,
                    HttpError::BadRequest,
                    String::from("User not found or invalid password"),
                );
                return;
            }
        }
        _ => {
            serve_error_json(
                stream,
                HttpError::NotFound,
                format!("What the hell is {request_line} supposed to mean?"),
            );
        }
    }
}

fn extract_user_id(
    header: &HashMap<String, &str>,
    session: Arc<RwLock<HashMap<String, SessionUser>>>,
) -> Result<String, &'static str> {
    let authority = match header.get("authority") {
        Some(auth) => *auth,
        None => {
            return Err("No Authority in header");
        }
    };

    let session_user: SessionUser;
    {
        let session = session.read().unwrap();
        session_user = match session.get(authority) {
            Some(yay) => yay.clone(),
            None => {
                return Err("No user associated with Authority");
            }
        };
    }

    if session_user.expire < Utc::now() {
        let mut session = session.write().unwrap();
        session.remove(authority);
        return Err("Authority expired");
    }

    Ok(session_user.user_id)
}

//TODO return result instead of accepting stream
fn extract_body(
    stream: &TcpStream,
    buf_reader: std::io::Take<BufReader<&TcpStream>>,
    header: HashMap<String, &str>,
) -> Option<String> {
    let content_length = header
        .get("content-length")
        .unwrap_or(&"0")
        .parse()
        .unwrap_or(0);

    let mut body = String::with_capacity(content_length);

    if content_length > 0 {
        match buf_reader
            .take(content_length as u64)
            .read_to_string(&mut body)
        {
            Ok(_) => Some(body),
            Err(err) => {
                serve_error_json(stream, HttpError::BadRequest, err.to_string());
                return None;
            }
        }
    } else {
        serve_error_json(stream, HttpError::LengthRequired, String::new());
        return None;
    }
}

fn handle_file_request(mut stream: TcpStream, settings: Arc<Settings>, request_line: String) {
    let request_path = match request_line.split(" ").last() {
        Some(path) => match path {
            "/" => "/index.html",
            path => path,
        },
        None => {
            serve_404_html(stream, format!("Your header sucks!"));
            return;
        }
    };

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

fn query_to_object<T: Sql>(
    sql_connection: Arc<Mutex<Connection>>,
    sql_query: &str,
) -> rusqlite::Result<Vec<Box<T>>> {
    let mut results = Vec::new();

    {
        let conn = sql_connection.lock().unwrap();
        let mut statement = conn.prepare(sql_query)?;
        //Would totally love to drop the connection mutex before any data conversions,
        //however, the data from 'query()' does not live long enough rip
        let query = statement.query_map([], |row| T::from_sql_row(row))?;

        results.extend(query);
    }

    let vec_of_boxes = results.into_iter().filter_map(|r| r.ok()).collect();

    Ok(vec_of_boxes)
}

fn serve_200_json(mut stream: &TcpStream, body: String) {
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

fn serve_error_json(mut stream: &TcpStream, error: HttpError, internal: String) {
    let body = match error {
        HttpError::BadRequest => JsonError {
            message: "400 Bad Request",
            code: 400,
            internal,
        },
        HttpError::Forbidden => JsonError {
            message: "403 Forbidden",
            code: 403,
            internal,
        },
        HttpError::NotFound => JsonError {
            message: "404 Not Found",
            code: 404,
            internal,
        },
        HttpError::LengthRequired => JsonError {
            message: "411 Length Required",
            code: 411,
            internal,
        },
        HttpError::InternalServerError => JsonError {
            message: "500 Internal Server Error",
            code: 500,
            internal,
        },
    };

    let message = format!("{{\"error\":{}}}", serde_json::to_string(&body).unwrap());

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        body.message,
        message.as_bytes().len(),
        message
    );

    match stream.write_all(response.as_bytes()) {
        Err(err) => {
            println!("Could not write {} message to stream", body.code);
            println!("{err}");
        }
        _ => {}
    };
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
