use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = parse_cli_args(args);
    let mut role = "master";

    let port = match args.get("port") {
        Some(port) => port.parse::<u16>().expect("port not in correct format"),
        None => 6379,
    };

    let replica: (String, String) = match args.get("replicaof") {
        Some(replica) => {
            eprintln!("{}", replica);
            let split = replica.split(',');
            let split = split.collect::<Vec<&str>>();
            role = "slave";
            (split[0].to_string(), split[1].to_string())
        }
        None => {
            eprintln!("invalid replica");
            ("".to_string(), "".to_string())
        }
    };

    eprintln!("replica: {:?}", replica);
    let store = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    eprintln!("listening on port {}", port);

    for stream in listener.incoming() {
        let store = Arc::clone(&store);
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let handle = thread::spawn(move || {
                    eprintln!("spawned new thread");
                    resp_decoder(stream, store, &role);
                });
                handles.push(handle);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn parse_cli_args(args: Vec<String>) -> HashMap<String, String> {
    let mut arguments: HashMap<String, String> = HashMap::new();
    for (i, _) in args.iter().enumerate() {
        eprintln!("arg: {:?}", args[i]);
        if args[i].starts_with("--") {
            let argkey = args[i].trim_start_matches("--").to_string();
            if argkey == "replicaof" {
                arguments.insert(argkey, format!("{},{}", args[i + 1], args[i + 2]));
            } else {
                arguments.insert(argkey, args[i + 1].to_string());
            }
        }
    }

    eprintln!("arguments: {:?}", arguments);
    arguments
}

fn resp_decoder(
    mut stream: TcpStream,
    store: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    role: &str,
) {
    let mut messages = Vec::new();
    let mut message = String::new();
    let mut buf = [0; 1024];
    loop {
        messages.clear();
        let bytes_read = stream.read(&mut buf).expect("read failed");
        if bytes_read == 0 {
            return;
        }
        eprintln!("buf: {:?}", str::from_utf8(&buf[0..bytes_read]).unwrap());
        match buf[0] {
            b'*' => {
                let mut iter = buf[1..buf.len()].iter();
                while let Some(b) = iter.next() {
                    if *b == b'\r' {
                        messages.push(message);
                        message = String::new();
                        iter.next();
                    } else if *b == b'\0' {
                        iter.next();
                    } else {
                        message.push(char::from_u32(*b as u32).unwrap());
                    }
                }

                eprintln!("messages: {:?}", messages);
                let _message_count: u8 = messages
                    .first()
                    .expect("there should be a number")
                    .parse()
                    .expect("invalid number");
                let first_message = messages.get(2).expect("invalid message");
                eprintln!("first_message: {:?}", first_message);
                match first_message.as_str() {
                    "ping" => {
                        eprintln!("PING");
                        stream.write_all(b"+PONG\r\n").expect("write failed");
                    }
                    "echo" => {
                        eprintln!("ECHO");
                        let echo_message = messages.get(4).expect("invalid message");
                        let echo_message = format!("+{}\r\n", echo_message);
                        stream
                            .write_all(echo_message.as_bytes())
                            .expect("write failed");
                    }
                    "set" => {
                        let mut store = store.lock().unwrap();
                        eprintln!("SET");
                        let key = messages.get(4).expect("invalid message");
                        let value = messages.get(6).expect("invalid message");
                        let expiry_message = match messages.get(8) {
                            Some(message) => {
                                if message == "px" {
                                    messages.get(10).expect("invalid message")
                                } else {
                                    ""
                                }
                            }
                            None => "",
                        };
                        eprintln!("expiry_message: {:?}", expiry_message);
                        let expiry = match expiry_message {
                            "" => 0,
                            _ => expiry_message.parse().expect("invalid number"),
                        };
                        let now = Instant::now();
                        if expiry != 0 {
                            eprintln!("expiry: {:?}", expiry);
                            let expiry_time = now + Duration::from_millis(expiry);
                            store.insert(key.to_string(), (value.to_string(), expiry_time));
                        } else {
                            store.insert(
                                key.to_string(),
                                (value.to_string(), now + Duration::from_secs(3600)),
                            );
                        }
                        stream.write_all(b"+OK\r\n").expect("write failed");
                    }
                    "get" => {
                        eprintln!("GET");
                        let mut store = store.lock().unwrap();
                        let key = messages.get(4).expect("invalid message");
                        let value = match store.get(key) {
                            Some(value) => {
                                if value.1 < Instant::now() {
                                    store.remove(key);
                                    "$-1\r\n".to_string()
                                } else {
                                    format!("+{}\r\n", value.0)
                                }
                            }
                            None => "$-1\r\n".to_string(),
                        };
                        stream.write_all(value.as_bytes()).expect("write failed");
                    }
                    "info" => {
                        eprintln!("INFO");
                        let mut message = String::new();
                        message.push_str("#Replication\n");
                        message.push_str(format!("role:{}\n", role).as_str());
                        message = make_bulk_string(&message);
                        eprintln!("{:?}", message);
                        stream.write_all(message.as_bytes()).expect("write failed");
                    }
                    _ => {
                        eprintln!("unknow type: {:?}", first_message);
                        stream.write_all(b"+UNKNOWN\r\n").expect("write failed");
                    }
                }
            }
            _ => {
                eprintln!("invalid");
            }
        }
    }
}

fn make_bulk_string(message: &str) -> String {
    format!("${}\r\n{}\r\n", message.len(), message)
}
